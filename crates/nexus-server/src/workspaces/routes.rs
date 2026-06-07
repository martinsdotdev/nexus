//! Cloud-mode workspace management routes (ADR-0009): list the caller's workspaces, create
//! one (the caller becomes owner), list members, invite/remove members and change roles
//! (owner only), and mint a read-only overlay watch link (owner or editor). Mounted by
//! `http::build_app` in cloud mode, behind the same CSRF guard as the auth routes.

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::response::Json;
use axum::routing::{delete, get, post, put};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::routes::{CurrentUser, csrf_guard};
use crate::http::AppState;
use crate::workspaces::access::{WorkspaceEditor, WorkspaceMember, WorkspaceOwner};
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
struct RenameWorkspace {
    name: String,
}

/// `PUT /workspaces/:id`: rename a workspace (owner only).
async fn rename_workspace(
    State(state): State<AppState>,
    WorkspaceOwner { workspace }: WorkspaceOwner,
    Json(body): Json<RenameWorkspace>,
) -> Result<StatusCode, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let name = body.name.trim();
    if name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    cloud
        .workspaces
        .rename(workspace, name)
        .await
        .map_err(internal)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /workspaces/:id`: delete a workspace and evict its live runtime (owner only).
/// The Postgres delete cascades the membership + overlay-token rows and drops the snapshot
/// column; eviction frees the in-memory replica.
async fn delete_workspace(
    State(state): State<AppState>,
    WorkspaceOwner { workspace }: WorkspaceOwner,
) -> Result<StatusCode, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    cloud
        .workspaces
        .delete(workspace)
        .await
        .map_err(internal)?
        .ok_or(StatusCode::NOT_FOUND)?;
    state.registry.evict(workspace).await;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct InviteMember {
    email: String,
    role: Option<Role>,
}

/// `POST /workspaces/:id/members`: invite an existing account by email (owner only).
async fn invite_member(
    State(state): State<AppState>,
    WorkspaceOwner { workspace }: WorkspaceOwner,
    Json(body): Json<InviteMember>,
) -> Result<StatusCode, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
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

#[derive(Serialize)]
struct MemberItem {
    user_id: String,
    display: String,
    role: Role,
}

/// `GET /workspaces/:id/members`: everyone in the workspace (any member may read).
async fn list_members(
    State(state): State<AppState>,
    WorkspaceMember { workspace }: WorkspaceMember,
) -> Result<Json<Vec<MemberItem>>, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let members = cloud
        .workspaces
        .list_members(workspace)
        .await
        .map_err(internal)?;
    Ok(Json(
        members
            .into_iter()
            .map(|m| MemberItem {
                user_id: m.user_id.to_string(),
                display: m.display,
                role: m.role,
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
struct SetRole {
    role: Role,
}

/// `PUT /workspaces/:id/members/:user`: change a member's role (owner only). The owner's
/// own seat is fixed, and no one is promoted to owner through this route.
async fn set_member_role(
    State(state): State<AppState>,
    WorkspaceOwner { workspace }: WorkspaceOwner,
    Path((_, target)): Path<(Uuid, Uuid)>,
    Json(body): Json<SetRole>,
) -> Result<StatusCode, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    if body.role == Role::Owner {
        return Err(StatusCode::BAD_REQUEST);
    }
    let current = cloud
        .workspaces
        .membership(target, workspace)
        .await
        .map_err(internal)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if current == Role::Owner {
        return Err(StatusCode::FORBIDDEN); // the owner's role is fixed
    }
    cloud
        .workspaces
        .set_member_role(workspace, target, body.role)
        .await
        .map_err(internal)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /workspaces/:id/members/:user`: remove a member (owner only; the owner cannot
/// be removed).
async fn remove_member(
    State(state): State<AppState>,
    WorkspaceOwner { workspace }: WorkspaceOwner,
    Path((_, target)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let current = cloud
        .workspaces
        .membership(target, workspace)
        .await
        .map_err(internal)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if current == Role::Owner {
        return Err(StatusCode::FORBIDDEN);
    }
    cloud
        .workspaces
        .remove_member(workspace, target)
        .await
        .map_err(internal)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
struct MintedToken {
    token: String,
}

/// `POST /workspaces/:id/overlay-token`: mint a read-only watch link (owner or editor).
/// Viewers cannot create one.
async fn mint_overlay_token(
    State(state): State<AppState>,
    WorkspaceEditor { workspace }: WorkspaceEditor,
) -> Result<Json<MintedToken>, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let token = cloud
        .overlay_tokens
        .mint(workspace)
        .await
        .map_err(internal)?;
    Ok(Json(MintedToken { token }))
}

#[derive(Serialize)]
struct OverlayTokenItem {
    id: String,
    created_at: String,
    revoked: bool,
}

/// `GET /workspaces/:id/overlay-tokens`: the workspace's minted watch links (owner or
/// editor). Only ids + status; the secret is shown once at mint and never again.
async fn list_overlay_tokens(
    State(state): State<AppState>,
    WorkspaceEditor { workspace }: WorkspaceEditor,
) -> Result<Json<Vec<OverlayTokenItem>>, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let tokens = cloud
        .overlay_tokens
        .list(workspace)
        .await
        .map_err(internal)?;
    Ok(Json(
        tokens
            .into_iter()
            .map(|t| OverlayTokenItem {
                id: t.id,
                created_at: t.created_at.to_rfc3339(),
                revoked: t.revoked_at.is_some(),
            })
            .collect(),
    ))
}

/// `DELETE /workspaces/:id/overlay-token/:token`: revoke a watch link (owner or editor).
async fn revoke_overlay_token(
    State(state): State<AppState>,
    WorkspaceEditor { workspace }: WorkspaceEditor,
    Path((_, token)): Path<(Uuid, String)>,
) -> Result<StatusCode, StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    cloud
        .overlay_tokens
        .revoke(workspace, &token)
        .await
        .map_err(internal)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(StatusCode::NO_CONTENT)
}

