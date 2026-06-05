-- Cloud-mode schema (ADR-0010), the email-code method. Two tables: the durable
-- email -> account identity, and the in-flight verification request (an *unauthenticated*
-- challenge, distinct from `session`, which is the authenticated result). Method tables
-- for OAuth and passkeys land in later increments.

-- The email a user has proven control of, linked to their (method-agnostic) account.
create table email_identity (
    email       text primary key,
    user_id     uuid not null references app_user (id) on delete cascade,
    verified_at timestamptz not null default now()
);

create index email_identity_user_id_idx on email_identity (user_id);

-- A pending email-code challenge: the Argon2id hash of the code (never the code itself),
-- an expiry, and the token-bucket counters that rate-limit verification attempts. Keyed
-- by an unguessable id carried in the HttpOnly verification cookie ("bound to the
-- initiating session"). At most one row per email (the issue step clears prior ones).
create table email_verification (
    id                text primary key,
    email             text not null,
    code_hash         text not null,
    expires_at        timestamptz not null,
    bucket_tokens     bigint not null,
    bucket_updated_at timestamptz not null,
    created_at        timestamptz not null default now()
);
