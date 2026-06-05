---
status: "accepted"
date: 2026-06-05
decision-makers: project owner
consulted:
informed: future contributors
---

# Activate v2 cloud mode: a multi-tenant hosted relay on the local-first Loro core

## Context and Problem Statement

[ADR-0002](0002-local-or-cloud-rust-core-with-qubit-rpc.md) committed Nexus to "the architecture supporting cloud deployment, not the specific cloud deployment itself," and listed the concrete cloud decisions (auth provider, database, hosting, migration, pricing, multi-tenancy) as "deferred to a future ADR when cloud becomes active." [ADR-0005](0005-offline-collaborative-loro-crdt-trusted-relay.md) likewise deferred "OAuth + actor identity in operations + per-workspace roles; cloud/Postgres persistence + multi-tenancy."

Cloud is now becoming active. A hosted instance of the relay was deployed to Railway (a configurable bind host, a multi-stage `Dockerfile`, `railway.toml`, a persistent volume), and the deployment is **not** a throwaway demo nor a side "hosted single-tenant" mode: it is **increment 1 of v2 cloud mode**, the multi-tenant hosted product. This ADR is the "future ADR" ADR-0002 promised.

The decision is sharpened by a real tension the local-first architecture creates. ADR-0005 puts the canonical workspace as a `LoroDoc` held **in the relay's process memory**, persisted to **one** snapshot, served by a **single replica**. A multi-tenant cloud is many users editing many workspaces. The question: how do we build the multi-tenant hosted product on the ADR-0005 local-first Loro core, without abandoning local-first and without the in-memory single-replica model becoming a wall, while authenticating both surfaces that open `/sync` (the editor in a logged-in browser, and the OBS overlay in a headless browser with no login)?

## Decision Drivers

* The multi-tenant hosted product is now a primary goal, not a deferred v2 feature.
* Preserve ADR-0005's local-first identity: the client must stay an offline-capable Loro replica; cloud must be additive, not a downgrade.
* Resolve the in-memory-Loro / single-replica vs many-workspaces tension surfaced by the Railway deploy (the "single replica only" constraint).
* Authenticate both `/sync` surfaces: the editor (a logged-in browser) and the OBS overlay (a headless browser with no session).
* Streamer-native onboarding (the audience already lives on Twitch).
* Reuse what ADR-0002 built for exactly this: the `Persistence` and `Auth` port seams, the one-way `nexus-core` -> `nexus-server` crate boundary, and ADR-0005's per-workspace validate/repair.
* Ground choices in the deploy realities already established on Railway (single replica, root container, public/no-auth interim).

## Considered Options

Each branch of the design tree was resolved explicitly (a `/grill-with-docs` session):

* **Deployment identity:** throwaway demo / a standalone "hosted single-tenant" mode / **increment 1 of v2 cloud mode** (chosen).
* **Local-first relationship:** **local-first stays the core, cloud is additive** (chosen) / cloud becomes primary with local as a cache / two separate products.
* **Tenancy and state model:** all workspaces resident in one process / **on-demand load/evict + sticky routing** (chosen) / per-tenant process or container / externalize sync to a managed engine.
* **Persistence backend:** **one Postgres for everything** (chosen) / Postgres for metadata + object storage for snapshots / per-workspace files on a volume.
* **Editor auth:** **HttpOnly session cookie** (chosen) / JWT bearer / query-param or subprotocol token.
* **Overlay auth:** **per-workspace read-only token in the OBS URL** (chosen) / public read-only by workspace id / a session inside OBS.
* **OAuth provider:** **Twitch first** (chosen) / Twitch + Google + GitHub at launch / email magic-link.
* **Collaboration scope at launch:** single-owner first then sharing / **full collaboration at launch** (chosen) / read-only sharing only.

## Decision Outcome

Build v2 cloud mode as a **multi-tenant hosted relay layered on the unchanged ADR-0005 local-first core**, with these eight decisions:

