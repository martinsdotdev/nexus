---
status: "accepted"
date: 2026-05-29
decision-makers: project owner
consulted:
informed: future contributors
---

# Pure data-driven themes (every theme is registry data; built-ins are seeded)

## Context and Problem Statement

[ADR-0006](0006-data-driven-custom-themes.md) introduced authored themes with a **hybrid** registry: the four built-ins stayed CSS `[data-theme="..."]` stylesheets, while custom themes lived as data in the Loro doc. That choice minimized churn to the just-shipped overlay, but it bought a two-universe system. With the editor and the registry now built, the hybrid's seams are visible: two apply paths in `resolveThemeStyle` (CSS cascade vs inline data), "edit a built-in" is a duplicate-then-edit dance, a browser-only `getComputedStyle` probe seeds duplicates (so it cannot run headless), and the same token values exist twice (the CSS files and whatever reads them), a drift surface.

The question, asked directly by the project owner after the editor shipped: do the built-ins need to be a separate CSS concept at all, or should every theme, built-in and custom alike, be the same kind of data? Polished first-run defaults are not in question (they are a product requirement); only the *form* of the built-ins is.

## Decision Drivers

* One uniform theme model: a single apply path, a single notion of "a theme."
* Built-ins editable in place (no duplicate-to-edit indirection), matching how a creative tool behaves.
* Remove accidental complexity: the second apply path, the browser-only seeding probe, and the two-copies drift surface ([ADR-0006](0006-data-driven-custom-themes.md) flagged both as costs).
* Preserve offline-collaborative safety ([ADR-0005](0005-offline-collaborative-loro-crdt-trusted-relay.md), trap T1): seeding canonical themes must not collide on merge.
* Keep polished first-run defaults and never render a scene unstyled, even for a partial or imported theme.
* Weigh against the costs: the built-in values move out of CSS (their natural medium) into a seed table, the relay's snapshot grows, and built-ins become mutable data that must be protected from deletion.

## Considered Options

* **A. Keep the hybrid registry** (built-ins CSS, custom themes data), per [ADR-0006](0006-data-driven-custom-themes.md).
* **B. Pure data-driven**, seed the four built-ins into the registry as data; drop the CSS theme files; one model ← chosen

## Decision Outcome

Chosen option: **"B. Pure data-driven."** This supersedes [ADR-0006](0006-data-driven-custom-themes.md) and, with it, the spec §8 v1 "four fixed themes" constraint.

* **Seeded built-ins.** The relay seeds the four built-ins into the root `"themes"` registry at first run ([`nexus-core::default_doc`](../../crates/nexus-core/src/default_doc.rs)), each a `protected` entry with the full 33-token vocabulary and `base = ""` (they stand alone). The values are a Rust const table ([`nexus-core::builtin_themes`](../../crates/nexus-core/src/builtin_themes.rs)) ported once from the former CSS files, which are deleted, so there is now a single source of truth.
* **Stable built-in ids are safe.** Built-ins keep stable string ids (`cozy`/`cyber`/`editorial`/`sticker`) rather than minted ids, because only the relay ever seeds them, once (trap T1 concerns client-side construction). User themes keep minted `theme-<uuid>` ids.
* **Protected, not read-only.** Built-ins are editable in place (every theme is data); they are `protected` so the builder hides Delete and the `deleteTheme` mutation refuses them, guaranteeing the default fallback (`cozy`) always exists. Two further invariants keep a protected theme a sound inheritance floor: it holds **only literal token values** (`setThemeToken` refuses a `link:` value on a protected theme, and the builder hides the link control for them), and a theme's **`base` may only reference a protected built-in** (`createTheme` sanitizes any other base to `""`). Together these make the default floor and any `base` layer always fully resolvable, and keep `base` single-level and cycle-free.
* **One apply path.** `resolveThemeStyle(themeId, themes, accentOverride)` returns `{ inlineVars }` only (no `data-theme`). It merges the default theme (a safety floor), the selected theme's `base`, then the selected theme, resolves token links over the union (cycle/dangling-safe), and falls back to the default theme's literal for any token whose winning value is a broken link. Because the floor and `base` are protected built-ins (literals only, per above), **every one of the 33 tokens is always emitted**, a partial or imported theme never renders unstyled, and the overlay never inherits the editor chrome's identically-named `:root` tokens. The accent override replaces the resolved accent in place (emitted once). The overlay and the editor canvas carry the result inline; there is no CSS cascade to load.
* **Validator over the registry.** The repairable invariant becomes "a scene's `themeId` names a theme in the registry"; the hardcoded built-in list is gone (the registry is the source). A dangling reference is still repaired to the default by the relay validator.

