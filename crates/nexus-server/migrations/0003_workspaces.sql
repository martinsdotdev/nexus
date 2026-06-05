-- Cloud-mode schema (ADR-0009): workspaces and their memberships/roles. Each
-- workspace's Loro snapshot lives in its row, upserted on merge. Method/account
-- tables came earlier (0001/0002); overlay tokens and actor identity land in
-- their own later migrations. Local file mode never runs migrations (ADR-0005).

create table workspace (
    id         uuid primary key,
    name       text not null default 'Untitled',
    created_at timestamptz not null default now(),
    -- The Loro snapshot, upserted on every merge. Null until first persist; the
    -- runtime seeds the curated default the first time the workspace is loaded.
    snapshot   bytea
);

-- A (User, Workspace) pair carries exactly one role.
create type workspace_role as enum ('owner', 'editor', 'viewer');

create table membership (
    workspace_id uuid not null references workspace (id) on delete cascade,
    user_id      uuid not null references app_user (id) on delete cascade,
    role         workspace_role not null,
    created_at   timestamptz not null default now(),
    primary key (workspace_id, user_id)
);

create index membership_user_id_idx on membership (user_id);
