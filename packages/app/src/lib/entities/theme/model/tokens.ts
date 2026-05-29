// The overlay theme token vocabulary (spec §8.1 / §12.2): the CSS custom
// properties every overlay widget reads via var(--token). A theme assigns a
// value to each token. Every theme is data in the workspace doc (ADR-0007): the
// relay seeds the built-ins (cozy/cyber/editorial/sticker) as protected entries;
// users author the rest. A token value is either a literal (e.g.
// `oklch(70% 0.15 50)`) or a `link:<token>` reference that resolves to another.

/** The id of the seeded default theme: the resolver's safety floor and the
 *  fallback a dangling scene reference is repaired to (mirrors nexus-core). */
export const DEFAULT_THEME_ID = 'cozy';

/** A token value of the form `link:<token>` aliases another token. */
export const LINK_PREFIX = 'link:';

// Grouped for the theme-builder rail (Zed-style); the flat list below drives
// resolution + validation. Order is the display order.
export const TOKEN_GROUPS = [
	{
		label: 'Surfaces',
		tokens: [
			'background',
			'foreground',
			'card',
			'card-foreground',
			'popover',
			'popover-foreground',
			'muted',
			'muted-foreground'
		]
	},
	{
		label: 'Emphasis',
		tokens: [
			'primary',
			'primary-foreground',
			'secondary',
			'secondary-foreground',
			'accent',
			'accent-foreground'
		]
	},
	{
		label: 'State',
		tokens: [
			'info',
			'info-foreground',
			'success',
			'success-foreground',
			'warning',
			'warning-foreground',
			'destructive',
			'destructive-foreground'
		]
	},
	{ label: 'Affordances', tokens: ['border', 'border-subtle', 'input', 'ring'] },
	{ label: 'Inverse', tokens: ['invert', 'invert-foreground'] },
	{
		label: 'Shape & motion',
		tokens: ['radius', 'shadow-widget', 'font-display', 'font-body', 'motion-feel']
	}
] as const;

/** Every token name, flat, in display order. */
export const THEME_TOKENS: readonly string[] = TOKEN_GROUPS.flatMap((group) => group.tokens);

/** Tokens whose value is NOT a CSS color (the builder gives these a text field). */
export const NON_COLOR_TOKENS = new Set([
	'radius',
	'shadow-widget',
	'font-display',
	'font-body',
	'motion-feel'
]);

/** A token gets a color picker in the builder unless it is a non-color token. */
export function isColorToken(token: string): boolean {
	return !NON_COLOR_TOKENS.has(token);
}
