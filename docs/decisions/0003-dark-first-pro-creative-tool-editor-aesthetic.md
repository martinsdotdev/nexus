---
status: "accepted"
date: 2026-05-27
decision-makers: project owner
consulted: 
informed: future contributors
---

# Dark-first pro-creative-tool editor aesthetic

## Context and Problem Statement

The original token system (spec §12) committed the editor chrome to a "Linear-band restrained" identity, with light and dark treated as equal peers and light living in `:root`. As the editor UI takes shape, the target shifted: the editor should feel like a dark-first professional creative tool, drawing from Zed, Graphite, Affinity, and DaVinci Resolve. That means deeper near-black surfaces, a real elevation ladder (app background below panels below floating surfaces), denser pro-tool chrome, and signature structural patterns (keyboard-first command palette, left tool rail, dockable studio panels, a bottom scene strip).

Two questions: do we redesign the token system or evolve it, and do we keep light as a co-equal mode or demote it to secondary?

## Decision Drivers

- Dark-first is the design priority. Streamers work in dim setups, and a near-neutral dark chrome lets them judge overlay colors against game capture without the editor biasing perception (the DaVinci Resolve rationale).
- Preserve the inherited shadcn/ui + OKLCH token vocabulary (contributor familiarity, zero churn for existing references).
- Keep the two-token-universe split intact: editor chrome tokens are separate from the overlay-runtime theme presets (Cozy/Cyber/Editorial/Sticker).
- The flat token set lacked pro-tool depth cues (multi-surface elevation, panel shadows, a shared focus-ring recipe, chrome-layout metrics). Add them.
- Ground the palette in real references rather than invented numbers.
- The restraint budget from product rule 9 (editor animations <= 300ms, ease-out) stays.

## Considered Options

- **A. Stay Linear-band-clean, light and dark co-equal** (the status quo from §12).
- **B. Dark-first evolution of the existing tokens** (deepen and retune darker, add chrome-structure and elevation tokens, demote light to a secondary, ground in Zed One Dark).
- **C. Full Figma-style redesign** (new vocabulary, multi-accent, heavier chrome).
- **D. Adopt a prebuilt dark theme wholesale** (ship an existing palette as-is).

## Decision Outcome

Chosen option: **"B. Dark-first evolution of the existing tokens."**

Dark values move to `:root` and are re-asserted under `.dark`, so the page paints dark with zero JavaScript and the documented class mechanism still resolves. `app.html` ships `class="dark"` on `<html>` as the explicit default marker. Light becomes the secondary, held under `.light`, ported from the original spec light palette plus a faint cool chroma so toggling does not shift temperature.

The palette is grounded in Zed's "One Dark" theme, translated to OKLCH at hue 264 with a faint cool chroma (0.008 to 0.015) so the near-blacks read as graphite rather than flat grey. The elevation ladder is explicit: `--background` (app shell, darkest) < `--card` (studio panel) < `--popover` (command palette, floating highest), with `--input` lifting above `--card`. Structural depth (chrome surfaces, panel and popover shadows, layout metrics) draws from Graphite, Affinity, and DaVinci Resolve.

New chrome-structure tokens are additive to the shadcn vocabulary (no renames): chrome-layout metrics (`--titlebar-height`, `--toolrail-width`, `--scenestrip-height`, `--panel-header-height`, panel width bounds), chrome surfaces (`--titlebar`, `--panel-header`, `--toolrail`, `--divider`), elevation shadows (`--shadow-panel`, `--shadow-popover`, `--shadow-dragging`), and a shared `--focus-ring` recipe.

This applies to the **editor universe only**. The overlay-runtime theme presets are untouched.

### Consequences

- **Good**, because the editor reads as a deeper, more professional dark tool with an explicit elevation ladder the flat token set lacked.
- **Good**, because keyboard-first affordances now have token support (the shared focus-ring recipe).
- **Good**, because no vocabulary churn: the shadcn names survive, retuned; existing references keep working.
- **Good**, because shipping `class="dark"` in `app.html` removes the light-flash risk on prerendered pages.
- **Good**, because the two-universe split is preserved; overlay presets are unaffected.
- **Bad**, because light mode is now lower-fidelity and less-tested (accepted; it is explicitly the secondary).
- **Bad**, because the spec §12 dark block and framing had to be rewritten (history-preserving: this ADR supersedes the relevant §12 framing; the spec is the living doc).
- **Neutral**, because the first consumer is a visual prototype; the chrome-structure tokens get exercised for real when the functional editor lands.

### Confirmation

- Tokens live in `packages/app/src/lib/shared/styles/tokens.css`. The `/edit` route renders the shell dark-by-default.
- `pnpm -F app build` (adapter-static), `pnpm -F app check`, and `pnpm -F app lint` all pass.
- Visual review at `/edit` confirms the elevation ladder reads (app bg darkest, panels lift, command palette floats highest with the strongest shadow) and the four signature patterns are present.

## Pros and Cons of the Options

### A. Stay Linear-band-clean, light and dark co-equal

- **Good**, because zero work; the existing tokens stand.
- **Bad**, because it does not deliver the pro-tool depth or the dark priority the direction calls for.

### B. Dark-first evolution of the existing tokens

- **Good**, because deepens the aesthetic while preserving the shadcn vocabulary and contributor familiarity.
- **Good**, because grounded in a real reference (Zed One Dark) with an explicit elevation ladder.
- **Good**, because additive chrome-structure tokens, no renames.
- **Neutral**, because light mode becomes secondary (a deliberate demotion).

### C. Full Figma-style redesign

- **Bad**, because throws away the shadcn vocabulary and the brand-blue primary; large churn.
- **Bad**, because heavier chrome than a restrained tool wants.

### D. Adopt a prebuilt dark theme wholesale

- **Bad**, because loses the bespoke restraint and the existing brand-blue primary.
- **Bad**, because still would not provide the chrome-structure tokens the shell needs.

## More Information

- Spec §11 (editor UX, the layout evolution) and §12 (design tokens, the dark-first retune).
- References: Zed (One Dark palette + command palette), Graphite (Rust/WASM/Svelte dark editor, tool rail + panels), Affinity (personas + dockable studios), DaVinci Resolve (page-based workspaces + very-dark color-neutral chrome).
- The editor-shell prototype landed at `packages/app/src/routes/edit/` with primitives in `packages/app/src/lib/shared/ui/`. The visible locale switcher and full drag-docking remain deferred; this cycle is the static shell + the token foundation.
- Implementation note: `lucide-svelte` was installed this cycle for the icon system (locked stack, §16).
