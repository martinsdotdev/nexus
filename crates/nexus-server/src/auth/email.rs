//! The Postgres-backed email-code store (ADR-0010): issue a one-time code bound to an
//! unguessable request id, then verify a presented code against it with rate-limiting,
//! expiry, and auto-provisioning. Built on the pure `email_code` primitives. Cloud mode
//! only.
//!
//! Auto-provision (the chosen account model): a successful verification for an unknown
//! email creates an `app_user` plus an `email_identity` row, so email-code doubles as
//! sign-up and the recovery path. The account stays method-agnostic, Twitch and passkeys
//! will link to the same `app_user` through their own identity tables.
//!
//! Queries are runtime-checked (no `query!` macro, no offline cache), as in `session`.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::email_code::{self, TokenBucket};

/// The columns selected from a verification request: email, code hash, expiry, and the
/// token-bucket counters. Named to keep the `query_as` row type readable.
type VerificationRow = (String, String, DateTime<Utc>, i64, DateTime<Utc>);

/// How long an issued code stays valid (ADR-0010: at most one hour).
fn code_ttl() -> Duration {
    Duration::hours(1)
}

/// A freshly issued code. The `request_id` goes into the verification cookie; the raw
/// `code` is handed to the `EmailSender` and is never persisted (only its hash is).
pub struct IssuedCode {
    pub request_id: String,
    pub code: String,
}

/// The result of verifying a presented code. A discriminated union per project rule #6:
/// the four gated outcomes the route maps to distinct HTTP statuses.
#[derive(Debug, PartialEq, Eq)]
pub enum VerifyOutcome {
    /// The code matched; this is the (possibly just-created) account to log in.
    Verified(Uuid),
    /// Unknown request, or a wrong code (the two are indistinguishable, by design).
    Invalid,
    /// The request existed but its one-hour window had passed (now reaped).
    Expired,
    /// Too many attempts; the token bucket is empty. The code was not even checked.
    RateLimited,
}

/// Postgres-backed email-code store. Cheap to clone (the pool is reference-counted).
#[derive(Clone)]
pub struct EmailStore {
    pool: PgPool,
}

