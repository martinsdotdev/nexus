---
status: "accepted"
date: 2026-05-28
decision-makers: project owner
consulted:
informed: future contributors
---

# Responsive editor shell via CSS-owned reflow, container-query panels, and off-canvas drawers

## Context and Problem Statement

The editor shell (spec §11, [ADR-0003](0003-dark-first-pro-creative-tool-editor-aesthetic.md)) was built desktop-only: a fixed CSS grid with a 48px tool rail, 40px title bar, 88px scene strip, and two 280px studio panels, with zero media or container queries. The canvas starves below ~1024px and the layout breaks below ~768px. The route is prerendered by adapter-static, so it must paint correctly with no JavaScript at first load.

We want the editor usable down to a ~640px floor (best-effort to ~360px) for split-screen-with-OBS, small laptops, and tablet-portrait, without abandoning the canonical desktop three-panel design.

## Decision Drivers

- Prerender-correct without JavaScript: layout must be right at any width before hydration.
- The locked stance: a 640px mobile-friendly floor, but the desktop three-panel layout stays canonical; narrow widths are graceful degradation, not a mobile-first redesign.
- Preserve the editor motion budget from rule 9 (<= 300ms, ease-out, never ease-in-out) and honor prefers-reduced-motion.
- FSD layering: a generic Drawer is a shared/ui primitive; the shell composition stays in routes/edit/.
- Tokens, not magic numbers; reuse the proven CommandPalette accessibility pattern rather than inventing one.
- Ground the breakpoints and techniques in current (May 2026) browser support, not assumptions.

## Considered Options

- **A. CSS-owned reflow** (viewport `@media` for the shell + `@container` for panel internals), with JavaScript only for drawer open and close.
- **B. JavaScript-driven layout** (read `matchMedia`, branch the grid in Svelte).
- **C. Mobile-first rewrite.**
- **D. Discrete per-breakpoint layouts** that reset and recompile (the DaVinci Resolve model).
- **E. Container queries only,** no viewport queries.
- Sub-decision: breakpoint values as PostCSS-substituted variables vs documented-constant triplication.

## Decision Outcome

Chosen option: **"A. CSS-owned reflow with JavaScript only for drawer open and close."**

CSS media and container queries make the prerendered HTML correct at every width before hydration. Four viewport zones: wide (>= 1280px, both panels docked), medium (960 to 1279px, the right inspector becomes an off-canvas drawer), narrow (640 to 959px, both panels are drawers), and best-effort (< 640px, the tool rail horizontalizes, the canvas goes full-bleed, the scene strip thins). The inspector sheds first because it is reference; the widget palette is the working surface.

A new Drawer primitive (shared/ui) mirrors CommandPalette's focus capture and restore, scrim, and Escape handling, and adds a focus trap and background `inert`. Drawers default closed, so the no-JS narrow baseline is a full-width canvas. A client-only `viewportMode` (derived from `matchMedia`, never read by first-paint layout) routes the panel-toggle intent (collapse when docked, open a drawer when off-canvas) and closes orphaned drawers when a panel re-docks. Panel internals adapt to their own width via `@container`. The shell adopts `100dvh`; display type sizes gain `clamp()`; coarse pointers get 44px tap targets via transparent overlays; hover is gated behind `@media (hover: hover)`; a global prefers-reduced-motion reset zeroes motion.

Breakpoints are a documented-constant triplication (a `tokens.css` reference block, a TS `BREAKPOINTS` const for `matchMedia`, and hard-coded literals in each query) because CSS `@media`/`@container` cannot read `var()`; PostCSS substitution was rejected to avoid a build dependency the repo does not have.

This applies to the editor universe only. The overlay-runtime route is a fixed-canvas OBS Browser Source and is untouched.

### Consequences

- **Good**, because the prerendered page is correct at every width with zero JavaScript; JS only adds drawer interactivity.
- **Good**, because the canonical desktop three-panel design is preserved; narrow widths degrade rather than redesign.
- **Good**, because the Drawer reuses a proven accessibility pattern and improves on it (focus trap + `inert`).
- **Good**, because panel internals are portable: a panel adapts the same way docked at 280px or stretched in a drawer.
- **Bad**, because the breakpoint values live in three places (mitigated by a sync comment at each site and this ADR).
- **Bad**, because sub-640px is best-effort, not pixel-perfect (accepted; it is below the stated floor).
- **Neutral**, because StudioPanel gains a `variant` prop and the panel body snippet renders at two mount points (docked and drawer) to avoid duplicating content.

### Confirmation

- `pnpm -F app build` prerenders `/edit` without error (the gate that catches `window`/`matchMedia` leaking into render scope), and `check` + `lint` pass with zero warnings.
- A manual width matrix (1920/1440/1280/960/640/360), a JavaScript-disabled reload at 1920/960/640, a coarse-pointer check, and a prefers-reduced-motion check confirm the zones, the no-JS baselines, the 44px targets, and the instant-motion fallback.

## Pros and Cons of the Options

### A. CSS-owned reflow + container-query panels + JS-only drawers

- **Good**, because prerender-correct without JS, and aligned with the Figma/shadcn split (viewport for the shell, container for components).
- **Good**, because additive: the existing grid and tokens survive, retuned.
- **Neutral**, because it requires the breakpoint triplication.

### B. JavaScript-driven layout

- **Bad**, because the prerendered HTML would be wrong until hydration (a flash of desktop layout on a phone), violating the no-JS-first-paint requirement.
- **Bad**, because it couples layout to client state and re-runs on every resize.

### C. Mobile-first rewrite

- **Bad**, because the locked stance keeps desktop canonical; a mobile-first base would invert the priority and churn the whole shell.

### D. Discrete per-breakpoint layouts (Resolve-style)

- **Bad**, because heavier (multiple full layouts), loses visual continuity across a resize, and adds state for a shell that is still a prototype.

### E. Container queries only

- **Bad**, because container queries cannot relocate regions across the top-level grid; the shell itself must reflow its columns, which is inherently a viewport concern.

### Sub-decision: breakpoints as PostCSS variables vs documented constants

- **Chosen:** documented-constant triplication, with a sync comment at each site. Zero new build dependencies.
- **Rejected:** PostCSS custom-media substitution, because it adds a preprocessor the project does not currently use, for a three-value table.

## More Information

- Spec §11.7 (responsive behavior) and §12.6 (responsive, touch, and breakpoint tokens).
- Reuses: CommandPalette focus capture and restore plus the scrim affordance, the StudioPanel collapse mechanism, and the `.shell` grid.
- Research basis (May 2026 browser support): container queries, `dvh`/`svh`/`lvh`, and `:has()` are Baseline; `interpolate-size`/`calc-size()` is Chromium-only and was avoided. Viewport-for-shell plus container-for-components follows Figma and shadcn; Zed uses a hybrid; DaVinci Resolve resets layouts (option D).
