---
status: "accepted"
date: 2026-05-28
decision-makers: project owner
consulted:
informed: future contributors
---

# Offline-collaborative local-first architecture on Loro CRDT with a trusted sync relay

## Context and Problem Statement

[ADR-0002](0002-local-or-cloud-rust-core-with-qubit-rpc.md) committed Nexus to a server-authoritative architecture: a pure Rust core (the Chassaing Decider, `decide`/`evolve`) owns workspace state as the single source of truth, the SvelteKit UI is a subscriber that mutates only through qubit RPC, `@tanstack/svelte-query` caches server state, and collaboration was explicitly deferred to a future cloud version.

A new hard requirement overrides that deferral: **multiplayer is a must, and it must work offline-collaboratively**, multiple people edit the same workspace, including while disconnected, and their divergent edits merge automatically on reconnect.

Offline-collaborative editing is incompatible with a single authority that validates every command before it applies: an offline client cannot round-trip to a server, so edits must apply locally and immediately, and the client becomes a smart local-first replica rather than a dumb subscriber. The hard sub-problem: Conflict-free Replicated Data Types (CRDTs) guarantee that replicas *converge*, but not that the converged state satisfies the workspace's *business invariants* (no widget on an archived layout, a valid active scene, unique identity). The question this ADR answers: which architecture delivers offline-collaborative multiplayer while preserving Nexus's domain invariants and its server-side product needs (rendering `/overlay` for OBS, speaking obs-websocket, running chat/event sources)?

## Decision Drivers

* Offline-collaborative multiplayer is now a primary requirement, not a deferred v2 feature.
* Domain invariants must survive concurrent and offline merges (the CRDT-convergence-vs-invariants tension).
* The server must remain able to read plaintext workspace state, `/overlay` rendering, obs-websocket, and chat sources all live server-side per ADR-0002.
* Preserve what still works from ADR-0002: a Rust core, the local-or-cloud "same binary, swap adapters" deployment, `obws`-native OBS control, hexagonal ports/adapters.
* Prefer mature, production-ready building blocks; ground library and pattern choices in current (May 2026) evidence, not assumption.
* Keep the streamer's local-first, low-friction default (works on one machine, offline).

## Considered Options

* **A. Keep the server-authoritative Decider; add online-only real-time collaboration** (Figma-style op broadcast).
* **B. Local-first CRDT replicas (Loro) + a trusted Rust sync relay** ← chosen
* **C. E2EE local-first with a blind relay** (Automerge + Keyhive/Beelay).
* **D. Adopt a turnkey server-authoritative sync engine** (Rocicorp Zero, ElectricSQL, ...).
* Substrate sub-decision: **Loro** vs Automerge vs Yjs (yrs).

## Decision Outcome

Chosen option: **"B. Local-first CRDT replicas on Loro 1.0 + a trusted Rust sync relay,"** because it is the only option that delivers genuine offline-collaborative editing (A cannot) while keeping the server able to do its plaintext server-side work (C cannot), on mature, self-hostable, Rust-native pieces that fit the single-binary local-first design (D cannot).

**State splits into four layers:**

1. **Collaborative document**, a `LoroDoc` (layouts, scenes, widgets, themes, bindings, active pointers), persisted as a Loro binary snapshot, synced between replicas. The hierarchy is a Loro `MovableTree`; per-node scalars live in each node's `get_meta()` LWW `LoroMap`; ordered collections are `MovableList`.
2. **Server runtime state**, `ObsConnection`, chat/event source status; per-instance, never in the document.
3. **Ephemeral presence**, cursors/selections/who-is-here, via Loro `EphemeralStore` (JS-side; the relay forwards opaque presence frames, it is not a Rust type today).
4. **Per-user UI prefs**, editor light/dark, panel layout; local, never shared.

**The trusted relay** (the Rust `nexus-server`) holds a canonical `LoroDoc` replica, imports peer updates, runs invariant **validate/repair**, persists, and rebroadcasts. It also hosts `/overlay`, speaks obs-websocket, and runs chat/event sources, all of which require plaintext, which is why the relay is trusted rather than a blind E2EE relay. It is the same binary deployed locally (solo/LAN) or hosted (remote collaboration), selected by env config, the ADR-0002 adapter philosophy preserved.

