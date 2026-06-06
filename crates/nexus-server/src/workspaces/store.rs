//! The Postgres-backed workspace + membership store (ADR-0009): create a workspace
//! owned by a user, look up a user's role in a workspace, and list the workspaces a
//! user belongs to. Cloud mode only; runtime-checked sqlx (no offline cache), like
//! the auth stores. The `/sync` gate and the workspace HTTP routes consume this in
//! later increments.

use sqlx::PgPool;
use uuid::Uuid;

use crate::persistence::WorkspaceId;

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
}
