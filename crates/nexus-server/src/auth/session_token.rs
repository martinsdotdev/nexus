//! Session tokens, the Auth Book's design (https://auth.pilcrowonpaper.com/sessions):
//! a session is a public `id` plus a 32-byte `secret`. The token handed to the client
//! is `<id>.<secret>`; the database stores only `SHA-256(secret)`, so a database leak
//! cannot forge a session. Validation re-hashes the presented secret and compares in
//! constant time. The secret is high-entropy (32 random bytes), so a fast hash is the
//! correct choice here; low-entropy credentials (email codes) use a slow hash instead.
//!
//! This module is pure: no I/O, no clock, no database. It only mints and verifies the
//! credential; the Postgres-backed store and the cookie layer build on top of it.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// A freshly minted session credential. `id` is the public lookup key the store rows by;
/// the raw `secret` is revealed to the client exactly once (inside `token`) and is never
/// persisted in the clear; `secret_hash` is what the store persists.
pub struct GeneratedSession {
    pub id: String,
    pub token: String,
    pub secret_hash: [u8; 32],
}

/// Mint a new session: a random 15-byte id and a random 32-byte secret, URL-safe
/// encoded. The client receives `token = "<id>.<secret>"`; the store keeps `id` and
/// `secret_hash`. Panics only if the OS RNG is unavailable (an unrecoverable condition).
pub fn generate() -> GeneratedSession {
    let mut id_bytes = [0u8; 15];
    let mut secret_bytes = [0u8; 32];
    getrandom::getrandom(&mut id_bytes).expect("OS RNG must be available");
    getrandom::getrandom(&mut secret_bytes).expect("OS RNG must be available");

    let id = URL_SAFE_NO_PAD.encode(id_bytes);
    let secret = URL_SAFE_NO_PAD.encode(secret_bytes);
    let secret_hash = hash_secret(&secret);
    let token = format!("{id}.{secret}");

    GeneratedSession {
        id,
        token,
        secret_hash,
    }
}

/// `SHA-256` of the secret. Fast by design: the secret is 32 random bytes, so brute
/// force is infeasible and a slow hash would only cost latency.
pub fn hash_secret(secret: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.finalize().into()
}

/// Split a client token `"<id>.<secret>"` into its parts, or `None` if malformed
/// (no delimiter, or an empty id or secret).
pub fn parse_token(token: &str) -> Option<(&str, &str)> {
    let (id, secret) = token.split_once('.')?;
    if id.is_empty() || secret.is_empty() {
        return None;
    }
    Some((id, secret))
}

/// Constant-time check that `secret` hashes to `stored_hash`. Constant-time so the
/// comparison cannot leak how many leading bytes matched.
pub fn verify_secret(secret: &str, stored_hash: &[u8; 32]) -> bool {
    let computed = hash_secret(secret);
    bool::from(computed.as_slice().ct_eq(stored_hash.as_slice()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_token_parses_back_and_verifies() {
        let session = generate();
        let (id, secret) = parse_token(&session.token).expect("token is well-formed");
        assert_eq!(id, session.id);
        assert!(verify_secret(secret, &session.secret_hash));
    }

    #[test]
    fn a_wrong_secret_does_not_verify() {
        let session = generate();
        let other = generate();
        let (_, other_secret) = parse_token(&other.token).unwrap();
        assert!(!verify_secret(other_secret, &session.secret_hash));
    }

    #[test]
    fn the_secret_is_never_stored_in_the_clear() {
        // A leaked store row (id + secret_hash) must not reconstruct the token: the raw
        // secret does not appear in the hash, and the hash is a deterministic SHA-256.
        let session = generate();
        let (_, secret) = parse_token(&session.token).unwrap();
        assert_ne!(secret.as_bytes(), session.secret_hash.as_slice());
        assert_eq!(hash_secret(secret), session.secret_hash);
    }

    #[test]
    fn malformed_tokens_are_rejected() {
        assert!(parse_token("no-delimiter").is_none());
        assert!(parse_token(".secret").is_none());
        assert!(parse_token("id.").is_none());
        assert!(parse_token("").is_none());
    }

    #[test]
    fn ids_and_tokens_are_unique_across_generations() {
        let a = generate();
        let b = generate();
        assert_ne!(a.id, b.id);
        assert_ne!(a.token, b.token);
    }
}
