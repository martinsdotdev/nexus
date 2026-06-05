//! Authentication for cloud mode (ADR-0010): one session layer plus the methods that
//! establish a session (email code, Twitch OAuth, passkeys). Local mode keeps no-op
//! auth (ADR-0005 local-first is untouched). The session-token core lands first; the
//! Postgres-backed store, the cookie + CSRF layer, and the routes that consume it
//! follow in the same increment.

#[allow(dead_code)]
pub mod session_token;

// `SessionStore::create` (and the token `generate` it calls) has no production caller
// until the first login method lands next, so the lower layers still read as partly
// dead code; validate/invalidate are wired through the routes below.
#[allow(dead_code)]
pub mod session;

// The cloud-mode auth HTTP layer (extractor, CSRF guard, /auth/me + /auth/logout),
// mounted by `http::build_app` in cloud mode. Fully wired.
pub mod routes;
