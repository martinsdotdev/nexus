// The overlay theme token vocabulary (spec §8.1 / §12.2): the CSS custom
// properties every overlay widget reads via var(--token). A theme assigns a
// value to each token. Built-in themes (cozy/cyber/editorial/sticker) ship as
// CSS files; a CUSTOM theme (ADR-0006) stores its values as data in the
// workspace doc. A token value is either a literal (e.g. `oklch(70% 0.15 50)`)
// or a `link:<token>` reference that resolves to another token's value.

export const BUILTIN_THEME_IDS = ['cozy', 'cyber', 'editorial', 'sticker'] as const;
export type BuiltinThemeId = (typeof BUILTIN_THEME_IDS)[number];

/** Fallback when a scene names a theme that no longer exists. */
export const DEFAULT_THEME_ID: BuiltinThemeId = 'cozy';

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

export function isBuiltinTheme(id: string): id is BuiltinThemeId {
	return (BUILTIN_THEME_IDS as readonly string[]).includes(id);
}
