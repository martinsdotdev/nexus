// Resolves a scene's themeId into the inline CSS-custom-property string the
// overlay/canvas root carries. Every theme is data now (ADR-0007): there is no
// [data-theme] cascade to fall back on, so this is the single source of theme
// application for both /overlay and the editor canvas. Pure: no DOM, no Loro.
//
// Layering (lowest to highest precedence): the default theme (a safety floor so
// a partial/imported theme never renders unstyled), the selected theme's `base`,
// then the selected theme itself. Token links (`link:<token>`) resolve over the
// merged set with cycle detection; a token whose winning value is a dangling or
// cyclic link falls back to the default theme's literal. The default theme and
// any `base` are protected built-ins, which hold only literals (no links; see
// mutations.setThemeToken), so this fallback is always resolvable: every one of
// the 33 tokens is emitted, which is what keeps the overlay from inheriting the
// editor chrome's same-named :root tokens. An accent override wins.

import { DEFAULT_THEME_ID, THEME_TOKENS } from '../model/tokens';
import type { Theme, ThemeTokens } from '$lib/shared/crdt/workspace-view';
import { resolveToken } from './link-graph';

export interface ResolvedThemeStyle {
	/** Inline `--token: value;` declarations applied to the themed root. */
	inlineVars: string;
}

export function resolveThemeStyle(
	themeId: string,
	themes: Theme[],
	accentOverride = ''
): ResolvedThemeStyle {
	const byId = new Map(themes.map((theme) => [theme.id, theme]));
	const fallback = byId.get(DEFAULT_THEME_ID);
	const selected = byId.get(themeId) ?? fallback;
	const base = selected?.base ? byId.get(selected.base) : undefined;
	const merged: ThemeTokens = { ...fallback?.tokens, ...base?.tokens, ...selected?.tokens };

	const parts: string[] = [];
	for (const token of THEME_TOKENS) {
		// The accent override (if any) replaces the resolved accent in place, so
		// `--accent` is emitted exactly once rather than appended as a duplicate.
		const value =
			token === 'accent' && accentOverride
				? accentOverride
				: (resolveToken(merged, token, new Set()) ?? resolveToken(fallback?.tokens ?? {}, token));
		if (value !== undefined) parts.push(`--${token}: ${value};`);
	}

	return { inlineVars: parts.join(' ') };
}
