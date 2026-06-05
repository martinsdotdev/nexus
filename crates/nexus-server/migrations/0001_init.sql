-- Cloud-mode schema (ADR-0009/0010), increment 1: accounts + sessions. Method tables
-- (email codes, OAuth accounts, passkeys) and workspace/membership tables land in
-- later increments. Local file mode never runs migrations.

create table app_user (
    id         uuid primary key,
    created_at timestamptz not null default now()
);

create table session (
    id          text primary key,
    secret_hash bytea not null,
    user_id     uuid not null references app_user (id) on delete cascade,
    created_at  timestamptz not null default now(),
    expires_at  timestamptz not null
);

create index session_user_id_idx on session (user_id);
