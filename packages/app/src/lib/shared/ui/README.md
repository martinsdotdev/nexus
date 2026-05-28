# shared/ui

Generic, business-logic-free UI primitives for the editor chrome. Style with design tokens only (no hard-coded colors, sizes, or durations). Import via the barrel: `import { IconButton, StudioPanel, CommandPalette } from '$lib/shared/ui';`.

## Primitives

- `IconButton`: square icon button. Pass the glyph as the default child (typically a lucide icon): `<IconButton label="Select" {active} {onclick}><MousePointer2 size={18} /></IconButton>`. Owns hover/active/focus/press affordances.
- `StudioPanel`: a dockable-looking panel frame (header + collapse chevron + faux drag handle + body slot). Props: `title`, `collapsed`, `onToggleCollapse`, `side`. The body is the default child. Real drag-docking is deferred; the frame only looks dockable.
- `CommandPalette`: an overlay command launcher. Props: `open`, `onClose`, `placeholder`, `items` (`CommandItem[]`). Handles focus capture/restore, local substring filter, arrow-key navigation, Escape, and backdrop close. Selecting an item closes the palette; command execution is deferred.

## Rules

- These are `shared/` primitives: no domain knowledge, no business logic, no imports from higher FSD layers. A future editor toolbar / feature composes them.
- Token-only styling. If a primitive needs a value the token system lacks, add the token to `shared/styles/tokens.css` rather than hard-coding it here.
- Components are default exports; the barrel re-exports them plus the `CommandItem` type (declared in `types.ts`, not inside a `.svelte` file, so it is cleanly importable).
