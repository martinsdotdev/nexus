//! The Postgres-backed session store (ADR-0010), built on the pure `session_token`
//! core. It persists `SHA-256(secret)` keyed by the public session id, validates a
//! presented token in constant time, slides the expiry forward on use, and reaps an
//! expired row lazily. Cloud mode only; local file mode never constructs one.
//!
//! Queries are runtime-checked (`sqlx::query` / `query_as`, not the `query!` macro),
//! so the build needs no live database or offline cache; the integration tests verify
//! the SQL against a real ephemeral Postgres (testcontainers, Docker at test time).

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::session_token::{self, GeneratedSession};

/// Session lifetime from last use (a sliding window).
fn session_ttl() -> chrono::Duration {
    chrono::Duration::days(30)
}

/// The authenticated principal a valid session resolves to.
#[derive(Debug, PartialEq, Eq)]
pub struct AuthUser {
    pub id: Uuid,
}

/// A session's metadata for the account's "where you are signed in" list. The secret is
/// never included; only the public id and the device hints captured at sign-in.
#[derive(Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct SessionInfo {
    pub id: String,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

/// Postgres-backed session store. Cheap to clone (the pool is reference-counted).
#[derive(Clone)]
pub struct SessionStore {
    pool: PgPool,
}

impl SessionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a device-less session (a test convenience; the production login flow always
    /// has request headers, so it uses [`create_with_device`](Self::create_with_device)).
    #[cfg(test)]
    pub async fn create(&self, user_id: Uuid) -> sqlx::Result<GeneratedSession> {
        self.create_with_device(user_id, None, None).await
    }

    /// Like [`create`](Self::create) but records the client's device hints (user agent and
    /// IP) so the account's session list can show where the account is signed in.
    pub async fn create_with_device(
        &self,
        user_id: Uuid,
        user_agent: Option<&str>,
        ip: Option<&str>,
    ) -> sqlx::Result<GeneratedSession> {
        let session = session_token::generate();
        let expires_at = Utc::now() + session_ttl();
        sqlx::query(
            "insert into session (id, secret_hash, user_id, expires_at, user_agent, ip) \
             values ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&session.id)
        .bind(session.secret_hash.as_slice())
        .bind(user_id)
        .bind(expires_at)
        .bind(user_agent)
        .bind(ip)
        .execute(&self.pool)
        .await?;
        Ok(session)
    }

    /// Validate a client token: parse it, look the session up by id, verify the secret
    /// in constant time, and check expiry. On success, slide the expiry forward and
    /// return the user. A malformed, unknown, mismatched, or expired token returns
    /// `None` (an expired one is reaped).
    pub async fn validate(&self, token: &str) -> sqlx::Result<Option<AuthUser>> {
        let Some((id, secret)) = session_token::parse_token(token) else {
            return Ok(None);
        };

        let row: Option<(Vec<u8>, Uuid, DateTime<Utc>)> =
            sqlx::query_as("select secret_hash, user_id, expires_at from session where id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;

        let Some((secret_hash, user_id, expires_at)) = row else {
            return Ok(None);
        };

        if expires_at <= Utc::now() {
            self.invalidate(id).await?;
            return Ok(None);
        }

        let stored: [u8; 32] = match secret_hash.try_into() {
            Ok(hash) => hash,
            Err(_) => return Ok(None), // a corrupt hash cannot authenticate anyone
        };
        if !session_token::verify_secret(secret, &stored) {
            return Ok(None);
        }

        // Slide the window forward and stamp the last use so active sessions stay alive
        // and the account's session list shows a meaningful "last active".
        sqlx::query("update session set expires_at = $1, last_used_at = now() where id = $2")
            .bind(Utc::now() + session_ttl())
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(Some(AuthUser { id: user_id }))
    }

    /// Delete a session by id (logout, or reaping an expired one).
    pub async fn invalidate(&self, id: &str) -> sqlx::Result<()> {
        sqlx::query("delete from session where id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Every live session for `user_id`, most-recently-used first. The secret hash is never
    /// returned; only the public id + device hints, for the account's session list.
    pub async fn list_for_user(&self, user_id: Uuid) -> sqlx::Result<Vec<SessionInfo>> {
        sqlx::query_as::<_, SessionInfo>(
            "select id, user_agent, ip, created_at, last_used_at from session \
             where user_id = $1 order by last_used_at desc",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Revoke one of `user_id`'s sessions by id. Scoped to the user so a token can only end
    /// its own account's sessions. Returns `None` if no such session belongs to them (→ 404).
    pub async fn invalidate_for_user(&self, user_id: Uuid, id: &str) -> sqlx::Result<Option<()>> {
        let done = sqlx::query("delete from session where id = $1 and user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok((done.rows_affected() > 0).then_some(()))
    }

    /// Revoke all of `user_id`'s sessions except `keep_id` (sign out every other device).
    /// Returns how many sessions were revoked.
    pub async fn invalidate_others(&self, user_id: Uuid, keep_id: &str) -> sqlx::Result<u64> {
        let done = sqlx::query("delete from session where user_id = $1 and id <> $2")
            .bind(user_id)
            .bind(keep_id)
            .execute(&self.pool)
            .await?;
        Ok(done.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;

    // Spin up an ephemeral Postgres, run the migrations, and return a pool. The
    // container is returned so the caller keeps it alive for the test's duration.
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
    async fn create_then_validate_round_trips_and_invalidate_revokes() {
        let (_container, pool) = fresh_db().await;
        let store = SessionStore::new(pool.clone());
        let user_id = seed_user(&pool).await;

        let session = store.create(user_id).await.unwrap();

        // The token validates to the user.
        assert_eq!(
            store.validate(&session.token).await.unwrap(),
            Some(AuthUser { id: user_id })
        );
        // A garbage token does not.
        assert_eq!(store.validate("bogus.token").await.unwrap(), None);
        // After logout, the token no longer validates.
        store.invalidate(&session.id).await.unwrap();
        assert_eq!(store.validate(&session.token).await.unwrap(), None);
    }

    #[tokio::test]
    async fn an_expired_session_is_rejected_and_reaped() {
        let (_container, pool) = fresh_db().await;
        let store = SessionStore::new(pool.clone());
        let user_id = seed_user(&pool).await;

        let session = store.create(user_id).await.unwrap();
        sqlx::query("update session set expires_at = now() - interval '1 hour' where id = $1")
            .bind(&session.id)
            .execute(&pool)
            .await
            .unwrap();

        assert_eq!(store.validate(&session.token).await.unwrap(), None);
        let remaining: i64 = sqlx::query_scalar("select count(*) from session where id = $1")
            .bind(&session.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(remaining, 0);
    }

    #[tokio::test]
    async fn lists_sessions_with_device_and_revokes_scoped_to_the_user() {
        let (_container, pool) = fresh_db().await;
        let store = SessionStore::new(pool.clone());
        let user = seed_user(&pool).await;
        let other = seed_user(&pool).await;

        let a = store
            .create_with_device(user, Some("Firefox"), Some("1.2.3.4"))
            .await
            .unwrap();
        let b = store.create(user).await.unwrap();
        let stranger = store.create(other).await.unwrap();

        // The user sees both of their sessions (not the stranger's), with device hints.
        let list = store.list_for_user(user).await.unwrap();
        assert_eq!(list.len(), 2);
        assert!(
            list.iter()
                .any(|s| s.user_agent.as_deref() == Some("Firefox")
                    && s.ip.as_deref() == Some("1.2.3.4"))
        );

        // A user cannot revoke another user's session.
        assert_eq!(
            store.invalidate_for_user(user, &stranger.id).await.unwrap(),
            None
        );
        assert!(store.validate(&stranger.token).await.unwrap().is_some());

        // Revoking their own works; the other survives.
        assert_eq!(
            store.invalidate_for_user(user, &a.id).await.unwrap(),
            Some(())
        );
        assert!(store.validate(&a.token).await.unwrap().is_none());
        assert!(store.validate(&b.token).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn invalidate_others_keeps_only_the_current_session() {
        let (_container, pool) = fresh_db().await;
        let store = SessionStore::new(pool.clone());
        let user = seed_user(&pool).await;

        let current = store.create(user).await.unwrap();
        store.create(user).await.unwrap();
        store.create(user).await.unwrap();

        let revoked = store.invalidate_others(user, &current.id).await.unwrap();
        assert_eq!(revoked, 2);
        let list = store.list_for_user(user).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, current.id);
        assert!(store.validate(&current.token).await.unwrap().is_some());
    }
}
