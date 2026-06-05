//! The cloud-mode auth HTTP layer (ADR-0010): the `CurrentUser` extractor (a valid
//! session cookie resolves to the authenticated user, else 401), a CSRF guard on
//! state-changing requests (the Auth Book's `Sec-Fetch-Site` check on top of the
//! cookie's `SameSite=Lax`), and the `/auth/me` + `/auth/logout` routes. Mounted only
//! in cloud mode. Login methods, which set the cookie, land in later increments; here
//! the cookie is only read and cleared.

use axum::Router;
use axum::extract::{FromRequestParts, Request, State};
use axum::http::{StatusCode, request::Parts};
use axum::middleware::{self, Next};
use axum::response::{Json, Response};
use axum::routing::{get, post};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use serde::Serialize;

use super::session::AuthUser;
use super::session_token;
use crate::http::AppState;

/// Name of the session cookie.
pub const COOKIE_NAME: &str = "nexus_session";

/// The authenticated user, extracted from a valid session cookie. A handler taking this
/// argument is reachable only by a logged-in request; otherwise the extractor rejects
/// with 401 (including in local mode, where there is no session store).
pub struct CurrentUser(pub AuthUser);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let sessions = state.sessions.as_ref().ok_or(StatusCode::UNAUTHORIZED)?;
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .expect("CookieJar extraction is infallible");
        let cookie = jar.get(COOKIE_NAME).ok_or(StatusCode::UNAUTHORIZED)?;
        match sessions.validate(cookie.value()).await {
            Ok(Some(user)) => Ok(CurrentUser(user)),
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }
}

/// CSRF guard (Auth Book): reject any non-safe (state-changing) request that is not
/// same-origin per `Sec-Fetch-Site`. The cookie's `SameSite=Lax` is the first layer;
/// this server-side check is the second. (An `Origin`-whitelist fallback for older
/// browsers is a later refinement.)
async fn csrf_guard(req: Request, next: Next) -> Result<Response, StatusCode> {
    let safe = matches!(req.method().as_str(), "GET" | "HEAD" | "OPTIONS");
    if !safe {
        let same_origin = req
            .headers()
            .get("sec-fetch-site")
            .is_some_and(|v| v.as_bytes() == b"same-origin");
        if !same_origin {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    Ok(next.run(req).await)
}

#[derive(Serialize)]
struct Me {
    user_id: String,
}

/// `GET /auth/me`: the current user, or 401 if not logged in.
async fn me(CurrentUser(user): CurrentUser) -> Json<Me> {
    Json(Me {
        user_id: user.id.to_string(),
    })
}

/// `POST /auth/logout`: invalidate the session server-side and clear the cookie.
async fn logout(State(state): State<AppState>, jar: CookieJar) -> (CookieJar, StatusCode) {
    if let (Some(sessions), Some(cookie)) = (state.sessions.as_ref(), jar.get(COOKIE_NAME))
        && let Some((id, _)) = session_token::parse_token(cookie.value())
    {
        let _ = sessions.invalidate(id).await;
    }
    let cleared = jar.remove(Cookie::build((COOKIE_NAME, "")).path("/").build());
    (cleared, StatusCode::NO_CONTENT)
}

/// The cloud-mode auth routes, with the CSRF guard layered on.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/me", get(me))
        .route("/auth/logout", post(logout))
        .layer(middleware::from_fn(csrf_guard))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::build_app;
    use crate::persistence::FilePersistence;
    use crate::runtime::WorkspaceRuntime;

    use super::super::session::SessionStore;
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use sqlx::PgPool;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use tower::ServiceExt;
    use uuid::Uuid;

    // A cloud AppState backed by an ephemeral Postgres, plus the pool for seeding.
    async fn cloud_app() -> (
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
        let runtime = WorkspaceRuntime::new(FilePersistence::new(tmp.path())).unwrap();
        let state = AppState {
            runtime,
            sessions: Some(SessionStore::new(pool.clone())),
        };
        (container, tmp, state, pool)
    }

    #[tokio::test]
    async fn me_and_logout_enforce_the_session_and_csrf() {
        let (_c, _tmp, state, pool) = cloud_app().await;
        let store = state.sessions.clone().unwrap();
        let user_id = Uuid::new_v4();
        sqlx::query("insert into app_user (id) values ($1)")
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();
        let session = store.create(user_id).await.unwrap();
        let cookie = format!("{COOKIE_NAME}={}", session.token);

        // /auth/me with a valid cookie -> 200.
        let resp = build_app(state.clone(), None)
            .oneshot(
                HttpRequest::builder()
                    .uri("/auth/me")
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // /auth/me with no cookie -> 401.
        let resp = build_app(state.clone(), None)
            .oneshot(
                HttpRequest::builder()
                    .uri("/auth/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // POST /auth/logout without a same-origin header -> 403 (CSRF guard).
        let resp = build_app(state.clone(), None)
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri("/auth/logout")
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // POST /auth/logout, same-origin -> 204, and the session is invalidated.
        let resp = build_app(state.clone(), None)
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri("/auth/logout")
                    .header("cookie", &cookie)
                    .header("sec-fetch-site", "same-origin")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert!(store.validate(&session.token).await.unwrap().is_none());
    }
}
