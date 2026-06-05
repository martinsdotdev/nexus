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
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};

use super::email::VerifyOutcome;
use super::session::AuthUser;
use super::session_token;
use crate::http::AppState;

/// Name of the session cookie.
pub const COOKIE_NAME: &str = "nexus_session";

/// Name of the short-lived cookie that binds an in-flight email-code challenge to this
/// browser (the Auth Book's "bound to the initiating session"). It carries the
/// verification request id; the code itself is mailed, never put in a cookie.
const EMAIL_COOKIE: &str = "nexus_email_verification";

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
        let cloud = state.cloud.as_ref().ok_or(StatusCode::UNAUTHORIZED)?;
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .expect("CookieJar extraction is infallible");
        let cookie = jar.get(COOKIE_NAME).ok_or(StatusCode::UNAUTHORIZED)?;
        match cloud.sessions.validate(cookie.value()).await {
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
    if let (Some(cloud), Some(cookie)) = (state.cloud.as_ref(), jar.get(COOKIE_NAME))
        && let Some((id, _)) = session_token::parse_token(cookie.value())
    {
        let _ = cloud.sessions.invalidate(id).await;
    }
    let cleared = jar.remove(Cookie::build((COOKIE_NAME, "")).path("/").build());
    (cleared, StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct EmailRequest {
    email: String,
}

#[derive(Deserialize)]
struct VerifyRequest {
    code: String,
}

/// `POST /auth/email`: issue a one-time code for the address and hand it to the sender.
/// Always responds 200 with no body whether or not the address maps to an account, so it
/// never reveals which addresses are registered (no account enumeration). A malformed
/// address is the one exception, 400, since that leaks nothing about registration. Sets
/// the verification cookie that the verify step requires.
async fn post_email(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<EmailRequest>,
) -> Result<(CookieJar, StatusCode), StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let email = normalize_email(&body.email);
    if !is_plausible_email(&email) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let issued = cloud.emails.issue(&email).await.map_err(internal)?;
    cloud
        .sender
        .send_code(&email, &issued.code)
        .await
        .map_err(internal)?;

    let jar = jar.add(verification_cookie(issued.request_id));
    Ok((jar, StatusCode::OK))
}

/// `POST /auth/email/verify`: check the submitted code against the challenge named by the
/// verification cookie. On success, mint a session, set the session cookie, clear the
/// challenge cookie, and return the user. Maps the other outcomes to distinct statuses:
/// rate-limited -> 429, expired or wrong -> 401, no challenge cookie -> 400.
async fn post_email_verify(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<VerifyRequest>,
) -> Result<(CookieJar, Json<Me>), StatusCode> {
    let cloud = state.cloud.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let challenge = jar.get(EMAIL_COOKIE).ok_or(StatusCode::BAD_REQUEST)?;

    match cloud
        .emails
        .verify(challenge.value(), &body.code)
        .await
        .map_err(internal)?
    {
        VerifyOutcome::Verified(user_id) => {
            let session = cloud.sessions.create(user_id).await.map_err(internal)?;
            let jar = jar
                .add(session_cookie(session.token))
                .remove(clear_cookie(EMAIL_COOKIE));
            Ok((
                jar,
                Json(Me {
                    user_id: user_id.to_string(),
                }),
            ))
        }
        VerifyOutcome::RateLimited => Err(StatusCode::TOO_MANY_REQUESTS),
        VerifyOutcome::Expired | VerifyOutcome::Invalid => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Trim and lowercase an address so `Foo@x.com` and `foo@x.com` are one identity.
fn normalize_email(raw: &str) -> String {
    raw.trim().to_lowercase()
}

/// A minimal plausibility check (not full RFC 5322): one `@`, non-empty local part, and a
/// dotted domain. Enough to reject obvious junk before issuing a code; real deliverability
/// is proven by the code arriving.
fn is_plausible_email(email: &str) -> bool {
    if email.len() > 320 {
        return false;
    }
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

/// The 30-day session cookie: `HttpOnly` + `Secure` + `SameSite=Lax` + `Path=/`
/// (ADR-0010). `HttpOnly` keeps it out of JS; `SameSite=Lax` plus the CSRF guard covers
/// cross-site posts.
fn session_cookie(token: String) -> Cookie<'static> {
    Cookie::build((COOKIE_NAME, token))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time::Duration::days(30))
        .build()
}

/// The one-hour challenge cookie, same attributes as the session cookie but scoped to the
/// code's lifetime.
fn verification_cookie(request_id: String) -> Cookie<'static> {
    Cookie::build((EMAIL_COOKIE, request_id))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(time::Duration::hours(1))
        .build()
}

/// An expired, empty cookie that instructs the browser to drop `name` (same path as when
/// it was set, so the removal matches).
fn clear_cookie(name: &'static str) -> Cookie<'static> {
    Cookie::build((name, "")).path("/").build()
}

/// Map any internal error to a 500 after logging it. Auth routes return bare statuses
/// today (RFC 9457 Problem Details across the auth surface is a later refinement).
fn internal<E: std::fmt::Display>(err: E) -> StatusCode {
    tracing::error!("auth route error: {err}");
    StatusCode::INTERNAL_SERVER_ERROR
}

/// The cloud-mode auth routes, with the CSRF guard layered on.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/me", get(me))
        .route("/auth/logout", post(logout))
        .route("/auth/email", post(post_email))
        .route("/auth/email/verify", post(post_email_verify))
        .layer(middleware::from_fn(csrf_guard))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::build_app;
    use crate::persistence::{FilePersistence, WorkspaceId};
    use crate::runtime::WorkspaceRuntime;

    use super::super::email::EmailStore;
    use super::super::email_sender::EmailSender;
    use super::super::session::SessionStore;
    use crate::http::CloudAuth;
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use sqlx::PgPool;
    use std::sync::{Arc, Mutex};
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use tower::ServiceExt;
    use uuid::Uuid;

    // Every (recipient, code) a Capture sender recorded, so a test can read the code a
    // handler "mailed".
    type Mailbox = Arc<Mutex<Vec<(String, String)>>>;

    // A cloud AppState backed by an ephemeral Postgres, with a Capture email sender. The
    // pool (for seeding) and the mailbox (for reading sent codes) come back too.
    async fn cloud_app() -> (
        ContainerAsync<Postgres>,
        tempfile::TempDir,
        AppState,
        PgPool,
        Mailbox,
    ) {
        let container = Postgres::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let runtime = WorkspaceRuntime::load(
            WorkspaceId::LOCAL,
            Arc::new(FilePersistence::new(tmp.path())),
        )
        .await
        .unwrap();
        let mailbox: Mailbox = Arc::new(Mutex::new(Vec::new()));
        let state = AppState {
            runtime,
            cloud: Some(CloudAuth {
                sessions: SessionStore::new(pool.clone()),
                emails: EmailStore::new(pool.clone()),
                sender: EmailSender::Capture(mailbox.clone()),
            }),
        };
        (container, tmp, state, pool, mailbox)
    }

    // Build a POST carrying a JSON body and the given headers.
    fn post_json(uri: &str, body: &str, headers: &[(&str, &str)]) -> HttpRequest<Body> {
        let mut builder = HttpRequest::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json");
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        builder.body(Body::from(body.to_string())).unwrap()
    }

    // The value of the first `Set-Cookie: <name>=...` on the response, if any.
    fn set_cookie(resp: &Response, name: &str) -> Option<String> {
        let prefix = format!("{name}=");
        for header in resp.headers().get_all("set-cookie") {
            let text = header.to_str().ok()?;
            if let Some(rest) = text.strip_prefix(&prefix) {
                return Some(rest.split(';').next().unwrap_or_default().to_string());
            }
        }
        None
    }

    #[tokio::test]
    async fn me_and_logout_enforce_the_session_and_csrf() {
        let (_c, _tmp, state, pool, _mail) = cloud_app().await;
        let store = state.cloud.clone().unwrap().sessions;
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

    #[tokio::test]
    async fn email_code_login_full_flow_sets_a_session() {
        let (_c, _tmp, state, _pool, mailbox) = cloud_app().await;
        let same_origin = ("sec-fetch-site", "same-origin");

        // 1. Request a code for a brand-new (mixed-case) address.
        let resp = build_app(state.clone(), None)
            .oneshot(post_json(
                "/auth/email",
                r#"{"email":"NewUser@Example.com"}"#,
                &[same_origin],
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let challenge = set_cookie(&resp, EMAIL_COOKIE).expect("verification cookie is set");

        // The sender received the code, addressed to the normalized address.
        let (to, code) = mailbox
            .lock()
            .unwrap()
            .last()
            .cloned()
            .expect("a code was sent");
        assert_eq!(to, "newuser@example.com");

        // 2. Submit the code, carrying the challenge cookie.
        let cookie_header = format!("{EMAIL_COOKIE}={challenge}");
        let resp = build_app(state.clone(), None)
            .oneshot(post_json(
                "/auth/email/verify",
                &format!(r#"{{"code":"{code}"}}"#),
                &[same_origin, ("cookie", cookie_header.as_str())],
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let session = set_cookie(&resp, COOKIE_NAME).expect("session cookie is set");
        assert!(!session.is_empty());

        // 3. The freshly minted session authenticates /auth/me.
        let resp = build_app(state.clone(), None)
            .oneshot(
                HttpRequest::builder()
                    .uri("/auth/me")
                    .header("cookie", format!("{COOKIE_NAME}={session}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn a_wrong_code_does_not_log_in() {
        let (_c, _tmp, state, _pool, _mail) = cloud_app().await;
        let same_origin = ("sec-fetch-site", "same-origin");

        let resp = build_app(state.clone(), None)
            .oneshot(post_json(
                "/auth/email",
                r#"{"email":"a@example.com"}"#,
                &[same_origin],
            ))
            .await
            .unwrap();
        let challenge = set_cookie(&resp, EMAIL_COOKIE).unwrap();
        let cookie_header = format!("{EMAIL_COOKIE}={challenge}");

        let resp = build_app(state.clone(), None)
            .oneshot(post_json(
                "/auth/email/verify",
                r#"{"code":"WRONGCOD"}"#,
                &[same_origin, ("cookie", cookie_header.as_str())],
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        assert!(
            set_cookie(&resp, COOKIE_NAME).is_none(),
            "no session is issued on a wrong code"
        );
    }

    #[tokio::test]
    async fn verify_is_rejected_without_a_challenge_or_without_same_origin() {
        let (_c, _tmp, state, _pool, _mail) = cloud_app().await;

        // No challenge cookie -> 400.
        let resp = build_app(state.clone(), None)
            .oneshot(post_json(
                "/auth/email/verify",
                r#"{"code":"WHATEVER1"}"#,
                &[("sec-fetch-site", "same-origin")],
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // Issuing a code cross-site (no same-origin header) -> 403 (CSRF guard).
        let resp = build_app(state.clone(), None)
            .oneshot(post_json(
                "/auth/email",
                r#"{"email":"a@example.com"}"#,
                &[],
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