1. **The Railway deployment is increment 1 of v2 cloud mode.** Auth, Postgres, and multi-tenancy are built on this service; it is not a separate mode.
2. **Local-first stays the core; cloud is additive.** The client remains an offline-capable local-first Loro replica; "cloud mode" points the same client at the hosted relay (a different sync URL plus auth) instead of a local one. Migration from local to cloud is the local replica authenticating and pushing its doc up. This preserves ADR-0005's "same binary/client, swap the relay" philosophy.
3. **Tenancy is a multi-workspace doc-server with on-demand load/evict.** The relay loads a workspace's Loro snapshot from the store on its first client join, runs validate/repair and persists on merge (per ADR-0005, now per workspace), and evicts when idle. Horizontal scale is by sticky routing (a workspace's clients pinned to the process that holds its doc, the Figma per-document model ADR-0005 cited). Increment 1 is a single process serving many workspaces; sticky routing is deferred until scale demands it.
4. **Persistence is one Postgres for everything**, via `sqlx`: relational tables (users, workspaces, memberships, roles) plus each workspace's Loro snapshot as a `bytea` blob, upserted on merge. It replaces the single-volume `FilePersistence` in cloud mode. Snapshots are small (overlay configuration, not media); op-log-append plus periodic compaction is a later optimization.
5. **Editor authentication is an HttpOnly session cookie**, set after OAuth. The browser sends it on HTTP requests and on the same-origin `/sync` WebSocket upgrade, so the relay authenticates the WS join and checks workspace membership before it loads or joins the doc. (This corrects a note from the deploy: a WS handshake cannot carry an `Authorization` header, but it does carry same-origin cookies.)
6. **Overlay authentication is a per-workspace, revocable, read-only token carried in the OBS URL** (`/overlay?token=...`), the StreamElements/Streamlabs model. The relay enforces read-only on token-authenticated `/sync` connections (writes are rejected). This keeps OBS login-free.
7. **The OAuth provider is Twitch first.** Streamers already have Twitch accounts (one-click login), and Twitch OAuth later unlocks reading their channel and chat to drive the event-driven widgets Nexus already mocks (chat box, alerts, follower bubble). Google, YouTube, and GitHub are fast-follows.
8. **Collaboration is full at v2 launch:** invites, owner/editor/viewer roles, live multi-editor, and presence in increment 1. Presence rides ADR-0005's existing design (clients use Loro's `EphemeralStore` on the JS side; the relay forwards opaque presence frames; no Rust presence type is needed).

### Domain language (glossary)

This ADR introduces the cloud vocabulary; future readers should use these terms precisely:

* **Hosted relay**, the same `nexus-server` trusted relay (ADR-0005), run on a public URL instead of `127.0.0.1`. The cloud is a deployment of the relay, not a new component.
* **Tenant / User**, an authenticated identity (a Twitch account at launch). **Workspace**, the unit a `LoroDoc` represents (layouts, scenes, widgets, themes). **Membership / Role**, a (User, Workspace) pair with `owner | editor | viewer`.
* **Session**, the HttpOnly cookie identifying a logged-in editor browser. **Overlay token**, the per-workspace read-only credential in an OBS URL.
* **Doc-server**, the relay acting as a host for many workspace docs. **On-demand load/evict**, loading a workspace's snapshot on first join and dropping it when idle. **Sticky routing**, pinning a workspace's clients to the one process holding its doc.
* **Actor identity**, attributing Loro operations to a User (a future PeerID <-> UserId mapping).

### Consequences

