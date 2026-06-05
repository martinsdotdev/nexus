//! Authentication for cloud mode (ADR-0010): one session layer plus the methods that
//! establish a session (email code, Twitch OAuth, passkeys). Local mode keeps no-op
//! auth (ADR-0005 local-first is untouched). The session-token core lands first; the
//! Postgres-backed store, the cookie + CSRF layer, and the routes that consume it
//! follow in the same increment.

#[allow(dead_code)]
pub mod session_token;

// The Postgres session store, built on the token core and exercised by its own
// (testcontainers) integration tests, but not yet wired into a route or the /sync
// gate; that wiring (and the dead_code removal) lands in the next increments.
#[allow(dead_code)]
pub mod session;
