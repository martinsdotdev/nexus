-- Cloud-mode schema (ADR-0009 #6): per-workspace, revocable, read-only overlay tokens.
-- An OBS browser source carries `?token=<id>.<secret>` and the relay grants a read-only
-- /sync connection to that workspace. Only the SHA-256 of the secret is stored (the same
-- discipline as `session.secret_hash`); revoke = set `revoked_at`. Multiple tokens per
-- workspace are allowed so a token can be rotated.

create table overlay_token (
    id           text primary key,
    secret_hash  bytea not null,
    workspace_id uuid not null references workspace (id) on delete cascade,
    created_at   timestamptz not null default now(),
    revoked_at   timestamptz
);

create index overlay_token_workspace_id_idx on overlay_token (workspace_id);
