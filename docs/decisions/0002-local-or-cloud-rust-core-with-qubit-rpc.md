---
status: "accepted"
date: 2026-05-13
decision-makers: project owner
consulted: 
informed: future contributors
---

# Local-or-cloud Rust core with qubit RPC and SvelteKit UI

## Context and Problem Statement

Earlier brainstorming committed Nexus to a TypeScript + React static-SPA architecture: front-end-only, persisted to localStorage, with obs-websocket spoken from the browser, BroadcastChannel mirroring editor↔overlay. That architecture has known weaknesses: state duplication between editor and overlay, fragile localStorage-as-source-of-truth, JS-bound obs-websocket integration with browser CORS quirks, and no clear v2 path when the backend lands.

Two new constraints emerged in subsequent rounds:

1. The implementation language pivots to **Rust + SvelteKit**, with Rust intended for the core (Decider, types, validation, business logic) and SvelteKit for the UI.
2. The system should run both **locally** (binary on the streamer's machine, default deployment) and eventually **in the cloud** (hosted, multi-tenant, with sync and collaboration).

These together rule out two simpler architectures:

- **Rust-as-WASM-in-browser** (Architecture A): the Rust core compiles to WebAssembly and runs in the browser; SvelteKit consumes it via `wasm-bindgen`. Avoids a backend, but doesn't translate to a cloud-hosted multi-tenant story without significant rearchitecture. State stays in browser localStorage; persistence and auth become bolt-ons in v2.
- **Static SvelteKit + a separate cloud backend later**: SvelteKit ships statically; cloud backend appears in v2 as a new project. Doubles the codebase and creates a hard split between local-only and cloud modes.

We need an architecture where the same Rust core powers both the local-default-deployment and the eventual cloud deployment, with adapters configured per environment.

## Decision Drivers

- Same codebase serves local and cloud deployments; no v2-shaped rewrite.
- Local-first remains the primary streamer UX (low friction, private, offline).
- Cloud sync / collaboration is a future paid offering on the same Rust core.
- Type safety across the Rust↔TypeScript boundary without manual schema-keeping.
- OBS Browser Source remains the streamer's render target (no UX regression on the streaming side).
- Open-source-friendly: contributors can run the full system locally without a cloud account.
- Hexagonal architecture and the seven discipline rules from the spec carry over unchanged.

## Considered Options

- **A. Rust-as-WASM, no server, browser-only** (the pre-pivot architecture)
- **B. Rust-as-local-server with qubit RPC; SvelteKit UI loads from the server; cloud is a future deployment of the same code** ← chosen
- **C. SvelteKit + TS-only on the frontend; a separate Rust backend for v2 only**
- **D. Tauri desktop app wrapping SvelteKit + Rust** (Rust as embedded process, not network server)

## Decision Outcome

Chosen option: **"B. Rust-as-local-server with qubit RPC; cloud is a future deployment of the same code."**

The Rust core is a daemon, a long-lived process that:
- Owns workspace state authoritatively (no editor↔overlay duplication).
- Persists via a `Persistence` trait: filesystem adapter for local deployment, database adapter for cloud deployment.
- Serves the static SvelteKit assets over HTTP (so OBS Browser Source URL is `http://localhost:<port>/overlay?layout=<id>` locally, or `https://nexus.app/overlay/<id>` in the cloud).
- Exposes business operations via qubit (JSON-RPC 2.0 over HTTP + WebSocket, with TypeScript clients generated from Rust handlers).
- Speaks obs-websocket natively via the Rust `obws` crate, no JS bridge.

The Rust core stays *pure functional* internally, the `nexus-core` crate holds the Decider, types, validation; the `nexus-server` crate holds the qubit handlers, adapters (auth, persistence, obs-control), and HTTP server. The hexagonal layering is enforced by the crate boundary: `nexus-server` depends on `nexus-core` (one way) and never the reverse.

For local deployment: bind to `127.0.0.1:<port>`, filesystem persistence in the OS app-data directory, no-op auth (single tenant, the user who launched the binary).

For cloud deployment (deferred but designed-for): bind to `0.0.0.0:<port>`, database persistence (Postgres or SQLite via `sqlx`), real auth via OAuth or session tokens, multi-tenancy through workspace ownership. **Same binary, environment-configured.**

### Consequences

- **Good**, because workspace state has one source of truth (the Rust server), eliminating editor↔overlay sync bugs.
- **Good**, because obs-websocket is spoken in Rust via `obws`, no browser-side CORS, no JS auth challenges, full protocol support.
- **Good**, because qubit gives us type-safe RPC for free; Rust types → TS types → wire format → back, all derived.
- **Good**, because filesystem persistence is meaningfully better than localStorage: no 5–10 MB limit, importable/exportable JSON files, project files visible to the streamer.
- **Good**, because the same code targets both local and cloud, v2's hosted offering is "swap two adapters and add a Dockerfile."
- **Good**, because Rust enforces our discipline (sum types, exhaustive matching, ownership) more strictly than TypeScript; the Decider lives natively.
- **Good**, because subscriptions over WebSocket replace BroadcastChannel, and unlike BroadcastChannel, subscriptions work across machines (relevant for cloud).
- **Good**, because contributors and non-technical streamers can run the full system locally without any account.
- **Bad**, because deployment changes: instead of "open this URL in OBS," streamers download and run a binary. Higher friction than a hosted SPA but lower than a desktop app like Streamlabs Desktop.
- **Bad**, because the build pipeline is more complex than a pure-TS SPA: Cargo + pnpm + qubit type-gen + SvelteKit build + static-asset embedding.
- **Bad**, because cross-platform binary distribution (Windows / macOS / Linux) is real work; auto-updates are real work; signing is real work.
- **Bad**, because the dev loop has a Rust recompile step (mitigated by `cargo watch` + `bacon` for fast feedback).
- **Neutral**, because the FSD organization and the seven architectural rules from the spec carry over unchanged; only the artifacts they describe move from "WASM module" / "static SPA" to "Rust binary" / "served SPA."

### Confirmation

When implementation lands:
- The Rust core compiles without depending on any environment-specific code (no filesystem calls, no network calls, no clock reads, everything via injected ports).
- The `nexus-core` crate has zero dependencies on `nexus-server` (one-way crate dependency confirmed by `cargo tree`).
- A test suite runs `nexus-core` purely (no I/O), then a separate suite runs `nexus-server` with mocked adapters, then E2E tests run the full binary.
- Local mode: binary launches, opens default browser to the editor, persists workspace files to `~/.local/share/nexus/` (or platform equivalent), uses `obws` to talk to OBS.
- Cloud mode (v2, deferred): same binary, run with `NEXUS_MODE=cloud` + database connection string + auth provider config; serves on `0.0.0.0:8080` behind a reverse proxy.
- OBS Browser Source URL: `http://127.0.0.1:<port>/overlay?layout=<id>` locally, `https://<domain>/overlay/<id>` in the cloud.

## Pros and Cons of the Options

### A. Rust-as-WASM, no server, browser-only

- **Good**, because no binary to install, just a URL.
- **Good**, because no cross-platform packaging burden.
- **Bad**, because WASM/JS FFI is awkward for stateful long-lived modules.
- **Bad**, because localStorage is the only persistence; no cloud sync without a new architecture.
- **Bad**, because obs-websocket must be spoken from JS, with CORS, auth, and bridge code.
- **Bad**, because v2 cloud offering requires inventing a backend from scratch.
- **Bad**, because editor↔overlay sync requires BroadcastChannel + localStorage, with race conditions.

### B. Rust-as-local-server with qubit RPC

- **Good**, because workspace state has a single authoritative location.
- **Good**, because obs-websocket is native via `obws`, no bridge code.
- **Good**, because filesystem persistence is robust, large-capacity, exportable.
- **Good**, because same codebase scales to cloud deployment.
- **Good**, because qubit provides type-safe RPC with generated TypeScript clients.
- **Bad**, because streamer must install a binary.
- **Bad**, because build pipeline complexity is higher.
- **Bad**, because dev loop has Rust recompile step.

### C. SvelteKit + TS-only frontend; Rust backend in v2

- **Good**, because v1 ships fast.
- **Bad**, because v2 requires inventing a backend and migrating state out of browser storage, significant rework.
- **Bad**, because the v1→v2 transition is a hard cliff; some users on v1 forever, fragmented codebase.
- **Bad**, because no Rust learning during v1; team is junior to Rust by v2 time.

### D. Tauri desktop app

- **Good**, because Tauri handles cross-platform packaging and auto-update natively.
- **Good**, because Rust as an embedded process has zero network surface.
- **Bad**, because Tauri's IPC isn't qubit; we'd lose the type-safe RPC story.
- **Bad**, because Tauri doesn't translate to a cloud-hosted multi-tenant deployment.
- **Bad**, because OBS Browser Source can't load a Tauri-embedded window, we'd still need a separate HTTP server for the overlay URL.
- **Neutral**, because Tauri *as a packaging layer over option B* is actually useful (later) for distribution, it can wrap the Rust server + bundled SvelteKit assets into a clean installer. That's a packaging decision, not a fundamental architecture change.

## More Information

- Qubit RPC framework: [github.com/andogq/qubit](https://github.com/andogq/qubit), Rust↔TypeScript RPC over JSON-RPC 2.0, with type generation from Rust to TS via `ts-rs`, built on `jsonrpsee`.
- `obws` (obs-websocket client in Rust): the natural Rust client; supports obs-websocket v5 protocol fully.
- `axum` (web framework) and `tokio` (async runtime): standard choices for the qubit server's HTTP/WebSocket transport.
- `serde` (serialization) + `tsify` (TS type generation from Rust): underpin the type-shared contracts.
- This ADR partially fulfills [ADR 0000](0000-record-architecture-decisions.md) backlog items #26 (FSD adoption, still applies, just within the new architecture) and #27 (v2 HTTP API design philosophy, qubit is the choice; the philosophy is "type-safe RPC over JSON-RPC 2.0, not REST, not GraphQL").
- Tauri (deferred): when binary distribution matures, Tauri is the natural packaging shell, it bundles the Rust server + SvelteKit static assets into platform-specific installers without changing the architecture.

### Cloud-mode design implications (deferred to a future ADR)

This ADR commits to *the architecture supporting* cloud deployment, not the specific cloud deployment itself. Decisions deferred to a future ADR when cloud becomes active:

- Auth provider (OAuth via Twitch / Google / GitHub? Magic links? Sessions vs. JWTs?)
- Database choice (Postgres for multi-tenancy, SQLite for hobbyist self-hosting, both?)
- Hosting (Fly.io? Cloudflare Workers? Railway? Self-rolled?)
- Pricing model for the hosted offering.
- Free-tier vs. paid-tier feature gating.
- Migration path: local-only workspaces → cloud-synced workspaces (probably an import endpoint).
- Cloud-only features: collaboration, team workspaces, audit logs, automated backups.

### Foundational implications applied now

- **Persistence is a trait, not a concrete type.** `nexus-core` defines the trait; `nexus-server` provides `FilePersistence` (local) and stubs out `DatabasePersistence` (cloud, deferred).
- **Auth is a port from day one.** Local adapter is a no-op (single tenant, authenticated by virtue of process ownership). Cloud adapter is deferred. The handler signatures already accept an `Auth` context.
- **Workspace ownership is in the data model from day one.** `Workspace.ownerId: Option<UserId>`, `None` locally, `Some(...)` in cloud. The Decider doesn't care; the persistence layer filters by owner in cloud mode.
- **Configuration is environment-variable-first.** `NEXUS_MODE=local|cloud`, `NEXUS_BIND_ADDR`, `NEXUS_DATA_DIR`, `NEXUS_DATABASE_URL`. Standard 12-factor.
