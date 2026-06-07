-- Cloud-mode session device metadata (ADR-0010): full session management lets a user see
-- where they are signed in and sign other devices out. Each session records the client's
-- user agent, its IP (the proxy's X-Forwarded-For in cloud), and the last time the token
-- was used (bumped on every validate, so "last active" is meaningful).
alter table session
    add column user_agent   text,
    add column ip           text,
    add column last_used_at timestamptz not null default now();
