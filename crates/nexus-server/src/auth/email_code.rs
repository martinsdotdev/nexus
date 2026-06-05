//! The email one-time code primitives (ADR-0010), pure and I/O-free except for the OS
//! RNG (as in `session_token`). Three pieces: the 8-character code (drawn from an
//! unambiguous 32-symbol alphabet, >= 40 bits), the Argon2id hash of it, and a token
//! bucket for rate-limiting verification attempts.
//!
//! The two-hashes rationale (ADR-0010): the session secret is high-entropy, so it uses a
//! fast `SHA-256`; this code is low-entropy (~40 bits), so it demands a deliberately slow
//! `Argon2id` (16 MiB, t=3, p=1), or a fast hash would let an attacker brute-force the
//! code within its one-hour validity. The clock is never read here: the token bucket
//! takes `now` as a value, so this module stays pure (the store injects the time).

use argon2::password_hash::{PasswordHash, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use chrono::{DateTime, Duration, Utc};

/// Code length in characters. Eight symbols over a 32-symbol alphabet is 40 bits.
pub const CODE_LEN: usize = 8;

/// A-Z and the digits, minus the visually ambiguous `I O 0 1`. Exactly 32 symbols, so a
/// random byte masked to its low 5 bits indexes it without modulo bias.
const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Token-bucket capacity: at most this many verification attempts before refilling.
pub const BUCKET_CAPACITY: i64 = 5;

/// One token accrues per this interval (1 per minute).
fn refill_interval() -> Duration {
    Duration::minutes(1)
}

/// Mint a fresh 8-character code. Panics only if the OS RNG is unavailable (as in
/// `session_token::generate`, an unrecoverable condition).
pub fn generate_code() -> String {
    let mut bytes = [0u8; CODE_LEN];
    getrandom::getrandom(&mut bytes).expect("OS RNG must be available");
    bytes
        .iter()
        .map(|b| ALPHABET[(b & 0x1F) as usize] as char)
        .collect()
}

/// Argon2id PHC hash of a code (16 MiB, t=3, p=1). The salt is drawn from the OS RNG and
/// encoded into the returned string, so the hash is self-describing and `verify_code`
/// needs no external parameters.
pub fn hash_code(code: &str) -> String {
    let mut salt_bytes = [0u8; 16];
    getrandom::getrandom(&mut salt_bytes).expect("OS RNG must be available");
    let salt = SaltString::encode_b64(&salt_bytes).expect("16 bytes is a valid salt");
    hasher()
        .hash_password(code.as_bytes(), &salt)
        .expect("hashing a code cannot fail")
        .to_string()
}

/// Constant-time-ish verify of a code against its Argon2id PHC hash (Argon2 derives the
/// parameters from the encoded hash). A malformed hash returns `false` rather than error.
pub fn verify_code(code: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(code.as_bytes(), &parsed)
        .is_ok()
}

/// The configured Argon2id hasher: 16 MiB of memory, 3 passes, 1 lane (ADR-0010).
fn hasher() -> Argon2<'static> {
    let params = Params::new(16 * 1024, 3, 1, None).expect("valid Argon2 parameters");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// A token bucket for rate-limiting verification attempts. Pure: refill is computed from
/// the `now` passed in, never from a clock read. The store persists `tokens` +
/// `updated_at` and reconstructs the bucket on the next attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenBucket {
    pub tokens: i64,
    pub updated_at: DateTime<Utc>,
}

impl TokenBucket {
    /// A full bucket as of `now` (the state stored when a code is issued).
    pub fn full(now: DateTime<Utc>) -> Self {
        TokenBucket {
            tokens: BUCKET_CAPACITY,
            updated_at: now,
        }
    }

    /// Try to spend one token as of `now`, accruing any tokens earned since `updated_at`
    /// first. Returns the new bucket state on success, or `None` if the bucket is empty
    /// (the caller treats `None` as rate-limited). Sub-interval progress is preserved
    /// unless the bucket refills to capacity, in which case the clock resets to `now`.
    pub fn try_consume(&self, now: DateTime<Utc>) -> Option<TokenBucket> {
        let interval = refill_interval().num_seconds();
        let elapsed = (now - self.updated_at).num_seconds().max(0);
        let refilled = elapsed / interval;

        let (tokens, updated_at) = if self.tokens + refilled >= BUCKET_CAPACITY {
            (BUCKET_CAPACITY, now)
        } else {
            (
                self.tokens + refilled,
                self.updated_at + Duration::seconds(refilled * interval),
            )
        };

        if tokens < 1 {
            return None;
        }
        Some(TokenBucket {
            tokens: tokens - 1,
            updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(secs: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(secs, 0).expect("valid timestamp")
    }

    #[test]
    fn a_code_is_eight_unambiguous_symbols() {
        for _ in 0..100 {
            let code = generate_code();
            assert_eq!(code.chars().count(), CODE_LEN);
            for c in code.chars() {
                assert!(
                    ALPHABET.contains(&(c as u8)),
                    "char {c:?} is outside the alphabet"
                );
                assert!(!"IO01".contains(c), "ambiguous char {c:?} leaked in");
            }
        }
    }

    #[test]
    fn codes_vary_between_calls() {
        // Collision odds are (1/32)^8 per pair; a difference is overwhelmingly expected.
        assert_ne!(generate_code(), generate_code());
    }

    #[test]
    fn a_code_verifies_against_its_hash_but_a_wrong_code_does_not() {
        let code = generate_code();
        let hash = hash_code(&code);
        assert!(verify_code(&code, &hash));
        assert!(!verify_code("WRONGCOD", &hash));
    }

    #[test]
    fn the_hash_is_argon2id_and_does_not_contain_the_code() {
        let code = generate_code();
        let hash = hash_code(&code);
        assert!(hash.starts_with("$argon2id$"), "got {hash}");
        assert!(!hash.contains(&code));
    }

    #[test]
    fn a_garbage_hash_string_fails_closed() {
        assert!(!verify_code("anything", "not-a-phc-string"));
    }

    #[test]
    fn the_bucket_allows_capacity_attempts_then_blocks() {
        let now = at(1_000_000_000);
        let mut bucket = TokenBucket::full(now);
        for _ in 0..BUCKET_CAPACITY {
            bucket = bucket.try_consume(now).expect("within capacity");
        }
        assert!(bucket.try_consume(now).is_none(), "should be exhausted");
    }

    #[test]
    fn the_bucket_refills_one_token_per_minute() {
        let now = at(1_000_000_000);
        let mut bucket = TokenBucket::full(now);
        for _ in 0..BUCKET_CAPACITY {
            bucket = bucket.try_consume(now).unwrap();
        }
        assert!(bucket.try_consume(now).is_none());

        // After one minute, exactly one token is available again.
        let later = now + Duration::minutes(1);
        let after = bucket.try_consume(later).expect("one token refilled");
        assert!(after.try_consume(later).is_none(), "only one, not two");
    }
}
