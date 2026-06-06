//! The Postgres-backed workspace + membership store (ADR-0009): create a workspace
//! owned by a user, look up a user's role in a workspace, and list the workspaces a
//! user belongs to. Cloud mode only; runtime-checked sqlx (no offline cache), like
//! the auth stores. The `/sync` gate and the workspace HTTP routes consume this in
//! later increments.

use sqlx::PgPool;
use uuid::Uuid;

use crate::persistence::WorkspaceId;

/// Postgres advisory-lock key serializing the one-time R5 bootstrap across concurrent
/// boots (an arbitrary fixed constant, "nxbs" in ASCII).
const BOOTSTRAP_LOCK: i64 = 0x6e78_6273;

/// A member's role in a workspace (ADR-0009). Maps to the Postgres `workspace_role`
/// enum; an aggregate with three gated states is a discriminated union (rule #6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "workspace_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Owner,
    Editor,
    Viewer,
}

impl Role {
    /// Whether this role may write to the document. Viewers are read-only.
    pub fn can_write(self) -> bool {
        matches!(self, Role::Owner | Role::Editor)
    }
}

/// A workspace the caller belongs to, with their role in it.
#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceSummary {
    pub id: WorkspaceId,
    pub name: String,
    pub role: Role,
}

/// A member of a workspace, for the share panel's people list.
#[derive(Debug, PartialEq, Eq)]
pub struct Member {
    pub user_id: Uuid,
    /// The display handle (email local-part), as in `/auth/me`.
    pub display: String,
    pub role: Role,
}

/// Postgres-backed workspace + membership store. Cheap to clone (the pool is
/// reference-counted).
#[derive(Clone)]
pub struct WorkspaceStore {
    pool: PgPool,
}