**Invariants via explicit consistency:** invariants are classified weak (LWW scalars, list ops, conflict-free identity) / repairable (clamp out-of-bounds, drop orphan refs) / strong (archived-layout-no-edits, resolved by precedence). The relay validates the post-merge state and emits a corrective Loro op for violations, which propagates to all replicas.

**Preserved from ADR-0002:** the Rust core/shell split (`nexus-core` pure, `nexus-server` shell; one-way crate dependency), the single binary, local-or-cloud via env config, `obws`-native OBS, hexagonal ports/adapters, and RFC 9457 at the HTTP boundary ([ADR-0001](0001-use-rfc-9457-problem-details-for-http-errors.md)).

**Changed from ADR-0002:** the Rust server is no longer the single source of truth (replicas are co-equal; the relay is the trusted merge/validate/persist hub); qubit is demoted from the document data-plane to a thin control-plane RPC (auth, obs-control, workspace listing); the document data-plane is Loro's own binary sync over WebSocket; `@tanstack/svelte-query` is dropped for document state (the local `LoroDoc` is the reactive store; Svelte runes subscribe to Loro diffs).

**Rule reinterpretations** (the spec §15 / `CLAUDE.md` "nine non-negotiable rules"):

* **Rule #2** ("core has no ... storage"): relaxed in letter. `nexus-core` depends on the `loro` crate (in-memory CRDT state, not storage) plus `serde`/`thiserror`. It still has no IO, async runtime, network, wall-clock, or randomness; time and randomness still arrive via `Clock`/`Random` ports; `cargo tree` still proves no dependency on `nexus-server`.
* **Rule #3** ("state mutates only through Command -> Decide -> Events -> Evolve; undo is event-log reversal"): superseded. CRDT merge is a first-class mutation path; undo is Loro's `UndoManager`.
* **Rule #4** ("`decide` pure/total, `evolve` mechanical"): reinterpreted. `decide` becomes a pure validator/repair function over the read model; there is no `evolve`/event log to fold, Loro is both the state and the operation log.
* **Rule #8** ("immutable end-to-end, no DTOs"): reframed. The `LoroDoc` is the mutable collaborative state by design; the immutability discipline applies to the plain read-model structs and to snapshots/messages crossing boundaries.
* Rules #1, #5, #6, #7, #9 stand (with `ObsConnection`'s typestate now living in server runtime state, not the document).

