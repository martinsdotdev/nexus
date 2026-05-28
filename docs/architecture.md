# Nexus, Architecture Overview

**Status:** Scaffold (sections defined; content TBD).
**Last updated:** 2026-05-13
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

## 2. Container view (C4 Level 2)

TBD, the major deployable units inside Nexus. v1 containers:
- `nexus` binary (single Rust process serving HTTP, WebSocket, and qubit RPC)
- SvelteKit UI (built static, served by the `nexus` binary as static assets)
- OBS Studio (external; Browser Source loads `/overlay`)
- Filesystem (local workspace JSON files in OS app-data dir)

Containers added in later versions:
- (v2) Postgres database, replaces filesystem in cloud mode
- (v2) OAuth provider, Twitch / Google / GitHub
- (v3) Tauri shell, wraps `nexus` binary + SvelteKit assets into platform installer

Diagram TBD.

## 3. Component view (C4 Level 3)

TBD, the components inside each container.

`nexus-server` (Rust crate inside the binary):
- HTTP/qubit handlers (axum + qubit + jsonrpsee)
- Adapters (`FilePersistence`, `ObwsAdapter`, `MockChatSource`, `MockEventSource`, `NoOpAuth`, `SystemClock`, `OsRandom`)
- Composition root (`main.rs`, wires ports to adapters by env-var-driven mode)
- Static asset serving (`tower-http::services::ServeDir` + `include_dir`)

`nexus-core` (Rust crate, dependency-free hexagonal core):
- Domain types (`Workspace`, `Layout`, `Scene`, `WidgetInstance`, `Theme`, `ChatBinding`, `ObsConnection`)
- Decider triple (`decide`, `evolve`, `isTerminal`) per Chassaing
- Ports as traits (`Persistence`, `Auth`, `ObsControl`, `ChatSource`, `EventSource`, `Clock`, `Random`)

SvelteKit UI:
- FSD layers, `app/` → `pages/` (filesystem routing) → `widgets/` (overlay + editor chrome) → `features/` (compose-widget, snap-to-edge, theming, ...) → `entities/` (thin TS wrappers around qubit-generated types) → `shared/` (UI primitives, tokens, lib)
- TanStack Query cache + qubit subscription bridge (server-state lives in cache; live updates from server flow in via `setQueryData`)

## 4. Key invariants

The nine non-negotiable rules from spec §15. Brief restatement:

1. Shell calls core. Core does not call shell.
2. Core has no clock, no randomness, no I/O, no DOM, no storage.
3. State mutates only through Command → Decide → Events → Evolve.
4. `decide` is pure and total. `evolve` is mechanical.
5. Fallible operations at port boundaries return `Result<T, E>`. No throws across ports.
6. Aggregate lifecycles with ≥3 gated states are discriminated unions (Rust `enum`).
7. Bounded contexts publish events; they do not import each other.
8. All types are `readonly` end-to-end (TS) / immutable by default (Rust). No DTOs.
9. Editor animations are restrained (≤ 300 ms, `ease-out`). Overlay-runtime animations earn their length by being rare and communicative.

Full text and rationale: spec §15 and §2.

## 5. Key technology choices

Locked stack. Pointer to spec §16 for the full table. Headline:
- **Core:** Rust 1.84+ (2024 edition), Cargo workspace, axum, qubit (RPC), tokio, serde, `obws` (obs-websocket), sqlx (v2 cloud), `thiserror` + `anyhow` (errors).
- **UI:** SvelteKit 2.x + Svelte 5 runes, `@tanstack/svelte-query`, `@qubit-rs/client`, `lucide-svelte`, scoped `<style>` + CSS custom properties.
- **Toolchain:** Node 22 LTS, pnpm, cargo-dist (axodotdev), Vitest + Testing Library + Playwright, ESLint + `eslint-plugin-boundaries` + Prettier.

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
