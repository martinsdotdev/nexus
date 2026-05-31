---
status: "accepted"
date: 2026-05-26
decision-makers: project owner
consulted:
informed: future contributors
---

# Adopt Ark UI (headless) for the editor component layer

## Context and Problem Statement

The `/edit` UI is built from hand-rolled primitives: a ~240-line `Drawer` with a bespoke focus trap and tab-cycling, a ~200-line `CommandPalette` with custom filter/keyboard logic, native `<select>`/`<input>`/`<details>` controls in the inspector and theme builder, and `aria-pressed` button groups for the tool rail and scene strip. They work and are token-styled, but the accessibility and interaction logic is re-implemented per component, lightly tested, and uneven (e.g. the theme builder's color tokens are edited as raw `oklch()` text).

The project owner asked to adopt a real headless component foundation. The question: which library integrates cleanly with Svelte 5 runes, `@sveltejs/adapter-static` prerendering, our scoped `<style>` + CSS-custom-property token system, and the dual token universes (editor chrome vs overlay theme), while keeping the `/overlay` (OBS) bundle lean since it never uses these controls?

## Decision Drivers

* Robust accessibility and interaction handled by audited state machines, not re-implemented per component.
* Keep the EXACT current visual design and the token system; the library must be headless.
* Svelte 5 (runes) support; SSR / prerender safety under adapter-static.
* Comprehensive coverage so the whole editor can move, including a visual Color Picker for the theme builder.
* The cost falls only on `/edit`; the `/overlay` bundle stays minimal.

## Considered Options

* **A. Keep hand-rolling** each primitive.
* **B. Ark UI** (`@ark-ui/svelte`, Zag.js state machines) ← chosen
* **C. Bits UI** (Svelte-native headless).
* **D. Melt UI** (Svelte-native low-level builders).

## Decision Outcome

Chosen option: **"B. Ark UI."**

* **`@ark-ui/svelte`** (Svelte 5 runes, stable). Built on framework-agnostic Zag.js machines; headless, styled via `[data-scope][data-part][data-state]` attributes, which fits our scoped styles + tokens exactly.
* **Wrapped behind `shared/ui` primitives.** New wrappers (`Select`, `Field`, `NumberInput`, `Switch`, `Collapsible`, `Tooltip`, `ColorField`, `ToggleGroup`) and rewritten internals of `Drawer`/`CommandPalette` expose plain single-value APIs (`value: string`, `onChange(v)`). Ark's `value: string[]`, details-object callbacks, portal anatomy, and `createListCollection` are contained in that one layer; the ~20 call sites in `features/` and `routes/edit/` stay dumb, and the FSD boundary holds.
* **Comprehensive scope.** Every mappable control becomes an Ark component, including the theme builder's color tokens, which become a **visual Color Picker with no raw oklch text field** (the project owner's choice).
* **oklch bridge via `culori`.** Tokens stay authored in oklch (the spec format; the built-ins are oklch), but Ark's Color Picker speaks rgba/hsla/hsba/hex, not oklch. A thin helper (`entities/theme/lib/oklch-color.ts`, using `culori` with per-function imports) converts oklch↔rgb at the picker boundary.
* **`/overlay` stays Ark-free.** Ark is imported only by `shared/ui` wrappers consumed by `/edit`; the overlay route (widgets + `resolveThemeStyle` only) must not pull Ark into its chunk, verified each build.

### Consequences

* **Good**, audited accessibility (real `dialog`/`listbox`/`combobox` semantics), far less custom interaction code, 45+ components incl. a Color Picker, and a headless model that reuses our tokens unchanged.
* **Good**, the wrapper layer makes the library swappable: a future change touches `shared/ui/`, not the call sites.
* **Bad**, a sizeable `@zag-js/*` dependency tree (borne only by `/edit`); Ark `Select`/`Combobox` are a button-trigger + portaled listbox, NOT a native `<select>`, so the component/e2e tests that poked `getByRole('combobox').value` are rewritten for the real interaction.
* **Bad**, "visual Color Picker only" means an sRGB picker cannot faithfully represent wide-gamut oklch tokens (e.g. `oklch(75% 0.2 200)`); editing such a token via the picker gamut-clips it. Accepted as the cost of dropping the oklch text field; a hex channel input preserves precise entry.
* **Neutral**, a second small dependency (`culori`) for oklch conversion; portaled parts are styled with `:global()`; the editor `:root` tokens still inherit to body-portaled content (it is editor chrome, so no overlay-token collision).

### Confirmation

* A `shared/ui/Select.svelte` smoke (renders via Ark, styled only with tokens on the `[data-part]` anatomy, drives its single-string API in a browser test) plus an explicit check that the `/overlay` build chunk contains no `@ark-ui`.
* Per-phase gates: `svelte-check` (0/0), eslint + prettier, `vite build` (prerender), vitest (server + chromium client), Playwright e2e. `cargo` is unaffected.

## Pros and Cons of the Options

### A. Keep hand-rolling

* Good, no new dependency; full control.
* Bad, re-implements accessibility per component; no Color Picker; the existing code is the thing we are trying to replace.

### B. Ark UI (chosen)

* Good, 45+ audited components incl. Color Picker; headless + data-attributes fit our tokens; Svelte 5 + SSR-safe; multi-framework parity (React/Vue/Solid/Svelte share one machine).
* Bad, larger dependency tree; non-native selects change test interactions; the picker is sRGB.

### C. Bits UI

* Good, Svelte-native, lighter (no Zag layer), idiomatic.
* Bad, smaller catalog, no Color Picker; would not cover the comprehensive scope.

### D. Melt UI

* Good, lowest-level builders, maximal flexibility.
* Bad, steeper builder API, fewer ready components, more to assemble; no Color Picker out of the box.

## More Information

Builds on [ADR-0003](0003-dark-first-pro-creative-tool-editor-aesthetic.md) (the editor aesthetic and token system are reused unchanged; Ark adds no styling) and [ADR-0004](0004-responsive-editor-shell.md) (the responsive shell is unaffected). Updates the locked tech stack in CLAUDE.md and spec §16 to add `@ark-ui/svelte` (+ `culori`) as editor-only UI dependencies; the `/overlay` runtime stays Ark-free. Bits UI and Melt UI were the Svelte-native alternatives considered. Deferred: drag-and-drop docking (no Ark primitive), and a wide-gamut color editor if the oklch sRGB limitation proves constraining.