fn internal<E: std::fmt::Display>(err: E) -> StatusCode {
    tracing::error!("workspace route error: {err}");
    StatusCode::INTERNAL_SERVER_ERROR
}

/// The cloud-mode workspace routes, with the CSRF guard layered on.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/workspaces",
            get(list_workspaces).post(create_workspace),
        )
        .route(
            "/api/workspaces/{id}",
            put(rename_workspace).delete(delete_workspace),
        )
        .route(
            "/api/workspaces/{id}/members",
            get(list_members).post(invite_member),
        )
        .route(
            "/api/workspaces/{id}/members/{user}",
            put(set_member_role).delete(remove_member),
        )
        .route(
            "/api/workspaces/{id}/overlay-tokens",
            get(list_overlay_tokens),
        )
        .route(
            "/api/workspaces/{id}/overlay-token",
            post(mint_overlay_token),
        )
        .route(
            "/api/workspaces/{id}/overlay-token/{token}",
            delete(revoke_overlay_token),
        )
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

    async fn seed_user(pool: &PgPool) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("insert into app_user (id) values ($1)")
            .bind(id)
            .execute(pool)
            .await
            .unwrap();
        id
    }

    async fn seed_user_with_email(pool: &PgPool, email: &str) -> Uuid {
        let id = seed_user(pool).await;
        sqlx::query("insert into email_identity (email, user_id) values ($1, $2)")
            .bind(email)
            .bind(id)
            .execute(pool)
            .await
            .unwrap();
        id
    }

    fn post(uri: &str, cookie: &str, body: &str) -> Request<Body> {
        req("POST", uri, cookie, body)
    }

    fn req(method: &str, uri: &str, cookie: &str, body: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .header("sec-fetch-site", "same-origin")
            .header("cookie", format!("nexus_session={cookie}"))
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn get_req(uri: &str, cookie: &str) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .header("cookie", format!("nexus_session={cookie}"))
            .body(Body::empty())
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
            .oneshot(post("/api/workspaces", &owner_cookie, r#"{"name":"Team"}"#))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ws_id = json(resp).await["id"].as_str().unwrap().to_string();

        // It appears in the owner's list, with the owner role.
        let resp = build_app(state.clone(), None)
            .oneshot(
                Request::builder()
                    .uri("/api/workspaces")
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
                &format!("/api/workspaces/{ws_id}/members"),
                &owner_cookie,
                r#"{"email":"editor@example.com"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        // Inviting an unknown email is a 404.
        let resp = build_app(state.clone(), None)
            .oneshot(post(
                &format!("/api/workspaces/{ws_id}/members"),
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
                &format!("/api/workspaces/{}/members", ws.0),
                &editor_cookie,
                r#"{"email":"o@example.com"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn lists_members_and_changes_a_role() {
        let (_c, _tmp, state, pool) = cloud().await;
        let cloud = state.cloud.clone().unwrap();
        let owner = seed_user_with_email(&pool, "owner@example.com").await;
        let owner_cookie = cloud.sessions.create(owner).await.unwrap().token;
        let ws = cloud
            .workspaces
            .create_with_owner(owner, "Team")
            .await
            .unwrap();
        let editor = seed_user_with_email(&pool, "ed@example.com").await;
        cloud
            .workspaces
            .add_member_by_email(ws, "ed@example.com", Role::Editor)
            .await
            .unwrap();

        // The owner lists members (both rows).
        let resp = build_app(state.clone(), None)
            .oneshot(get_req(
                &format!("/api/workspaces/{}/members", ws.0),
                &owner_cookie,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(json(resp).await.as_array().unwrap().len(), 2);

        // The owner demotes the editor to a viewer.
        let resp = build_app(state.clone(), None)
            .oneshot(req(
                "PUT",
                &format!("/api/workspaces/{}/members/{}", ws.0, editor),
                &owner_cookie,
                r#"{"role":"viewer"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            cloud.workspaces.membership(editor, ws).await.unwrap(),
            Some(Role::Viewer)
        );
    }

    #[tokio::test]
    async fn a_non_owner_cannot_change_roles() {
        let (_c, _tmp, state, pool) = cloud().await;
        let cloud = state.cloud.clone().unwrap();
        let owner = seed_user(&pool).await;
        let ws = cloud
            .workspaces
            .create_with_owner(owner, "W")
            .await
            .unwrap();
        let editor = seed_user_with_email(&pool, "ed@example.com").await;
        cloud
            .workspaces
            .add_member_by_email(ws, "ed@example.com", Role::Editor)
            .await
            .unwrap();
        let editor_cookie = cloud.sessions.create(editor).await.unwrap().token;

        let resp = build_app(state.clone(), None)
            .oneshot(req(
                "PUT",
                &format!("/api/workspaces/{}/members/{}", ws.0, owner),
                &editor_cookie,
                r#"{"role":"viewer"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn owner_mints_a_watch_token_but_a_viewer_cannot() {
        let (_c, _tmp, state, pool) = cloud().await;
        let cloud = state.cloud.clone().unwrap();
        let owner = seed_user(&pool).await;
        let ws = cloud
            .workspaces
            .create_with_owner(owner, "W")
            .await
            .unwrap();
        let owner_cookie = cloud.sessions.create(owner).await.unwrap().token;

        let resp = build_app(state.clone(), None)
            .oneshot(req(
                "POST",
                &format!("/api/workspaces/{}/overlay-token", ws.0),
                &owner_cookie,
                "",
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let token = json(resp).await["token"].as_str().unwrap().to_string();
        assert_eq!(
            cloud.overlay_tokens.validate(&token).await.unwrap(),
            Some(ws)
        );

        let viewer = seed_user_with_email(&pool, "v@example.com").await;
        cloud
            .workspaces
            .add_member_by_email(ws, "v@example.com", Role::Viewer)
            .await
            .unwrap();
        let viewer_cookie = cloud.sessions.create(viewer).await.unwrap().token;
        let resp = build_app(state.clone(), None)
            .oneshot(req(
                "POST",
                &format!("/api/workspaces/{}/overlay-token", ws.0),
                &viewer_cookie,
                "",
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn the_owner_renames_then_deletes_a_workspace() {
        let (_c, _tmp, state, pool) = cloud().await;
        let cloud = state.cloud.clone().unwrap();
        let owner = seed_user(&pool).await;
        let owner_cookie = cloud.sessions.create(owner).await.unwrap().token;
        let ws = cloud
            .workspaces
            .create_with_owner(owner, "Old")
            .await
            .unwrap();

        let resp = build_app(state.clone(), None)
            .oneshot(req(
                "PUT",
                &format!("/api/workspaces/{}", ws.0),
                &owner_cookie,
                r#"{"name":"New"}"#,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        let name: String = sqlx::query_scalar("select name from workspace where id = $1")
            .bind(ws.0)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(name, "New");

        let resp = build_app(state.clone(), None)
            .oneshot(req(
                "DELETE",
                &format!("/api/workspaces/{}", ws.0),
                &owner_cookie,
                "",
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        let count: i64 = sqlx::query_scalar("select count(*) from workspace where id = $1")
            .bind(ws.0)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "the workspace row is gone");
    }

    #[tokio::test]
    async fn a_non_owner_cannot_delete_a_workspace() {
        let (_c, _tmp, state, pool) = cloud().await;
        let cloud = state.cloud.clone().unwrap();
        let owner = seed_user(&pool).await;
        let ws = cloud
            .workspaces
            .create_with_owner(owner, "W")
            .await
            .unwrap();
        let editor = seed_user_with_email(&pool, "e@example.com").await;
        cloud
            .workspaces
            .add_member_by_email(ws, "e@example.com", Role::Editor)
            .await
            .unwrap();
        let editor_cookie = cloud.sessions.create(editor).await.unwrap().token;

        let resp = build_app(state.clone(), None)
            .oneshot(req(
                "DELETE",
                &format!("/api/workspaces/{}", ws.0),
                &editor_cookie,
                "",
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn an_owner_lists_and_revokes_overlay_tokens() {
        let (_c, _tmp, state, pool) = cloud().await;
        let cloud = state.cloud.clone().unwrap();
        let owner = seed_user(&pool).await;
        let owner_cookie = cloud.sessions.create(owner).await.unwrap().token;
        let ws = cloud
            .workspaces
            .create_with_owner(owner, "W")
            .await
            .unwrap();
        let token = cloud.overlay_tokens.mint(ws).await.unwrap();
        let token_id = cloud.overlay_tokens.list(ws).await.unwrap()[0].id.clone();

        // GET lists the one active token.
        let resp = build_app(state.clone(), None)
            .oneshot(get_req(
                &format!("/api/workspaces/{}/overlay-tokens", ws.0),
                &owner_cookie,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let list = json(resp).await;
        assert_eq!(list.as_array().unwrap().len(), 1);
        assert_eq!(list[0]["revoked"], false);

        // DELETE revokes it (same-origin); the token then fails validation.
        let resp = build_app(state.clone(), None)
            .oneshot(req(
                "DELETE",
                &format!("/api/workspaces/{}/overlay-token/{}", ws.0, token_id),
                &owner_cookie,
                "",
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert_eq!(cloud.overlay_tokens.validate(&token).await.unwrap(), None);
    }
}