* Good, because ADR-0005's local-first core is untouched: the same client works offline and merely chooses a relay, so there is no v2 rewrite of the client and no loss of the offline-first default.
* Good, because it reuses the seams ADR-0002 built for this exact moment: `Persistence` (swap `FilePersistence` for a Postgres `DatabasePersistence`), `Auth` (swap `NoOpAuth` for a real adapter), and the per-workspace `nexus-core` validate/repair.
* Good, because on-demand load/evict resolves the single-replica tension: one process serves many workspaces with bounded memory, and the model scales out later via sticky routing without rearchitecting.
* Good, because one same-origin session cookie authenticates both HTTP and the `/sync` WebSocket, and the overlay token keeps OBS login-free; the two surfaces that open `/sync` each get a fitting credential.
* Good, because Twitch-first onboarding is streamer-native and sets up the channel/chat integration the event widgets are built around.
* Bad, because full collaboration at launch is a large first increment: invites, role enforcement, and a presence protocol (which ADR-0005 flagged as JS-only and unbuilt) all land at once.
* Bad, because Postgres + multi-tenancy + OAuth is substantial new server surface (schema, migrations, session handling, the doc-server's eviction lifecycle) on top of today's single-workspace relay.
* Bad, because the currently-live service is public and unauthenticated in the interim, so the open-editor exposure stands until the auth sub-increment lands.
* Neutral, because sticky routing, secondary OAuth providers, account-linking, pricing/tiers, the migration UX, and op-log compaction are all deferred to later increments or ADRs.
* Neutral, because the hosted image runs as root on `debian:bookworm-slim` (it must write a freshly provisioned volume and expand `$PORT` through a shell); a hardening pass can revisit this later.

### Confirmation

This ADR is a design record, confirmed by review. The build that realizes it confirms compliance with: `cargo test -p nexus-server` for the Postgres `DatabasePersistence` (round-trip, per-workspace load/evict) and the `Auth` adapter; a Playwright e2e for the Twitch OAuth -> session cookie -> authenticated `/sync` join; a Playwright e2e for the overlay read-only token (read succeeds, write is rejected); and `cargo tree` continuing to prove `nexus-core` does not depend on `nexus-server` (the hexagonal seam survives the cloud layer).

## Pros and Cons of the Options

### Deployment identity: increment 1 of v2 cloud mode (chosen) vs hosted-single-tenant vs demo

* Good, because it commits the project to the multi-tenant product and gives the deploy work a durable purpose.
* Neutral, because it raises the stakes: the committed Dockerfile/railway.toml are now product infrastructure, not convenience.
* Bad (the rejected alternatives), because "hosted single-tenant mode" would have been a third mode to maintain with no product upside, and "throwaway demo" would leave the live URL undocumented and unowned.

### Tenancy: on-demand load/evict + sticky routing (chosen)

* Good, because it preserves ADR-0005 per workspace and scales with bounded memory.
* Neutral, because it needs an eviction lifecycle and, for multi-process scale, a sticky-routing layer (deferred).
* Bad (alternatives), because "all resident in one process" hits a RAM and single-replica wall, "per-tenant process" is heavy ops and cost, and "externalize sync to a managed engine" partially reverses ADR-0005 (which rejected Zero/Electric for being non-local-first and non-Rust).

### Persistence: one Postgres for everything (chosen)

* Good, because a single transactional backend holds both metadata and snapshots; `sqlx` was already the anticipated choice; Railway runs managed Postgres.
* Neutral, because hot/large docs may later want op-log-append or an object store for blobs.
* Bad (alternatives), because metadata-plus-object-storage is two backends and cross-store consistency for KB-sized snapshots, and per-workspace volume files re-impose the single-replica wall.

### Editor auth: session cookie (chosen) vs JWT vs URL token

* Good, because one same-origin cookie covers HTTP and the WS handshake with no URL token.
* Neutral, because it relies on the UI and `/sync` staying same-origin (true on Railway; revisit if the UI ever moves to a separate CDN origin).
* Bad (alternatives), because a JWT still needs a cookie or query param for the WS, and URL tokens leak through logs, history, and referrers.

### Overlay auth: read-only token in the URL (chosen)

* Good, because it authorizes a login-free OBS source to one workspace, revocably, the proven streaming-overlay pattern.
* Bad (alternatives), because public-by-id exposes unpublished scenes/drafts with no revocation, and a session inside OBS is fragile cookie/login management.

### OAuth: Twitch first (chosen)

* Good, because it is streamer-native and unlocks the chat/channel tie-in.
* Neutral, because non-Twitch users wait for a fast-follow provider.
* Bad (alternatives), because three providers at launch multiply OAuth apps and account-linking edge cases, and magic-link means building the whole email surface.

### Collaboration: full at launch (chosen)

* Good, because it honors ADR-0005's multiplayer thesis immediately; the relay already does multi-editor sync.
* Bad, because it is the largest first increment (invites, roles, presence) and presence has no Rust type today.

## More Information

* **Relationship to [ADR-0002](0002-local-or-cloud-rust-core-with-qubit-rpc.md):** this ADR is the "future ADR when cloud becomes active" that ADR-0002 promised. It resolves ADR-0002's deferred cloud decisions: auth provider (Twitch OAuth + session cookie), database (Postgres via `sqlx`), hosting (Railway), and migration (local replica pushes its doc up). It honors ADR-0002's foundational implications: `Persistence` and `Auth` stay ports, workspace ownership enters the data model (now as Postgres tables rather than `Workspace.ownerId`), and configuration is environment-variable-first.
* **Relationship to [ADR-0005](0005-offline-collaborative-loro-crdt-trusted-relay.md):** this ADR **preserves**, and does not supersede, ADR-0005's local-first Loro core. The cloud relay is the hosted instance of the trusted relay; per-workspace validate/repair, the Loro substrate, and offline editing are unchanged. It un-defers ADR-0005's explicitly deferred items (OAuth + actor identity in operations + per-workspace roles; cloud/Postgres persistence + multi-tenancy; presence forwarding).
* **Increment 1's hosting** is already committed: the configurable bind host (`--host` / `NEXUS_HOST`), the multi-stage `Dockerfile` (debian-slim runtime, shell-form `CMD` so `$PORT` expands, runs as root to write the volume), and `railway.toml`. The live service is `https://nexus-production-9382.up.railway.app`.
* **Deferred** to later increments or ADRs: the sticky-routing mechanism; secondary OAuth providers and account-linking; the actor-identity-in-operations encoding; the presence protocol specifics; the invite/role UX; pricing and free/paid tiers; the migration UX; op-log compaction.
* **Interim posture:** the live service stays open (unguessable URL, no real data) as increment 1's staging target, but Twitch OAuth plus the session cookie should be the first sub-increment so the open-editor exposure closes early.
