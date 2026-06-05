//! Authentication for cloud mode (ADR-0010): one session layer plus the methods that
//! establish a session (email code, Twitch OAuth, passkeys). Local mode keeps no-op
//! auth (ADR-0005 local-first is untouched). The session-token core lands first; the
//! Postgres-backed store, the cookie + CSRF layer, and the routes that consume it
//! follow in the same increment.

// The token core is exercised by its own unit tests but not yet wired into a store or
// route, so its public functions read as dead code until the store lands next.
#[allow(dead_code)]
pub mod session_token;
