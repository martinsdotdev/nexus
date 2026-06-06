//! The `/sync` WebSocket. In cloud mode the upgrade is gated: the same-origin session
//! cookie must validate and the user must be a member of the requested workspace
//! (`?workspace=<uuid>`); non-members are rejected and viewers are read-only. Local mode
//! serves the single workspace, writable, with no auth (ADR-0005). After the upgrade:
//! acquire the workspace from the registry, snapshot-on-connect, then relay Loro update
//! frames (merged through the runtime) and opaque presence frames (forwarded to other
//! peers, never merged or persisted).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

use crate::auth::routes::COOKIE_NAME;
use crate::http::AppState;
use crate::persistence::WorkspaceId;
use crate::protocol::Frame;
use crate::registry::WorkspaceRegistry;
use crate::workspaces::access::resolve_membership;

/// Process-wide unique id per connection, so a session can skip echoing its own
/// presence back to itself.
static NEXT_CONN: AtomicU64 = AtomicU64::new(1);

/// Query parameters on the `/sync` upgrade. In cloud mode, `workspace` (a UUID string)
/// selects the workspace for a cookie-authenticated editor, or `token` carries an
/// overlay read-only token (the OBS path). Both are ignored in local mode.
#[derive(Debug, Deserialize)]
pub struct SyncParams {
    workspace: Option<String>,
    token: Option<String>,
}

/// What a connection is allowed to do, resolved from auth + membership before the
/// upgrade. `can_write` is false for viewers (read-only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Access {
    workspace: WorkspaceId,
    can_write: bool,
}

pub async fn sync_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<SyncParams>,
    jar: CookieJar,
) -> Response {
    match resolve_access(&state, &params, &jar).await {
        Ok(access) => ws.on_upgrade(move |socket| handle_socket(socket, state.registry, access)),
        Err(status) => status.into_response(),
    }
}

/// Resolve a connection's access. Local mode (no cloud): the single workspace, writable,
/// no auth (ADR-0005). Cloud mode: a workspace id is required, the session cookie must
/// validate, and the user must be a member; viewers are read-only. On failure, returns
/// the HTTP status to reject the upgrade with.
async fn resolve_access(
    state: &AppState,
    params: &SyncParams,
    jar: &CookieJar,
) -> Result<Access, StatusCode> {
    let Some(cloud) = state.cloud.as_ref() else {
        return Ok(Access {
            workspace: WorkspaceId::LOCAL,
            can_write: true,
        });
    };

    // Overlay read-only token path (OBS): no cookie, anonymous, read-only. The token
    // itself names the workspace.
    if let Some(token) = params.token.as_deref() {
        let workspace = cloud
            .overlay_tokens
            .validate(token)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED)?;
        return Ok(Access {
            workspace,
            can_write: false,
        });
    }

    let raw = params.workspace.as_deref().ok_or(StatusCode::BAD_REQUEST)?;
    let workspace = WorkspaceId(Uuid::parse_str(raw).map_err(|_| StatusCode::BAD_REQUEST)?);

    // The same-origin session cookie rides the upgrade. `SameSite=Lax` keeps it off
    // cross-site WebSocket handshakes, so a missing cookie also blocks cross-site joins.
    let cookie = jar.get(COOKIE_NAME).ok_or(StatusCode::UNAUTHORIZED)?;
    let user = match cloud.sessions.validate(cookie.value()).await {
        Ok(Some(user)) => user,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let role = resolve_membership(&cloud.workspaces, user.id, workspace).await?;

    Ok(Access {
        workspace,
        can_write: role.can_write(),
    })
}

