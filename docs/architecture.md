# Nexus, Architecture Overview

**Status:** Scaffold (sections defined; content TBD).
**Last updated:** 2026-05-28
**Architecture revised by [ADR-0005](decisions/0005-offline-collaborative-loro-crdt-trusted-relay.md):** Nexus is now local-first and offline-collaborative on Loro CRDT replicas with a trusted sync relay; the Rust server is no longer the single source of truth. The component bullets below reflect this; ADR-0002's superseded parts remain as history.
**Related:** spec at [`docs/superpowers/specs/2026-05-13-stream-overlay-editor-design.md`](superpowers/specs/2026-05-13-stream-overlay-editor-design.md), ADRs at [`docs/decisions/`](decisions/)

A short (~2-page) architectural entry point. Defers to the spec for depth.

Uses [C4 model](https://c4model.com/) conventions: Context (L1), Container (L2), Component (L3). Code-level (L4) is deferred to actual source code.

## 1. System context (C4 Level 1)

TBD, diagram showing the people and external systems Nexus interacts with:
- The streamer
- OBS Studio (loads the overlay route as a Browser Source; communicates with the Nexus binary via obs-websocket v5)
- The Nexus binary (local mode) or Nexus cloud service (v2)
- Streaming platforms (v2: Twitch, YouTube, Kick, for chat and event sources)
- The viewer (indirectly, sees the overlay rendered through OBS to the stream)
- Collaborators (other editors syncing the same workspace through the relay, per [ADR-0005](decisions/0005-offline-collaborative-loro-crdt-trusted-relay.md))

## 2. Container view (C4 Level 2)

TBD, the major deployable units inside Nexus. v1 containers:
- `nexus` binary (single Rust process: trusted sync relay over WebSocket, control-plane qubit RPC, HTTP static serving)
- SvelteKit UI (built static, served by the `nexus` binary; runs a local Loro replica in the browser)
- OBS Studio (external; Browser Source loads `/overlay`)
- Filesystem (local workspace Loro snapshot in OS app-data dir)

Containers added in later versions:
- (v2) Postgres database, replaces the filesystem snapshot in cloud mode
- (v2) OAuth provider, Twitch / Google / GitHub
- (v3) Tauri shell, wraps `nexus` binary + SvelteKit assets into platform installer

Diagram TBD.

## 3. Component view (C4 Level 3)

TBD, the components inside each container.

`nexus-server` (Rust crate inside the binary), the **trusted sync relay**:
- WebSocket sync endpoint carrying Loro document updates; holds the canonical `LoroDoc` replica, validates/repairs the merged state, persists, and rebroadcasts to peers
- Control-plane qubit handlers (auth, obs-control, workspace listing), **not** the document data-plane
- Adapters (`FilePersistence` for the Loro snapshot, `ObwsAdapter`, `MockChatSource`, `MockEventSource`, `NoOpAuth`, `SystemClock`, `OsRandom`)
- Composition root (`main.rs`, wires ports to adapters by env-var-driven mode; the same binary runs locally or hosted)
- Static asset serving (`tower-http::services::ServeDir` + `include_dir`)

`nexus-core` (Rust crate, hexagonal core; depends on `loro` for the CRDT document, otherwise IO-free, no clock/randomness/network):
- The Loro-backed workspace document schema + plain read-model types (`Workspace`, `Layout`, `Scene`, `WidgetInstance`, ...)
- A validator/repair function over the document (the morphed `decide`; there is no `evolve`/event log, Loro is both the state and the op-log), see [ADR-0005](decisions/0005-offline-collaborative-loro-crdt-trusted-relay.md)
- Ports as traits (`Persistence`, `Auth`, `ObsControl`, `ChatSource`, `EventSource`, `Clock`, `Random`)
- `ObsConnection` and other session typestate live in `nexus-server` runtime state, not in the document

SvelteKit UI (a **local-first Loro replica**):
- FSD layers, `app/` → `pages/` (filesystem routing) → `widgets/` (overlay + editor chrome) → `features/` (compose-widget, snap-to-edge, theming, ...) → `entities/` → `shared/` (UI primitives, tokens, the `crdt/` client)
- The local `LoroDoc` **is** the reactive store: edits apply locally (offline-capable) and Svelte runes subscribe to Loro diffs; no TanStack Query for document state
- qubit is a thin control-plane client only (auth, obs-control, workspace listing)

## 4. Key invariants

The nine rules from spec §15. [ADR-0005](decisions/0005-offline-collaborative-loro-crdt-trusted-relay.md) reinterprets rules 2, 3, 4, and 8 for the Loro/CRDT model (annotated below):

1. Shell calls core. Core does not call shell.
2. Core has no clock, no randomness, no I/O, no DOM, no network. *(ADR-0005: `nexus-core` may depend on the `loro` crate, in-memory CRDT state, not storage; time/randomness still arrive via ports.)*
3. ~~State mutates only through Command → Decide → Events → Evolve.~~ *(ADR-0005: superseded, CRDT merge is a first-class mutation path; undo is Loro's `UndoManager`.)*
4. `decide` is pure and total. *(ADR-0005: `decide` is now a pure validator/repair function over the read model; there is no `evolve`/event log.)*
5. Fallible operations at port boundaries return `Result<T, E>`. No throws across ports.
6. Aggregate lifecycles with ≥3 gated states are discriminated unions (Rust `enum`).
7. Bounded contexts publish events; they do not import each other.
8. Immutable by default. *(ADR-0005: the `LoroDoc` is the mutable collaborative state; the discipline applies to plain read-model structs and to snapshots/messages crossing boundaries.)*
9. Editor animations are restrained (≤ 300 ms, `ease-out`). Overlay-runtime animations earn their length by being rare and communicative.

Full text and rationale: spec §15 and §2; reinterpretations in ADR-0005.

## 5. Key technology choices

Locked stack. Pointer to spec §16 for the full table; [ADR-0005](decisions/0005-offline-collaborative-loro-crdt-trusted-relay.md) revises the data/sync layer. Headline:
- **Core:** Rust 1.84+ (2024 edition), Cargo workspace, **`loro` (CRDT document)**, axum, tokio, serde, `obws` (obs-websocket), sqlx (v2 cloud), `thiserror` + `anyhow` (errors). qubit (RPC) is retained for the **control plane only**, not the document data-plane.
- **UI:** SvelteKit 2.x + Svelte 5 runes, **`loro-crdt` (the local replica, the reactive store)**, `@qubit-rs/client` (control plane), `lucide-svelte`, scoped `<style>` + CSS custom properties. (`@tanstack/svelte-query` is no longer used for document state.)
- **Toolchain:** Node 22 LTS, pnpm, cargo-dist (axodotdev), Vitest + Testing Library + Playwright, ESLint + Prettier. (`eslint-plugin-boundaries` is referenced in the spec but is not yet installed; FSD layering is a convention until it is added.)

## 6. Where to find what

| Question | Source of truth |
|---|---|
| What does Nexus do? | [PRD](prd.md) |
| Why does the architecture look like this? | [ADR index](decisions/README.md) |
| How does X work in detail? | [Design spec](superpowers/specs/2026-05-13-stream-overlay-editor-design.md) |
| When does X land? | [Milestones](milestones.md) |
| How do agents work on this codebase? | [`CLAUDE.md`](../CLAUDE.md) |
| How is FSD layering enforced? | [`.agents/skills/feature-sliced-design/SKILL.md`](../.agents/skills/feature-sliced-design/SKILL.md) |
| What error format does the HTTP API use? | [ADR-0001](decisions/0001-use-rfc-9457-problem-details-for-http-errors.md) |
| Why local Rust binary instead of WASM-in-browser or cloud-only? | [ADR-0002](decisions/0002-local-or-cloud-rust-core-with-qubit-rpc.md) |
| Why offline-collaborative on a CRDT (Loro) instead of server-authoritative? | [ADR-0005](decisions/0005-offline-collaborative-loro-crdt-trusted-relay.md) |
