// Resolves a scene's themeId into how to apply it: a `data-theme` attribute
// (built-in CSS cascade, or a custom theme's `base` as the fallback layer) plus
// an inline CSS-custom-property string for a custom theme's token values. The
// single source of theme application for both /overlay and the editor canvas.
// Pure: no DOM, no Loro. Token links (`link:<token>`) resolve here with cycle
// detection; a dangling or cyclic token is dropped so the base CSS still covers
// it. An accent override is appended last so it wins.

import { DEFAULT_THEME_ID, LINK_PREFIX, THEME_TOKENS, isBuiltinTheme } from '../model/tokens';
import type { CustomTheme } from '$lib/shared/crdt/workspace-view';

export interface ResolvedThemeStyle {
	/** Value for the canvas root's `data-theme` attribute. */
	dataTheme: string;
	/** Inline `--token: value;` declarations (empty for a plain built-in). */
	inlineVars: string;
}

/** Follow `link:` references to a literal value; returns undefined on a dangling
 *  reference or a cycle (the canonical-token caller then drops the token). */
function resolveToken(
	tokens: Record<string, string>,
	name: string,
	seen: Set<string>
): string | undefined {
	const value = tokens[name];
	if (value === undefined) return undefined;
	if (!value.startsWith(LINK_PREFIX)) return value;
	if (seen.has(name)) return undefined; // cycle
	seen.add(name);
	return resolveToken(tokens, value.slice(LINK_PREFIX.length), seen);
}

export function resolveThemeStyle(
	themeId: string,
	customThemes: CustomTheme[],
	accentOverride = ''
): ResolvedThemeStyle {
	const parts: string[] = [];
	let dataTheme: string;

	if (isBuiltinTheme(themeId)) {
		dataTheme = themeId;
	} else {
		const custom = customThemes.find((theme) => theme.id === themeId);
		if (custom) {
			// The base built-in is the fallback layer for any token left unset,
			// dangling, or cyclic; resolved values override it inline.
			dataTheme = isBuiltinTheme(custom.base) ? custom.base : DEFAULT_THEME_ID;
			for (const token of THEME_TOKENS) {
				const resolved = resolveToken(custom.tokens, token, new Set());
				if (resolved !== undefined) parts.push(`--${token}: ${resolved};`);
			}
		} else {
			dataTheme = DEFAULT_THEME_ID;
		}
	}

	if (accentOverride) parts.push(`--accent: ${accentOverride};`);
	return { dataTheme, inlineVars: parts.join(' ') };
}
