//! Cloud-mode workspace management routes (ADR-0009): list the caller's workspaces,
//! create one (the caller becomes owner), and invite a member by email (owner only).
//! Mounted by `http::build_app` in cloud mode, behind the same CSRF guard as the auth
//! routes. Overlay-token management routes are a later refinement.

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::response::Json;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::routes::{CurrentUser, csrf_guard};
use crate::http::AppState;
use crate::persistence::WorkspaceId;
use crate::workspaces::store::Role;

#[derive(Serialize)]
struct WorkspaceItem {
    id: String,
    name: String,
    role: Role,
}

/// `GET /workspaces`: the caller's workspaces, each with their role.
async fn list_workspaces(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<Vec<WorkspaceItem>>, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let list = cloud
        .workspaces
        .list_for_user(user.id)
        .await
        .map_err(internal)?;
    Ok(Json(
        list.into_iter()
            .map(|w| WorkspaceItem {
                id: w.id.0.to_string(),
                name: w.name,
                role: w.role,
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
struct CreateWorkspace {
    name: Option<String>,
}

#[derive(Serialize)]
struct CreatedWorkspace {
    id: String,
}

/// `POST /workspaces`: create a workspace owned by the caller.
async fn create_workspace(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<CreateWorkspace>,
) -> Result<Json<CreatedWorkspace>, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let raw = body.name.unwrap_or_default();
    let name = if raw.trim().is_empty() {
        "Untitled"
    } else {
        raw.trim()
    };
    let id = cloud
        .workspaces
        .create_with_owner(user.id, name)
        .await
        .map_err(internal)?;
    Ok(Json(CreatedWorkspace {
        id: id.0.to_string(),
    }))
}

#[derive(Deserialize)]
struct InviteMember {
    email: String,
    role: Option<Role>,
}

/// `POST /workspaces/:id/members`: invite an existing account by email (owner only).
async fn invite_member(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(workspace_id): Path<Uuid>,
    Json(body): Json<InviteMember>,
) -> Result<StatusCode, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let workspace = WorkspaceId(workspace_id);

    // Only an owner may invite.
    let role = cloud
        .workspaces
        .membership(user.id, workspace)
        .await
        .map_err(internal)?
        .ok_or(StatusCode::FORBIDDEN)?;
    if role != Role::Owner {
        return Err(StatusCode::FORBIDDEN);
    }

    let email = body.email.trim().to_lowercase();
    let invited_role = body.role.unwrap_or(Role::Editor);
    cloud
        .workspaces
        .add_member_by_email(workspace, &email, invited_role)
        .await
        .map_err(internal)?
        .ok_or(StatusCode::NOT_FOUND)?; // no account with that email
    Ok(StatusCode::NO_CONTENT)
}

fn internal<E: std::fmt::Display>(err: E) -> StatusCode {
    tracing::error!("workspace route error: {err}");
    StatusCode::INTERNAL_SERVER_ERROR
}

/// The cloud-mode workspace routes, with the CSRF guard layered on.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/workspaces", get(list_workspaces).post(create_workspace))
        .route("/workspaces/{id}/members", post(invite_member))
        .layer(middleware::from_fn(csrf_guard))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::email::EmailStore;
    use crate::auth::email_sender::EmailSender;
    use crate::auth::session::SessionStore;
    use crate::http::{CloudAuth, build_app};
    use crate::persistence::FilePersistence;
    use crate::registry::WorkspaceRegistry;
    use crate::workspaces::overlay_token::OverlayTokenStore;
    use crate::workspaces::store::WorkspaceStore;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use sqlx::PgPool;
    use std::sync::Arc;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use tower::ServiceExt;

    async fn cloud() -> (
        ContainerAsync<Postgres>,
        tempfile::TempDir,
        AppState,
        PgPool,
    ) {
        let container = Postgres::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let registry = WorkspaceRegistry::new(Arc::new(FilePersistence::new(tmp.path())));
        let state = AppState {
            registry,
            cloud: Some(CloudAuth {
                sessions: SessionStore::new(pool.clone()),
                emails: EmailStore::new(pool.clone()),
                sender: EmailSender::Log,
                workspaces: WorkspaceStore::new(pool.clone()),
                overlay_tokens: OverlayTokenStore::new(pool.clone()),
            }),
        };
        (container, tmp, state, pool)
    }

    async fn seed_user_with_email(pool: &PgPool, email: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("insert into app_user (id) values ($1)")
            .bind(id)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("insert into email_identity (email, user_id) values ($1, $2)")
            .bind(email)
            .bind(id)
            .execute(pool)
            .await
            .unwrap();
        id
    }

    fn post(uri: &str, cookie: &str, body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .header("sec-fetch-site", "same-origin")
            .header("cookie", format!("nexus_session={cookie}"))
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    async fn json(resp: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn create_list_and_invite_flow() {
        let (_c, _tmp, state, pool) = cloud().await;
        let cloud = state.cloud.clone().unwrap();
        let owner = seed_user_with_email(&pool, "owner@example.com").await;
        let owner_cookie = cloud.sessions.create(owner).await.unwrap().token;

        // Create a workspace.
        let resp = build_app(state.clone(), None)
            .oneshot(post("/workspaces", &owner_cookie, r#"{"name":"Team"}"#))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ws_id = json(resp).await["id"].as_str().unwrap().to_string();

        // It appears in the owner's list, with the owner role.
        let resp = build_app(state.clone(), None)
            .oneshot(
                Request::builder()
                    .uri("/workspaces")
                    .header("cookie", format!("nexus_session={owner_cookie}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let list = json(resp).await;
        assert_eq!(list[0]["id"], ws_id);
        assert_eq!(list[0]["role"], "owner");

        // The owner invites an existing account.
        seed_user_with_email(&pool, "editor@example.com").await;
        let resp = build_app(state.clone(), None)
            .oneshot(post(
                &format!("/workspaces/{ws_id}/members"),
                &owner_cookie,
                r#"{"email":"editor@example.com"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        // Inviting an unknown email is a 404.
        let resp = build_app(state.clone(), None)
            .oneshot(post(
                &format!("/workspaces/{ws_id}/members"),
                &owner_cookie,
                r#"{"email":"ghost@example.com"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn a_non_owner_cannot_invite() {
        let (_c, _tmp, state, pool) = cloud().await;
        let cloud = state.cloud.clone().unwrap();
        let owner = seed_user_with_email(&pool, "o@example.com").await;
        let ws = cloud
            .workspaces
            .create_with_owner(owner, "W")
            .await
            .unwrap();
        // An editor member, not an owner.
        let editor = seed_user_with_email(&pool, "e@example.com").await;
        cloud
            .workspaces
            .add_member_by_email(ws, "e@example.com", Role::Editor)
            .await
            .unwrap();
        let editor_cookie = cloud.sessions.create(editor).await.unwrap().token;

        let resp = build_app(state.clone(), None)
            .oneshot(post(
                &format!("/workspaces/{}/members", ws.0),
                &editor_cookie,
                r#"{"email":"o@example.com"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