### Consequences

* **Good**, one model and one apply path; built-ins are editable in place; the duplicate-to-edit dance and the browser-only `getComputedStyle` probe are gone (duplicate copies registry data, runnable headless).
* **Good**, a single source of truth for built-in values (the seed table), eliminating the CSS-vs-reader drift surface; the validator no longer hardcodes the built-in set.
* **Good**, the resolver's default-floor + per-token fallback make unstyled rendering structurally impossible, the role the base CSS used to play, now covered without CSS.
* **Bad**, the built-in values live as a Rust const table rather than CSS (less natural as a styling medium, edited as data); and built-ins are now mutable, requiring the `protected` flag to keep the default fallback alive.
* **Neutral**, the relay's snapshot grows by the seeded themes (~4 × 33 token strings, a few KB, shipped once per connect); negligible for the local-first single-machine target and acceptable for v2 cloud.
* **Neutral**, the read model's `customThemes` becomes `themes` (it holds built-ins too) and the `CustomTheme` type becomes `Theme` with a `protected` field.

### Confirmation

* `cargo test -p nexus-core` (the registry is seeded with four protected, 33-token built-ins; the scene-theme invariant checks the registry; a dangling `themeId` repairs to the default) and `cargo test -p nexus-server` (a peer's dangling `themeId` is repaired and rebroadcast).
* `vitest` headless: theme CRUD with the `protected` flag, `deleteTheme` refusing a protected theme, and `resolveThemeStyle` unit tests for inline output, base inheritance, the default floor, link resolution, cycle drop, and dangling fallback.
* A browser test for the theme builder (in-place edit, duplicate, link a token, Delete hidden for protected) and an e2e proving a theme authored in `/edit` reaches `/overlay` via inline vars.

## Pros and Cons of the Options

### A. Keep the hybrid registry (ADR-0006)

* Good, zero further churn; built-ins immutable and zero-sync by construction; CSS is theme's natural medium.
* Bad, two apply paths, a browser-only seeding probe, duplicate-to-edit indirection, and a CSS-vs-data drift surface, all accidental complexity once a full registry exists.

### B. Pure data-driven (chosen)

* Good, one uniform model and apply path; built-ins editable in place; no probe; single source of truth.
* Bad, built-in values move from CSS into a seed table; built-ins become mutable (needing a protected flag); the synced snapshot grows by a few KB.

## More Information

Builds on [ADR-0005](0005-offline-collaborative-loro-crdt-trusted-relay.md) (the collaborative `LoroDoc` + trusted relay seed the built-ins, and the registry invariant uses the same validate/repair mechanism). Supersedes [ADR-0006](0006-data-driven-custom-themes.md): its hybrid registry, duplicate-to-edit probe, and dual apply paths are replaced; the reserved `link:<token>` sentinel, the minted ids for user themes, and the editor-vs-overlay token-universe isolation it established all carry forward unchanged. The editor aesthetic ([ADR-0003](0003-dark-first-pro-creative-tool-editor-aesthetic.md)) is unaffected. Deferred: a "reset built-in to default" affordance (now that built-ins are editable in place), a curated palette/ramp generator, and per-token color-syntax validation.
