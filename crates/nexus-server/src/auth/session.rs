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

/// Postgres-backed session store. Cheap to clone (the pool is reference-counted).
#[derive(Clone)]
pub struct SessionStore {
    pool: PgPool,
}

impl SessionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a session for `user_id` and return the token to set as the cookie. The
    /// store keeps only the session id and the hashed secret, never the raw secret.
    pub async fn create(&self, user_id: Uuid) -> sqlx::Result<GeneratedSession> {
        let session = session_token::generate();
        let expires_at = Utc::now() + session_ttl();
        sqlx::query(
            "insert into session (id, secret_hash, user_id, expires_at) \
             values ($1, $2, $3, $4)",
        )
        .bind(&session.id)
        .bind(session.secret_hash.as_slice())
        .bind(user_id)
        .bind(expires_at)
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

        // Slide the window forward so active sessions stay alive.
        sqlx::query("update session set expires_at = $1 where id = $2")
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
}
