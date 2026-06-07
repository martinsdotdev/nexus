//! Per-workspace, revocable, read-only overlay tokens (ADR-0009 #6). An OBS browser
//! source carries `?token=<id>.<secret>` and the relay grants a read-only `/sync`
//! connection to that workspace. Reuses the session-token primitives (a token is an
//! id + secret; only `SHA-256(secret)` is stored), so a database leak cannot forge one.
//! Cloud mode only.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::session_token;
use crate::persistence::WorkspaceId;

/// An `overlay_token` row as fetched for validation: hash, workspace, revocation time.
type TokenRow = (Vec<u8>, Uuid, Option<DateTime<Utc>>);

/// An overlay token's public metadata for the management list (never the secret, which is
/// shown once at mint). The id is the public half of the `id.secret` token.
#[derive(Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct TokenInfo {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Postgres-backed overlay-token store. Cheap to clone (the pool is reference-counted).
#[derive(Clone)]
pub struct OverlayTokenStore {
    pool: PgPool,
}

impl OverlayTokenStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Mint a read-only token for `workspace`. The raw token is returned once (for the
    /// OBS URL); only its hash is persisted.
    pub async fn mint(&self, workspace: WorkspaceId) -> sqlx::Result<String> {
        let token = session_token::generate();
        sqlx::query(
            "insert into overlay_token (id, secret_hash, workspace_id) values ($1, $2, $3)",
        )
        .bind(&token.id)
        .bind(token.secret_hash.as_slice())
        .bind(workspace.0)
        .execute(&self.pool)
        .await?;
        Ok(token.token)
    }

    /// Validate a token, returning the workspace it grants read-only access to. A
    /// malformed, unknown, revoked, or mismatched token returns `None`.
    pub async fn validate(&self, token: &str) -> sqlx::Result<Option<WorkspaceId>> {
        let Some((id, secret)) = session_token::parse_token(token) else {
            return Ok(None);
        };
        let row: Option<TokenRow> = sqlx::query_as(
            "select secret_hash, workspace_id, revoked_at from overlay_token where id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        let Some((secret_hash, workspace_id, revoked_at)) = row else {
            return Ok(None);
        };
        if revoked_at.is_some() {
            return Ok(None);
        }
        let stored: [u8; 32] = match secret_hash.try_into() {
            Ok(hash) => hash,
            Err(_) => return Ok(None), // a corrupt hash cannot authenticate anything
        };
        if !session_token::verify_secret(secret, &stored) {
            return Ok(None);
        }
        Ok(Some(WorkspaceId(workspace_id)))
    }

    /// Every overlay token minted for `workspace`, newest first, for the management list.
    /// Only ids + status; the secret is shown once at mint and never stored in clear.
    pub async fn list(&self, workspace: WorkspaceId) -> sqlx::Result<Vec<TokenInfo>> {
        sqlx::query_as::<_, TokenInfo>(
            "select id, created_at, revoked_at from overlay_token \
             where workspace_id = $1 order by created_at desc",
        )
        .bind(workspace.0)
        .fetch_all(&self.pool)
        .await
    }

    /// Revoke a token by id, scoped to its workspace so an editor cannot revoke another
    /// workspace's token. Idempotent; returns `None` if no such token belongs to the
    /// workspace (→ 404).
    pub async fn revoke(&self, workspace: WorkspaceId, id: &str) -> sqlx::Result<Option<()>> {
        let done = sqlx::query(
            "update overlay_token set revoked_at = now() where id = $1 and workspace_id = $2",
        )
        .bind(id)
        .bind(workspace.0)
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

    async fn fresh_workspace() -> (ContainerAsync<Postgres>, PgPool, WorkspaceId) {
        let container = Postgres::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let workspace = WorkspaceId(Uuid::new_v4());
        sqlx::query("insert into workspace (id) values ($1)")
            .bind(workspace.0)
            .execute(&pool)
            .await
            .unwrap();
        (container, pool, workspace)
    }

    #[tokio::test]
    async fn mint_then_validate_grants_the_workspace() {
        let (_c, pool, workspace) = fresh_workspace().await;
        let store = OverlayTokenStore::new(pool);

        let token = store.mint(workspace).await.unwrap();

        assert_eq!(store.validate(&token).await.unwrap(), Some(workspace));
        assert_eq!(store.validate("bogus.token").await.unwrap(), None);
    }

    #[tokio::test]
    async fn a_revoked_token_is_rejected() {
        let (_c, pool, workspace) = fresh_workspace().await;
        let store = OverlayTokenStore::new(pool);

        let token = store.mint(workspace).await.unwrap();
        let (id, _) = session_token::parse_token(&token).unwrap();
        assert_eq!(store.revoke(workspace, id).await.unwrap(), Some(()));

        assert_eq!(store.validate(&token).await.unwrap(), None);
    }

    #[tokio::test]
    async fn lists_tokens_and_revocation_is_workspace_scoped() {
        let (_c, pool, workspace) = fresh_workspace().await;
        let store = OverlayTokenStore::new(pool.clone());
        // A second workspace, to prove revoke cannot reach across workspaces.
        let other = WorkspaceId(Uuid::new_v4());
        sqlx::query("insert into workspace (id) values ($1)")
            .bind(other.0)
            .execute(&pool)
            .await
            .unwrap();

        let token = store.mint(workspace).await.unwrap();
        let (id, _) = session_token::parse_token(&token).unwrap();

        let list = store.list(workspace).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert!(list[0].revoked_at.is_none());

        // Revoking under the wrong workspace finds nothing; the token still validates.
        assert_eq!(store.revoke(other, id).await.unwrap(), None);
        assert!(store.validate(&token).await.unwrap().is_some());

        // Revoking under the right workspace works.
        assert_eq!(store.revoke(workspace, id).await.unwrap(), Some(()));
        assert!(store.validate(&token).await.unwrap().is_none());
    }
}