async fn handle_socket(socket: WebSocket, registry: Arc<WorkspaceRegistry>, access: Access) {
    let runtime = match registry.acquire(access.workspace).await {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::warn!(%error, "failed to load workspace for /sync");
            return;
        }
    };
    let conn_id = NEXT_CONN.fetch_add(1, Ordering::Relaxed);
    let (mut sender, mut receiver) = socket.split();

    // Snapshot-on-connect: the new peer imports the canonical state, then both
    // sides exchange incremental updates.
    let snapshot = Frame::Snapshot(runtime.snapshot()).encode();
    if sender.send(Message::Binary(snapshot.into())).await.is_err() {
        registry.release(access.workspace).await;
        return;
    }

    // Forward rebroadcast deltas and other peers' presence out to this socket.
    let mut deltas = runtime.subscribe();
    let mut presence = runtime.subscribe_presence();
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                delta = deltas.recv() => {
                    // A closed or lagged delta stream tears the socket down so the
                    // client reconnects and resyncs from a fresh snapshot; a missed
                    // incremental delta would otherwise leave the doc desynced.
                    let Ok(delta) = delta else { break };
                    let frame = Frame::Update(delta).encode();
                    if sender.send(Message::Binary(frame.into())).await.is_err() {
                        break;
                    }
                }
                msg = presence.recv() => match msg {
                    // Skip our own presence; forward everyone else's.
                    Ok((from, _)) if from == conn_id => {}
                    Ok((_, bytes)) => {
                        let frame = Frame::Presence(bytes).encode();
                        if sender.send(Message::Binary(frame.into())).await.is_err() {
                            break;
                        }
                    }
                    // Lagging presence is harmless (last-write-wins); only a closed
                    // channel ends the loop.
                    Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => break,
                },
            }
        }
    });

    // Route incoming frames. Read-only connections (viewers) cannot mutate the document;
    // presence is ephemeral, so every present peer may share its cursor.
    let recv_runtime = Arc::clone(&runtime);
    let can_write = access.can_write;
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Binary(bytes) = message {
                match Frame::decode(bytes.as_ref()) {
                    Some(Frame::Update(update)) if can_write => {
                        let _ = recv_runtime.apply_remote(&update).await;
                    }
                    Some(Frame::Update(_)) => {}
                    Some(Frame::Presence(presence)) => {
                        recv_runtime.forward_presence(conn_id, presence)
                    }
                    _ => {}
                }
            }
        }
    });

    // When either direction closes, tear down the other.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Wait for both tasks to fully unwind (dropping their broadcast receivers) before
    // releasing, so the registry sees an accurate subscriber count and can evict.
    let _ = send_task.await;
    let _ = recv_task.await;
    registry.release(access.workspace).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::email::EmailStore;
    use crate::auth::email_sender::EmailSender;
    use crate::auth::session::SessionStore;
    use crate::http::CloudAuth;
    use crate::persistence::FilePersistence;
    use crate::workspaces::overlay_token::OverlayTokenStore;
    use crate::workspaces::store::WorkspaceStore;
    use axum_extra::extract::cookie::Cookie;
    use sqlx::PgPool;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;

    // A cloud AppState backed by an ephemeral Postgres. The registry is never exercised
    // by `resolve_access` (which only reads `cloud`), so its file persistence is inert.
    async fn cloud_state() -> (
        ContainerAsync<Postgres>,
        tempfile::TempDir,
        PgPool,
        AppState,
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
        (container, tmp, pool, state)
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

    fn jar_with(token: &str) -> CookieJar {
        CookieJar::new().add(Cookie::new(COOKIE_NAME, token.to_string()))
    }

    fn params_for(workspace: WorkspaceId) -> SyncParams {
        SyncParams {
            workspace: Some(workspace.0.to_string()),
            token: None,
        }
    }

    #[tokio::test]
    async fn local_mode_grants_writable_access_to_the_single_workspace() {
        let tmp = tempfile::tempdir().unwrap();
        let registry = WorkspaceRegistry::new(Arc::new(FilePersistence::new(tmp.path())));
        let state = AppState {
            registry,
            cloud: None,
        };

        let access = resolve_access(
            &state,
            &SyncParams {
                workspace: None,
                token: None,
            },
            &CookieJar::new(),
        )
        .await
        .unwrap();

        assert_eq!(access.workspace, WorkspaceId::LOCAL);
        assert!(access.can_write);
    }

    #[tokio::test]
    async fn a_member_resolves_to_their_workspace() {
        let (_c, _tmp, pool, state) = cloud_state().await;
        let cloud = state.cloud.as_ref().unwrap();
        let user = seed_user(&pool).await;
        let ws = cloud.workspaces.create_with_owner(user, "W").await.unwrap();
        let session = cloud.sessions.create(user).await.unwrap();

        let access = resolve_access(&state, &params_for(ws), &jar_with(&session.token))
            .await
            .unwrap();

        assert_eq!(access.workspace, ws);
        assert!(access.can_write, "an owner can write");
    }

    #[tokio::test]
    async fn a_viewer_resolves_to_read_only() {
        let (_c, _tmp, pool, state) = cloud_state().await;
        let cloud = state.cloud.as_ref().unwrap();
        let owner = seed_user(&pool).await;
        let viewer = seed_user(&pool).await;
        let ws = cloud
            .workspaces
            .create_with_owner(owner, "W")
            .await
            .unwrap();
        sqlx::query(
            "insert into membership (workspace_id, user_id, role) values ($1, $2, 'viewer')",
        )
        .bind(ws.0)
        .bind(viewer)
        .execute(&pool)
        .await
        .unwrap();
        let session = cloud.sessions.create(viewer).await.unwrap();

        let access = resolve_access(&state, &params_for(ws), &jar_with(&session.token))
            .await
            .unwrap();

        assert!(!access.can_write, "a viewer is read-only");
    }

    #[tokio::test]
    async fn a_non_member_is_forbidden() {
        let (_c, _tmp, pool, state) = cloud_state().await;
        let cloud = state.cloud.as_ref().unwrap();
        let owner = seed_user(&pool).await;
        let stranger = seed_user(&pool).await;
        let ws = cloud
            .workspaces
            .create_with_owner(owner, "W")
            .await
            .unwrap();
        let session = cloud.sessions.create(stranger).await.unwrap();

        assert_eq!(
            resolve_access(&state, &params_for(ws), &jar_with(&session.token)).await,
            Err(StatusCode::FORBIDDEN)
        );
    }

    #[tokio::test]
    async fn a_missing_cookie_is_unauthorized() {
        let (_c, _tmp, _pool, state) = cloud_state().await;
        let params = params_for(WorkspaceId(Uuid::new_v4()));

        assert_eq!(
            resolve_access(&state, &params, &CookieJar::new()).await,
            Err(StatusCode::UNAUTHORIZED)
        );
    }

    #[tokio::test]
    async fn a_missing_workspace_is_a_bad_request() {
        let (_c, _tmp, _pool, state) = cloud_state().await;

        assert_eq!(
            resolve_access(
                &state,
                &SyncParams {
                    workspace: None,
                    token: None,
                },
                &CookieJar::new()
            )
            .await,
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[tokio::test]
    async fn an_overlay_token_resolves_to_read_only_access() {
        let (_c, _tmp, pool, state) = cloud_state().await;
        let cloud = state.cloud.as_ref().unwrap();
        let owner = seed_user(&pool).await;
        let ws = cloud
            .workspaces
            .create_with_owner(owner, "W")
            .await
            .unwrap();
        let token = cloud.overlay_tokens.mint(ws).await.unwrap();

        let params = SyncParams {
            workspace: None,
            token: Some(token),
        };
        let access = resolve_access(&state, &params, &CookieJar::new())
            .await
            .unwrap();

        assert_eq!(access.workspace, ws);
        assert!(!access.can_write, "an overlay token is read-only");
    }

    #[tokio::test]
    async fn a_revoked_overlay_token_is_unauthorized() {
        let (_c, _tmp, pool, state) = cloud_state().await;
        let cloud = state.cloud.as_ref().unwrap();
        let owner = seed_user(&pool).await;
        let ws = cloud
            .workspaces
            .create_with_owner(owner, "W")
            .await
            .unwrap();
        let token = cloud.overlay_tokens.mint(ws).await.unwrap();
        let (id, _) = crate::auth::session_token::parse_token(&token).unwrap();
        cloud.overlay_tokens.revoke(id).await.unwrap();

        let params = SyncParams {
            workspace: None,
            token: Some(token.clone()),
        };
        assert_eq!(
            resolve_access(&state, &params, &CookieJar::new()).await,
            Err(StatusCode::UNAUTHORIZED)
        );
    }
}
