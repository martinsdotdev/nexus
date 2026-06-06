//! The workspace-authorization seam (ADR-0009): one place that answers "who is the caller, and
//! may they touch this workspace?". `resolve_membership` is the single resolution used by both
//! the HTTP routes (through the extractors below) and the `/sync` gate (`ws.rs`). The three
//! extractors put the required role in a handler's type signature, so authorization is stated
//! once, at the seam, and cannot be forgotten or drift between call sites.

use std::collections::HashMap;

use axum::extract::{FromRequestParts, Path};
use axum::http::StatusCode;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::auth::routes::CurrentUser;
use crate::http::AppState;
use crate::persistence::WorkspaceId;
use crate::workspaces::store::{Role, WorkspaceStore};

/// Resolve the caller's role in a workspace, or the HTTP status to reject with (500 on a store
/// error, 403 when the caller is not a member). The shared authorization resolution.
pub async fn resolve_membership(
    workspaces: &WorkspaceStore,
    user_id: Uuid,
    workspace: WorkspaceId,
) -> Result<Role, StatusCode> {
    workspaces
        .membership(user_id, workspace)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)
}

/// Shared prelude of the role extractors: the authenticated caller, the `{id}` workspace from the
/// path (read by name, so a route with an extra `{user}` param still resolves), and the caller's
/// role in it.
async fn workspace_and_role(
    parts: &mut Parts,
    state: &AppState,
) -> Result<(WorkspaceId, Role), StatusCode> {
    let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;
    let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let id = params
        .get("id")
        .and_then(|raw| Uuid::parse_str(raw).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let workspace = WorkspaceId(id);
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let role = resolve_membership(&cloud.workspaces, user.id, workspace).await?;
    Ok((workspace, role))
}

/// Extractor: the caller is any member of the `{id}` workspace.
pub struct WorkspaceMember {
    pub workspace: WorkspaceId,
}

impl FromRequestParts<AppState> for WorkspaceMember {
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let (workspace, _role) = workspace_and_role(parts, state).await?;
        Ok(WorkspaceMember { workspace })
    }
}

/// Extractor: the caller may write to the `{id}` workspace (owner or editor; viewers get 403).
pub struct WorkspaceEditor {
    pub workspace: WorkspaceId,
}

impl FromRequestParts<AppState> for WorkspaceEditor {
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let (workspace, role) = workspace_and_role(parts, state).await?;
        if !role.can_write() {
            return Err(StatusCode::FORBIDDEN);
        }
        Ok(WorkspaceEditor { workspace })
    }
}

/// Extractor: the caller owns the `{id}` workspace (everyone else gets 403).
pub struct WorkspaceOwner {
    pub workspace: WorkspaceId,
}

impl FromRequestParts<AppState> for WorkspaceOwner {
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let (workspace, role) = workspace_and_role(parts, state).await?;
        if role != Role::Owner {
            return Err(StatusCode::FORBIDDEN);
        }
        Ok(WorkspaceOwner { workspace })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;

    #[tokio::test]
    async fn resolves_a_members_role_and_rejects_a_stranger() {
        let container = Postgres::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let store = WorkspaceStore::new(pool.clone());

        let owner = Uuid::new_v4();
        sqlx::query("insert into app_user (id) values ($1)")
            .bind(owner)
            .execute(&pool)
            .await
            .unwrap();
        let ws = store.create_with_owner(owner, "W").await.unwrap();
        assert_eq!(resolve_membership(&store, owner, ws).await, Ok(Role::Owner));

        let stranger = Uuid::new_v4();
        sqlx::query("insert into app_user (id) values ($1)")
            .bind(stranger)
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            resolve_membership(&store, stranger, ws).await,
            Err(StatusCode::FORBIDDEN)
        );
    }
}
