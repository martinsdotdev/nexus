---
status: "accepted"
date: 2026-05-29
decision-makers: project owner
consulted:
informed: future contributors
---

# Data-driven custom themes with a hybrid registry and token linking

## Context and Problem Statement

The design spec (§8) ships v1 with **four fixed themes** (Cozy, Cyber, Editorial, Sticker), each a static CSS file, plus a per-scene `overridesAccent` / `overridesDensity` escape hatch. The editor build adds a Zed-theme-builder-style theme editor: the streamer creates, duplicates, names, edits (the full ~33-token overlay set), imports, and exports themes, and links one token to another (a cascade). The fixed-four model cannot express any of that, a theme must become editable *data*, not a static stylesheet.

The question: where do custom themes live so they (a) sync offline-collaboratively like the rest of the workspace, (b) preserve the workspace's invariants under merge, (c) do not force a costly rewrite of the just-shipped overlay theming, and (d) keep the editor's own chrome tokens isolated from the overlay theme tokens?

## Decision Drivers

* Full token-level authoring with live preview (Zed parity), including create / duplicate / rename / delete / import / export and token linking.
* Offline-collaborative safety: custom themes must merge without identity collisions ([ADR-0005](0005-offline-collaborative-loro-crdt-trusted-relay.md), trap T1).
* Minimal churn to the working, tested overlay theming (the four CSS themes shipped and are covered by tests + e2e).
* Bounded sync cost: the relay ships the workspace snapshot to every replica on connect.
* The editor chrome (its own dark token universe) must never adopt overlay theme tokens, and vice versa.

## Considered Options

* **A. Spec-aligned**, keep four fixed CSS themes + per-scene accent/density override only.
* **B. Hybrid registry**, built-ins stay CSS; custom themes are editable data in the Loro doc ← chosen
* **C. Pure data-driven**, port all four built-ins into the doc as data; drop the CSS theme files.

## Decision Outcome

Chosen option: **"B. Hybrid registry."** This supersedes the spec §8 v1 "four fixed themes" constraint.

* **Registry**, a new root `LoroMap` `"themes"` ([`schema::THEMES`](../../crates/nexus-core/src/schema.rs)), keyed by a **client-minted unique id** (`theme-<uuid>`, never the name, trap T1), each value a nested map `{ name, base, tokens }` where `tokens` is a `{ tokenName -> value }` map. It is part of the collaborative `LoroDoc`, so it merges, persists, and rebroadcasts through the existing relay pipeline with no new sync machinery.
* **Built-ins stay CSS.** The four themes remain `[data-theme="..."]` stylesheets (zero snapshot cost, exact computed values). A scene's `themeId` may be a built-in id (string) or a registry id.
* **Duplicate-to-edit.** "Editing a built-in" duplicates it into a custom theme: its 33 token values are read exactly via `getComputedStyle` of a hidden `<div data-theme="...">` probe, then stored as data. Built-ins themselves stay read-only.
* **Runtime application.** A single pure helper, `resolveThemeStyle(themeId, customThemes, accentOverride)`, maps a scene to `{ dataTheme, inlineVars }`: a built-in resolves to `data-theme` only; a custom theme resolves to `data-theme = base` (the fallback layer for any unset token) plus an inline `--token: value;` string. The overlay and the editor canvas both use it, the same path the overlay already used for `--accent`.
* **Token linking.** A token value may be a literal or a `link:<token>` reference. Links resolve inside `resolveThemeStyle` with cycle and dangling detection; a cyclic or dangling token is dropped so the base layer still covers it. The format is a reserved value sentinel, so it needs no schema change.
* **Invariant.** A new repairable core invariant: a scene's `themeId` must name a built-in or a registered custom theme; a dangling reference (e.g. a deleted theme) is repaired to the default (`cozy`) by the relay validator (`validate` / `apply_repairs`, [ADR-0005](0005-offline-collaborative-loro-crdt-trusted-relay.md)'s explicit-consistency mechanism). The client is independently defensive (it falls back when rendering), so a dangling ref never breaks the overlay even before the repair propagates.

### Consequences

* **Good**, full Zed-style authoring with live preview, offline-collaborative by construction, on the existing CRDT + relay.
* **Good**, minimal churn: the shipped CSS themes are untouched; only authored themes pay a (small, bounded) snapshot cost.
* **Good**, identity-collision-safe under merge (minted ids, never names); two themes named the same coexist.
* **Good**, editor chrome and overlay theme token universes stay isolated (the registry only feeds the overlay/canvas subtree).
* **Bad**, two theme code paths (built-in CSS vs custom inline data) the apply helper must reconcile, and "edit a built-in" is duplicate-then-edit, not in-place.
* **Bad**, the duplicate-to-edit probe is browser-only (`getComputedStyle`), so seeding a duplicate cannot run headless.
* **Neutral**, the validator gains a second repair kind (`Repair` becomes an enum: `SetActiveScene` | `SetSceneTheme`).

### Confirmation

* `cargo test -p nexus-core` (theme-reference invariant + dangling-theme repair) and `cargo test -p nexus-server` (a peer's dangling `themeId` is repaired and rebroadcast).
* `vitest` headless: theme CRUD round-trip, same-name distinct-id (collision safety), and a custom theme resolving to inline vars end to end, plus `resolveThemeStyle` unit tests for built-in / custom / accent-override / link cascade / cycle fallback.
* A browser test for the theme builder (duplicate, edit a token live, link a token, export/import) and an e2e proving a custom theme set in `/edit` renders on `/overlay`.

## Pros and Cons of the Options

### A. Spec-aligned (four fixed CSS themes + accent/density override)

* Good, zero new machinery; fully spec-compliant; smallest snapshot.
* Bad, **cannot author themes** at all, which is the feature being built.

### B. Hybrid registry (chosen)

* Good, full authoring with minimal churn and bounded snapshot cost; offline-safe; token universes stay isolated.
* Neutral, two apply paths; a second repair kind.
* Bad, "edit a built-in" is duplicate-to-edit; the seeding probe is browser-only.

### C. Pure data-driven (all themes as doc data)

* Good, one uniform model; built-ins editable in place.
* Bad, rewrites the working overlay theming, drops the four tested CSS files, and inflates the snapshot the relay ships to every replica with ~130 token strings the app already has as static CSS.

## More Information

Builds on [ADR-0005](0005-offline-collaborative-loro-crdt-trusted-relay.md) (the collaborative `LoroDoc` + trusted relay; the registry is a new root container synced by the same pipeline, and the new invariant uses the same validate/repair mechanism). The editor aesthetic ([ADR-0003](0003-dark-first-pro-creative-tool-editor-aesthetic.md)) is unaffected, the editor chrome keeps its own token universe; only the overlay/canvas consume the registry. Supersedes the spec §8 v1 "four fixed themes" constraint. Deferred: a curated palette/ramp generator, theme sharing/registry beyond local import/export, and per-token validation of color syntax.