impl EmailStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Issue a fresh code for `email`: hash it, store a new verification request (clearing
    /// any prior one for that email so there is at most one in flight), and return the
    /// request id and the plaintext code for delivery.
    pub async fn issue(&self, email: &str) -> sqlx::Result<IssuedCode> {
        let code = email_code::generate_code();
        let code_hash = email_code::hash_code(&code);
        let request_id = new_request_id();
        let now = Utc::now();
        let bucket = TokenBucket::full(now);

        let mut tx = self.pool.begin().await?;
        sqlx::query("delete from email_verification where email = $1")
            .bind(email)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "insert into email_verification \
             (id, email, code_hash, expires_at, bucket_tokens, bucket_updated_at) \
             values ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&request_id)
        .bind(email)
        .bind(&code_hash)
        .bind(now + code_ttl())
        .bind(bucket.tokens)
        .bind(bucket.updated_at)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(IssuedCode { request_id, code })
    }

    /// Verify a presented `code` against the request `request_id`. Order matters: expiry
    /// and the rate-limit bucket are checked before the (slow) Argon2 verify, so an empty
    /// bucket short-circuits without spending CPU on a hash. On success the user is
    /// found-or-created and the request consumed.
    pub async fn verify(&self, request_id: &str, code: &str) -> sqlx::Result<VerifyOutcome> {
        let now = Utc::now();
        let row: Option<VerificationRow> = sqlx::query_as(
            "select email, code_hash, expires_at, bucket_tokens, bucket_updated_at \
             from email_verification where id = $1",
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some((email, code_hash, expires_at, tokens, bucket_updated_at)) = row else {
            return Ok(VerifyOutcome::Invalid);
        };

        if expires_at <= now {
            self.delete_request(request_id).await?;
            return Ok(VerifyOutcome::Expired);
        }

        // Spend one token first; an empty bucket is rate-limited before any hashing.
        let bucket = TokenBucket {
            tokens,
            updated_at: bucket_updated_at,
        };
        let Some(next) = bucket.try_consume(now) else {
            return Ok(VerifyOutcome::RateLimited);
        };
        sqlx::query(
            "update email_verification set bucket_tokens = $1, bucket_updated_at = $2 \
             where id = $3",
        )
        .bind(next.tokens)
        .bind(next.updated_at)
        .bind(request_id)
        .execute(&self.pool)
        .await?;

        if !email_code::verify_code(code, &code_hash) {
            return Ok(VerifyOutcome::Invalid);
        }

        let user_id = self.upsert_identity(&email).await?;
        self.delete_request(request_id).await?;
        Ok(VerifyOutcome::Verified(user_id))
    }

    /// A human display handle for the user: the local-part of their first verified email
    /// (until OAuth supplies a real display name). Used for presence and the roster.
    pub async fn display_handle(&self, user_id: Uuid) -> sqlx::Result<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as("select email from email_identity where user_id = $1 limit 1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|(email,)| email.split('@').next().unwrap_or(&email).to_string()))
    }

    /// Find the account for `email`, or create one (auto-provision). Done in a transaction
    /// so the `app_user` and its `email_identity` appear together or not at all.
    async fn upsert_identity(&self, email: &str) -> sqlx::Result<Uuid> {
        let mut tx = self.pool.begin().await?;
        let existing: Option<(Uuid,)> =
            sqlx::query_as("select user_id from email_identity where email = $1")
                .bind(email)
                .fetch_optional(&mut *tx)
                .await?;

        let user_id = match existing {
            Some((id,)) => {
                sqlx::query("update email_identity set verified_at = now() where email = $1")
                    .bind(email)
                    .execute(&mut *tx)
                    .await?;
                id
            }
            None => {
                let id = Uuid::new_v4();
                sqlx::query("insert into app_user (id) values ($1)")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("insert into email_identity (email, user_id) values ($1, $2)")
                    .bind(email)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                id
            }
        };
        tx.commit().await?;
        Ok(user_id)
    }

    async fn delete_request(&self, request_id: &str) -> sqlx::Result<()> {
        sqlx::query("delete from email_verification where id = $1")
            .bind(request_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// A 160-bit URL-safe random request id, unguessable so a request cannot be enumerated.
fn new_request_id() -> String {
    let mut bytes = [0u8; 20];
    getrandom::getrandom(&mut bytes).expect("OS RNG must be available");
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers_modules::postgres::Postgres;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;

    async fn fresh_db() -> (ContainerAsync<Postgres>, PgPool) {
        let container = Postgres::default().start().await.expect("start postgres");
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("mapped port");
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        let pool = PgPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        (container, pool)
    }

    async fn count(pool: &PgPool, sql: &str, email: &str) -> i64 {
        sqlx::query_scalar(sql)
            .bind(email)
            .fetch_one(pool)
            .await
            .expect("count query")
    }

    #[tokio::test]
    async fn issue_then_verify_auto_provisions_and_logs_in() {
        let (_c, pool) = fresh_db().await;
        let store = EmailStore::new(pool.clone());

        let issued = store.issue("user@example.com").await.unwrap();
        let outcome = store
            .verify(&issued.request_id, &issued.code)
            .await
            .unwrap();

        let user_id = match outcome {
            VerifyOutcome::Verified(id) => id,
            other => panic!("expected Verified, got {other:?}"),
        };

        // The identity links the email to the returned account...
        let identity_user: Uuid =
            sqlx::query_scalar("select user_id from email_identity where email = $1")
                .bind("user@example.com")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(identity_user, user_id);

        // ...the account row exists...
        let users: i64 = sqlx::query_scalar("select count(*) from app_user where id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(users, 1);

        // ...and the request was consumed.
        let remaining = count(
            &pool,
            "select count(*) from email_verification where id = $1",
            &issued.request_id,
        )
        .await;
        assert_eq!(remaining, 0, "the request should be consumed on success");
    }

    #[tokio::test]
    async fn a_second_login_reuses_the_same_account() {
        let (_c, pool) = fresh_db().await;
        let store = EmailStore::new(pool.clone());

        let first = store.issue("repeat@example.com").await.unwrap();
        let VerifyOutcome::Verified(id1) =
            store.verify(&first.request_id, &first.code).await.unwrap()
        else {
            panic!("first login should verify");
        };
        let second = store.issue("repeat@example.com").await.unwrap();
        let VerifyOutcome::Verified(id2) = store
            .verify(&second.request_id, &second.code)
            .await
            .unwrap()
        else {
            panic!("second login should verify");
        };

        assert_eq!(id1, id2, "the same email maps to one account");
        let users: i64 = sqlx::query_scalar("select count(*) from app_user")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(users, 1, "no duplicate account was created");
    }

    #[tokio::test]
    async fn a_wrong_code_is_invalid_and_spends_a_token() {
        let (_c, pool) = fresh_db().await;
        let store = EmailStore::new(pool.clone());

        let issued = store.issue("wrong@example.com").await.unwrap();
        assert_eq!(
            store.verify(&issued.request_id, "WRONGCOD").await.unwrap(),
            VerifyOutcome::Invalid
        );
        // The request survives a wrong attempt, with one token spent.
        let tokens: i64 =
            sqlx::query_scalar("select bucket_tokens from email_verification where id = $1")
                .bind(&issued.request_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(tokens, email_code::BUCKET_CAPACITY - 1);
    }

    #[tokio::test]
    async fn an_expired_request_is_expired_and_reaped() {
        let (_c, pool) = fresh_db().await;
        let store = EmailStore::new(pool.clone());

        let issued = store.issue("stale@example.com").await.unwrap();
        sqlx::query(
            "update email_verification set expires_at = now() - interval '1 minute' where id = $1",
        )
        .bind(&issued.request_id)
        .execute(&pool)
        .await
        .unwrap();

        assert_eq!(
            store
                .verify(&issued.request_id, &issued.code)
                .await
                .unwrap(),
            VerifyOutcome::Expired
        );
        let remaining = count(
            &pool,
            "select count(*) from email_verification where id = $1",
            &issued.request_id,
        )
        .await;
        assert_eq!(remaining, 0, "an expired request is reaped");
    }

    #[tokio::test]
    async fn attempts_are_rate_limited_after_capacity() {
        let (_c, pool) = fresh_db().await;
        let store = EmailStore::new(pool.clone());

        let issued = store.issue("burst@example.com").await.unwrap();
        // Capacity wrong attempts are merely Invalid...
        for _ in 0..email_code::BUCKET_CAPACITY {
            assert_eq!(
                store.verify(&issued.request_id, "WRONGCOD").await.unwrap(),
                VerifyOutcome::Invalid
            );
        }
        // ...the next is rate-limited, and even the correct code cannot get through now.
        assert_eq!(
            store
                .verify(&issued.request_id, &issued.code)
                .await
                .unwrap(),
            VerifyOutcome::RateLimited
        );
    }

    #[tokio::test]
    async fn an_unknown_request_is_invalid() {
        let (_c, pool) = fresh_db().await;
        let store = EmailStore::new(pool.clone());
        assert_eq!(
            store.verify("no-such-request", "WHATEVER1").await.unwrap(),
            VerifyOutcome::Invalid
        );
    }
}