**Substrate = Loro 1.0** (current Rust line `loro = "1.10"`; the browser uses the wire-compatible `loro-crdt` npm package). Chosen over Automerge (whose batteries, automerge-repo + Keyhive, lost their edge once a trusted relay was chosen, and whose JSON model fits the tree less directly than Loro's native `MovableTree`) and Yjs/yrs (JS-first; the Rust port is secondary to a Rust-core-first design).

### Consequences

* **Good**, offline-collaborative editing works: local edits apply instantly, merge on reconnect, no server round-trip required to edit.
* **Good**, the server keeps full plaintext capability, so `/overlay` rendering, `obws`, and chat sources stay server-side as designed.
* **Good**, invariants are preserved by an explicit-consistency validator at the trusted relay; the convergence-vs-invariants tension is resolved by classification + repair, not abandoned.
* **Good**, much of the Decider's conceptual investment survives: Loro's op-log is the event log, `decide` becomes the validator, draft/live mode maps onto Loro branches, undo onto `UndoManager`.
* **Good**, fewer client moving parts: the local `LoroDoc` replaces TanStack Query's cache + optimistic-update machinery for document state.
* **Bad**, it reinterprets four of the "non-negotiable" rules and supersedes the core of ADR-0002; the design spec and `CLAUDE.md` need reconciling (this ADR anchors that reconciliation).
* **Bad**, the client is now a smart replica carrying a CRDT (WASM) rather than a thin subscriber, more client complexity and a WASM dependency.
* **Bad**, presence has no Rust `EphemeralStore` today (Loro's is JS-only), so server-originated presence needs a different mechanism if ever required.
* **Neutral**, qubit remains in the stack but only as a future control-plane RPC; it is not used by the first increments.

### Confirmation

* A walking-skeleton increment proves the full offline-collaborative loop on one operation (scene activation across two replicas): `cargo test -p nexus-core` (validator + LWW convergence), `cargo test -p nexus-server` (two-replica sync, offline replay, repair propagation, persistence round-trip), and one Playwright two-tab e2e.
* `cargo tree` confirms `nexus-core` does not depend on `nexus-server` (the hexagonal seam survives the Loro adoption).
* The existing app gates stay green: `pnpm -F app check`, `lint`, `build`.

## Pros and Cons of the Options

### A. Server-authoritative Decider + online-only collaboration

* Good, smallest change; the Decider and ADR-0002 mostly stand; Figma proves the model at scale.
* Good, invariants stay centrally enforced.
* Bad, **does not deliver offline-collaborative editing**, the stated hard requirement (edits need a live authority).

### B. Local-first CRDT (Loro) + trusted relay

* Good, genuine offline-collaborative editing; replicas converge by construction.
* Good, the trusted relay keeps server-side plaintext work (overlay / obs / chat) intact.
* Good, Rust-native (Loro), self-hostable, single-binary, fits the local-first default.
* Neutral, requires the explicit-consistency validator and reinterprets several rules.
* Bad, more client complexity (CRDT / WASM); presence is JS-only in Loro.

### C. E2EE local-first (Automerge + Keyhive / Beelay)

* Good, maximum privacy; truest local-first; purpose-built local-first auth.
* Bad, Keyhive/Beelay is **pre-alpha** ("do not use in production," unaudited) as of 2026.
* Bad, a blind encrypted relay **cannot** render `/overlay`, speak obs-websocket, or run chat sources, breaking Nexus's server-side product model.

### D. Turnkey server-authoritative sync engine (Zero, ElectricSQL, ...)

* Good, batteries included (sync protocol, caching, optimistic mutations, permissions).
* Bad, Zero is explicitly "not local-first," requires Postgres + a Node zero-cache + TypeScript-only clients; ElectricSQL/PowerSync are Postgres-replication. All fight the Rust + local-snapshot + single-binary design and none deliver offline-divergent merge the way a CRDT does.

### Substrate: Loro vs Automerge vs Yjs (yrs)

* **Loro (chosen)**, Rust-native, production-ready 1.0, `MovableTree` fits the workspace hierarchy, WASM for the browser, time-travel / branches / undo built in.
* Automerge, mature, multi-language, automerge-repo + Keyhive ecosystem; but the Keyhive advantage is moot under a trusted relay, and the JSON model fits the tree less directly.
* Yjs / yrs, fastest and most production-proven, but JS-first; the Rust port is secondary for a Rust-core-first server.

## More Information

* Supersedes the core of [ADR-0002](0002-local-or-cloud-rust-core-with-qubit-rpc.md) (server-as-single-source-of-truth; qubit as the document data-plane). Preserves ADR-0002's Rust core/shell split, single-binary local-or-cloud deployment, `obws`-native OBS, and hexagonal ports. Honors [ADR-0001](0001-use-rfc-9457-problem-details-for-http-errors.md) at the HTTP boundary. The editor aesthetic and shell ([ADR-0003](0003-dark-first-pro-creative-tool-editor-aesthetic.md), [ADR-0004](0004-responsive-editor-shell.md)) are unaffected.
* Research basis (May 2026): Figma's multiplayer is a per-document **server-authoritative** model (it rejected pure CRDTs and OT), which validates option A's mechanism but A cannot go offline; Rocicorp **Zero**'s "custom mutators" are structurally the Decider, but Zero is "not local-first," Postgres- and Node-bound (option D); CRDT **invariant preservation** is an established research area (**LoRe**, ACM TOPLAS; **Consistent Local-First Software / ConLoc**, IEEE TSE 2024) underpinning the explicit-consistency strategy; **Loro 1.0** is production-ready and Rust-native; **Keyhive/Beelay** (the E2EE option C) is pre-alpha and unaudited as of 2026.
* Deferred to later ADRs / increments: the presence protocol; draft/live via Loro branches (`fork`/`merge`/`checkout`); `UndoManager` undo; OAuth + actor identity in operations + per-workspace roles; cloud/Postgres persistence + multi-tenancy; the qubit control plane; and client-side shared validation (compiling a predicate-only slice of `nexus-core` to WASM).
