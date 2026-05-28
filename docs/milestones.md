# Nexus, Milestones & Roadmap

**Status:** Scaffold (versions defined; deliverables, dependencies, dates TBD).
**Last updated:** 2026-05-13
**Related:** [`docs/prd.md`](prd.md), [`docs/architecture.md`](architecture.md)

This document captures the version sequence. Each version corresponds to a persona expansion and/or surface-area expansion.

Dates are intentionally absent. **Sequencing > scheduling** at this stage; once v1 implementation is underway, dates become possible to estimate.

## v1, Developer/Designer-streamer beachhead

**Target personas:** developer-streamer, designer-streamer (per [PRD §3](prd.md) and spec §1.6 once committed).

**Deliverables (TBD; reference spec §1.5 v1 scope for the complete in-scope list):**
- Local Rust binary, cross-platform via `cargo-dist`
- SvelteKit UI served by the binary
- qubit RPC + TanStack Query integration
- 4 themes (Cozy / Cyber / Editorial / Sticker), 8 widgets, 1 default layout
- obs-websocket integration via `obws`
- Mock chat / event sources (no real platform integrations)
- Filesystem persistence (no cloud)
- Single tenant (no auth)
- ADRs and design spec maintained alongside implementation

**Out of scope:** accounts, cloud sync, multi-platform integrations, team workspaces, real chat sources, marketplaces.

**Success criteria:** TBD, depends on PRD §7. Honest baseline: "used in production by N developer-streamers, with docs and ADRs current."

## v2, Cloud mode + multi-platform integrations

**Target personas:** + mass-market streamer.

**Deliverables (TBD):**
- Cloud-mode deployment (`NEXUS_MODE=cloud` switches adapters)
- `DatabasePersistence` adapter (Postgres via sqlx)
- OAuth-based `Auth` adapter
- Multi-tenancy (workspace ownership via `Workspace.ownerId: Some(UserId)`)
- Twitch chat + EventSub source adapters
- YouTube live chat + event source adapters
- Kick chat + event source adapters
- Hosted website (nexus.app or similar) serving the editor + overlay routes

**Architectural enabler:** [ADR-0002](decisions/0002-local-or-cloud-rust-core-with-qubit-rpc.md) was designed for this. The architecture supports v2 without rework, only new adapters.

## v3, Team workspaces + premium tier (Tauri installer)

**Target personas:** + agency / multi-streamer manager + partnered / megastar.

**Deliverables (TBD):**
- Team workspaces (`Workspace.ownerId` extends from `UserId` to `OrgId`)
- Role-based access (owner / editor / viewer)
- Asset libraries shared across team members
- Audit logs
- Billing integration (subscription tiers)
- Tauri-packaged desktop installer (auto-update, system tray, native OS integration)
- Premium widget packs / curated theme commissions

## v4, Enterprise / esports broadcast

**Target personas:** + esports broadcast production.

**Deliverables (TBD):**
- OpenTelemetry semantic conventions (structured tracing, OTLP export)
- High-reliability SLA tier
- Multi-operator scene sync (multiple operators editing the same overlay coordinated)
- Network-replicated layouts (cross-machine state via WebSocket)
- On-premise / self-hosted deployment option

## Cross-cutting milestones (orthogonal to version line)

- Documentation site (docs.nexus.app or similar), TBD when content density justifies
- Marketing site (nexus.app landing), TBD aligned with v2 launch
- `cargo-dist` binary releases per version, starting v1
- Theme marketplace, deferred until at least v3, depends on community gravity
- Internationalization (i18n), TBD per market expansion

## Sequencing constraints

- v1 is independent (local mode only, mocked sources).
- v2 cloud mode requires v1 architecture intact (no rearchitecture; ADR-0002 confirmed this).
- v3 team workspaces require v2 cloud (auth + multi-tenancy already in place).
- v4 enterprise requires v3 (premium tier mechanics already in place).

Each version is **additive**, v3 doesn't break v1 local mode for existing developer/designer-streamer adopters. The Rust binary keeps working in single-user mode forever.

## Open questions on milestones

- **v1 → v2 cadence:** 3 months? 6 months? Depends on v1 adoption traction.
- **Whether v3 team workspaces precede or follow v2 cloud-mode release for individuals**, currently sequenced individuals first, but a B2B-first sequence is defensible if early v1 adoption signals an agency interest.
- **Whether to ship a "v1.5" with a Tauri installer** (no cloud) before v2, to broaden the local-mode audience to designer-streamers without committing to cloud infrastructure.
- **Pricing model for v2 hosted offering**, affects v2 deliverable list (free tier? freemium? paid only with trial?).