impl WorkspaceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a workspace owned by `user_id`: the workspace row plus an `owner`
    /// membership, together in one transaction.
    pub async fn create_with_owner(&self, user_id: Uuid, name: &str) -> sqlx::Result<WorkspaceId> {
        let id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        sqlx::query("insert into workspace (id, name) values ($1, $2)")
            .bind(id)
            .bind(name)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "insert into membership (workspace_id, user_id, role) values ($1, $2, 'owner')",
        )
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(WorkspaceId(id))
    }

    /// The user's role in a workspace, or `None` if they are not a member.
    pub async fn membership(
        &self,
        user_id: Uuid,
        workspace: WorkspaceId,
    ) -> sqlx::Result<Option<Role>> {
        let row: Option<(Role,)> =
            sqlx::query_as("select role from membership where workspace_id = $1 and user_id = $2")
                .bind(workspace.0)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|(role,)| role))
    }

    /// Every workspace the user belongs to, oldest first, with their role.
    pub async fn list_for_user(&self, user_id: Uuid) -> sqlx::Result<Vec<WorkspaceSummary>> {
        let rows: Vec<(Uuid, String, Role)> = sqlx::query_as(
            "select w.id, w.name, m.role from membership m \
             join workspace w on w.id = m.workspace_id \
             where m.user_id = $1 order by w.created_at",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id, name, role)| WorkspaceSummary {
                id: WorkspaceId(id),
                name,
                role,
            })
            .collect())
    }

    /// One-time bootstrap (ADR-0009 R5): when no workspace exists yet, create a single
    /// unclaimed one (a row with no memberships) seeded with `snapshot`, the pre-cloud
    /// local document if the volume still has one. Returns the new id, or `None` if a
    /// workspace already exists (so it runs only on the first cloud boot). A transaction
    /// advisory lock serializes concurrent boots (e.g. a rolling deploy) so exactly one
    /// bootstrap row is ever made. The first user to sign in claims it (`claim_unclaimed_for`).
    pub async fn bootstrap_unclaimed(
        &self,
        name: &str,
        snapshot: Option<&[u8]>,
    ) -> sqlx::Result<Option<WorkspaceId>> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("select pg_advisory_xact_lock($1)")
            .bind(BOOTSTRAP_LOCK)
            .execute(&mut *tx)
            .await?;
        let already: bool = sqlx::query_scalar("select exists(select 1 from workspace)")
            .fetch_one(&mut *tx)
            .await?;
        if already {
            return Ok(None); // tx rolls back on drop, releasing the lock
        }
        let id = Uuid::new_v4();
        sqlx::query("insert into workspace (id, name, snapshot) values ($1, $2, $3)")
            .bind(id)
            .bind(name)
            .bind(snapshot)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(Some(WorkspaceId(id)))
    }

    /// "First sign-in claims it" (ADR-0009 R5): atomically make `user_id` the owner of the
    /// oldest unclaimed workspace (a bootstrap row with no memberships), if one exists.
    /// Returns the claimed id, or `None` when nothing is unclaimed. Idempotent across logins
    /// and race-safe: the row is locked `for update skip locked`, so under a simultaneous
    /// second login the loser's CTE is empty and it claims nothing.
    pub async fn claim_unclaimed_for(&self, user_id: Uuid) -> sqlx::Result<Option<WorkspaceId>> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "with unclaimed as ( \
                 select w.id from workspace w \
                 where not exists (select 1 from membership m where m.workspace_id = w.id) \
                 order by w.created_at \
                 limit 1 \
                 for update skip locked \
             ) \
             insert into membership (workspace_id, user_id, role) \
             select id, $1, 'owner' from unclaimed \
             returning workspace_id",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(id,)| WorkspaceId(id)))
    }

    /// Add (or re-role) the account that owns `email` as a member of `workspace`.
    /// Returns `None` if no account has that email (the invitee must have signed in at
    /// least once); the caller maps that to a 404.
    pub async fn add_member_by_email(
        &self,
        workspace: WorkspaceId,
        email: &str,
        role: Role,
    ) -> sqlx::Result<Option<()>> {
        let row: Option<(Uuid,)> =
            sqlx::query_as("select user_id from email_identity where email = $1")
                .bind(email)
                .fetch_optional(&self.pool)
                .await?;
        let Some((user_id,)) = row else {
            return Ok(None);
        };
        sqlx::query(
            "insert into membership (workspace_id, user_id, role) values ($1, $2, $3) \
             on conflict (workspace_id, user_id) do update set role = $3",
        )
        .bind(workspace.0)
        .bind(user_id)
        .bind(role)
        .execute(&self.pool)
        .await?;
        Ok(Some(()))
    }

    /// Everyone in `workspace`, oldest membership first (so the owner leads), with their
    /// display handle (email local-part) and role. Backs the share panel's people list.
    pub async fn list_members(&self, workspace: WorkspaceId) -> sqlx::Result<Vec<Member>> {
        let rows: Vec<(Uuid, String, Role)> = sqlx::query_as(
            "select m.user_id, split_part(e.email, '@', 1) as display, m.role \
             from membership m \
             join email_identity e on e.user_id = m.user_id \
             where m.workspace_id = $1 \
             order by m.created_at",
        )
        .bind(workspace.0)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(user_id, display, role)| Member {
                user_id,
                display,
                role,
            })
            .collect())
    }

    /// Change a member's role. Returns `None` if they are not a member (the caller maps
    /// that to a 404).
    pub async fn set_member_role(
        &self,
        workspace: WorkspaceId,
        user_id: Uuid,
        role: Role,
    ) -> sqlx::Result<Option<()>> {
        let done =
            sqlx::query("update membership set role = $1 where workspace_id = $2 and user_id = $3")
                .bind(role)
                .bind(workspace.0)
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        Ok((done.rows_affected() > 0).then_some(()))
    }

    /// Remove a member from a workspace. Returns `None` if they were not a member (→ 404).
    pub async fn remove_member(
        &self,
        workspace: WorkspaceId,
        user_id: Uuid,
    ) -> sqlx::Result<Option<()>> {
        let done = sqlx::query("delete from membership where workspace_id = $1 and user_id = $2")
            .bind(workspace.0)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok((done.rows_affected() > 0).then_some(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;

    async fn fresh_db() -> (ContainerAsync<Postgres>, PgPool) {
        let container = Postgres::default().start().await.expect("start postgres");
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("mapped port");
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        let pool = PgPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        (container, pool)
    }

    async fn seed_user(pool: &PgPool) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("insert into app_user (id) values ($1)")
            .bind(id)
            .execute(pool)
            .await
            .expect("seed user");
        id
    }

    async fn seed_user_with_email(pool: &PgPool, email: &str) -> Uuid {
        let id = seed_user(pool).await;
        sqlx::query("insert into email_identity (email, user_id) values ($1, $2)")
            .bind(email)
            .bind(id)
            .execute(pool)
            .await
            .expect("seed email");
        id
    }

    #[tokio::test]
    async fn create_with_owner_makes_an_owner_membership() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());
        let user = seed_user(&pool).await;

        let ws = store.create_with_owner(user, "My Overlay").await.unwrap();

        assert_eq!(store.membership(user, ws).await.unwrap(), Some(Role::Owner));
        let name: String = sqlx::query_scalar("select name from workspace where id = $1")
            .bind(ws.0)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(name, "My Overlay");
    }

    #[tokio::test]
    async fn membership_is_none_for_a_non_member() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());
        let owner = seed_user(&pool).await;
        let stranger = seed_user(&pool).await;
        let ws = store.create_with_owner(owner, "Private").await.unwrap();

        assert_eq!(store.membership(stranger, ws).await.unwrap(), None);
    }

    #[tokio::test]
    async fn list_for_user_returns_each_membership() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());
        let user = seed_user(&pool).await;

        let a = store.create_with_owner(user, "A").await.unwrap();
        let b = store.create_with_owner(user, "B").await.unwrap();

        let list = store.list_for_user(user).await.unwrap();
        let ids: Vec<WorkspaceId> = list.iter().map(|w| w.id).collect();
        assert!(ids.contains(&a) && ids.contains(&b));
        assert!(list.iter().all(|w| w.role == Role::Owner));
    }

    #[tokio::test]
    async fn each_role_round_trips_through_postgres() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());
        let owner = seed_user(&pool).await;
        let ws = store.create_with_owner(owner, "Team").await.unwrap();

        // Bind each role into Postgres and read it back (Encode + Decode).
        for role in [Role::Owner, Role::Editor, Role::Viewer] {
            let member = seed_user(&pool).await;
            sqlx::query("insert into membership (workspace_id, user_id, role) values ($1, $2, $3)")
                .bind(ws.0)
                .bind(member)
                .bind(role)
                .execute(&pool)
                .await
                .unwrap();
            assert_eq!(store.membership(member, ws).await.unwrap(), Some(role));
        }
    }

    #[tokio::test]
    async fn bootstrap_creates_one_unclaimed_workspace_with_the_snapshot() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());

        let id = store
            .bootstrap_unclaimed("My Overlays", Some(&[1, 2, 3]))
            .await
            .unwrap()
            .expect("a workspace was created");

        // The snapshot is preserved, and the row has no memberships (it is unclaimed).
        let snapshot: Option<Vec<u8>> =
            sqlx::query_scalar("select snapshot from workspace where id = $1")
                .bind(id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(snapshot, Some(vec![1, 2, 3]));
        let members: i64 =
            sqlx::query_scalar("select count(*) from membership where workspace_id = $1")
                .bind(id.0)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(members, 0, "the bootstrap workspace starts unclaimed");
    }

    #[tokio::test]
    async fn bootstrap_is_a_no_op_once_a_workspace_exists() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());
        let user = seed_user(&pool).await;
        store.create_with_owner(user, "Existing").await.unwrap();

        assert_eq!(
            store
                .bootstrap_unclaimed("My Overlays", None)
                .await
                .unwrap(),
            None,
            "no bootstrap when a workspace already exists"
        );
        let count: i64 = sqlx::query_scalar("select count(*) from workspace")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn the_first_caller_claims_the_bootstrapped_workspace_as_owner() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());
        let id = store.bootstrap_unclaimed("W", None).await.unwrap().unwrap();
        let user = seed_user(&pool).await;

        assert_eq!(store.claim_unclaimed_for(user).await.unwrap(), Some(id));
        assert_eq!(store.membership(user, id).await.unwrap(), Some(Role::Owner));
    }

    #[tokio::test]
    async fn claim_is_a_no_op_when_nothing_is_unclaimed() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());
        let user = seed_user(&pool).await;

        // Nothing bootstrapped yet.
        assert_eq!(store.claim_unclaimed_for(user).await.unwrap(), None);

        // An owned workspace is not claimable by a stranger.
        store.create_with_owner(user, "Owned").await.unwrap();
        let stranger = seed_user(&pool).await;
        assert_eq!(store.claim_unclaimed_for(stranger).await.unwrap(), None);
    }

    #[tokio::test]
    async fn only_the_first_caller_claims_the_workspace() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());
        let id = store.bootstrap_unclaimed("W", None).await.unwrap().unwrap();
        let first = seed_user(&pool).await;
        let second = seed_user(&pool).await;

        assert_eq!(store.claim_unclaimed_for(first).await.unwrap(), Some(id));
        assert_eq!(
            store.claim_unclaimed_for(second).await.unwrap(),
            None,
            "the workspace is already claimed"
        );
        assert_eq!(
            store.membership(first, id).await.unwrap(),
            Some(Role::Owner)
        );
        assert_eq!(store.membership(second, id).await.unwrap(), None);
    }

    #[tokio::test]
    async fn list_members_returns_each_member_with_their_handle() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());
        let owner = seed_user_with_email(&pool, "mara@example.com").await;
        let ws = store.create_with_owner(owner, "W").await.unwrap();
        seed_user_with_email(&pool, "dub@example.com").await;
        store
            .add_member_by_email(ws, "dub@example.com", Role::Editor)
            .await
            .unwrap();

        let members = store.list_members(ws).await.unwrap();
        assert_eq!(members.len(), 2);
        // The owner leads (oldest membership), shown by their email local-part.
        assert_eq!(
            members[0],
            Member {
                user_id: owner,
                display: "mara".into(),
                role: Role::Owner,
            }
        );
        assert!(
            members
                .iter()
                .any(|m| m.display == "dub" && m.role == Role::Editor)
        );
    }

    #[tokio::test]
    async fn set_member_role_updates_or_reports_missing() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());
        let owner = seed_user(&pool).await;
        let ws = store.create_with_owner(owner, "W").await.unwrap();
        let member = seed_user_with_email(&pool, "e@example.com").await;
        store
            .add_member_by_email(ws, "e@example.com", Role::Editor)
            .await
            .unwrap();

        assert_eq!(
            store
                .set_member_role(ws, member, Role::Viewer)
                .await
                .unwrap(),
            Some(())
        );
        assert_eq!(
            store.membership(member, ws).await.unwrap(),
            Some(Role::Viewer)
        );
        let stranger = seed_user(&pool).await;
        assert_eq!(
            store
                .set_member_role(ws, stranger, Role::Viewer)
                .await
                .unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn remove_member_deletes_or_reports_missing() {
        let (_c, pool) = fresh_db().await;
        let store = WorkspaceStore::new(pool.clone());
        let owner = seed_user(&pool).await;
        let ws = store.create_with_owner(owner, "W").await.unwrap();
        let member = seed_user_with_email(&pool, "e@example.com").await;
        store
            .add_member_by_email(ws, "e@example.com", Role::Editor)
            .await
            .unwrap();

        assert_eq!(store.remove_member(ws, member).await.unwrap(), Some(()));
        assert_eq!(store.membership(member, ws).await.unwrap(), None);
        assert_eq!(store.remove_member(ws, member).await.unwrap(), None);
    }
}
