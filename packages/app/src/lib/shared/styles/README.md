# shared/styles

Global CSS for the editor: the design-token system and a minimal reset. Imported once at the app root (`src/routes/+layout.svelte`) so the tokens apply to every route.

## Files

- `tokens.css`: the alias-tier design tokens (color, typography, spacing, radii, stroke, motion, chrome structure). Consume via `var(--token-name)` in component scoped styles. Never hard-code a color, size, or duration; reference a token.
- `reset.css`: minimal cross-browser reset, depends on tokens for the base body color/font and the `:focus-visible` ring.

## Dark-first, with a light secondary

Dark is the baseline. The token values live in `:root` (and are re-asserted under `.dark`), so the page paints dark with zero JavaScript. `app.html` ships `class="dark"` on `<html>` as the explicit default marker. Light mode is the secondary: add `class="light"` to `<html>` to opt out. The light palette is hue-stable with the dark one (faint cool chroma at hue 264) so toggling does not shift temperature.

This is the EDITOR universe. The overlay-runtime theme presets (Cozy, Cyber, Editorial, Sticker) are a separate token universe with their own files; do not consume editor tokens from overlay code or vice versa.

## Adding a token

1. Add it to the correct section of `tokens.css` (typography, color surfaces/emphasis/state/affordances/inverse, chrome structure, motion, spacing, radii, stroke).
2. If it is a color or a chrome surface, add the light-mode value to the `.light` block. Theme-independent tokens (metrics, typography, motion) live only in `:root`.
3. Reference it as `var(--name)`; never inline the literal.

The palette is grounded in Zed's One Dark (translated to OKLCH); structural depth draws from Graphite, Affinity, and DaVinci Resolve. Rationale and the considered alternatives are in `docs/decisions/0003-dark-first-pro-creative-tool-editor-aesthetic.md` and spec section 12.
