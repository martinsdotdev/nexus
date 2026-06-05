//! Authentication for cloud mode (ADR-0010): one session layer plus the methods that
//! establish a session. Local mode keeps no-op auth (ADR-0005 local-first is untouched).
//!
//! Layering, lowest to highest:
//! - `session_token`: the pure session-credential core (id + secret, fast `SHA-256`,
//!   constant-time verify).
//! - `email_code`: the pure email one-time-code primitives (code, slow `Argon2id`,
//!   token bucket).
//! - `session`: the Postgres-backed session store built on `session_token`.
//! - `email`: the Postgres-backed email-code store built on `email_code`; auto-provisions
//!   accounts on first verification.
//! - `email_sender`: the delivery port (log-only until a real provider is wired).
//! - `routes`: the HTTP layer (the `CurrentUser` extractor, the CSRF guard, and the
//!   `/auth/*` routes), mounted by `http::build_app` in cloud mode.
//!
//! Twitch OAuth and passkeys are later increments; they add method stores beside `email`
//! and reuse this same session layer.

pub mod email;
pub mod email_code;
pub mod email_sender;
pub mod routes;
pub mod session;
pub mod session_token;
