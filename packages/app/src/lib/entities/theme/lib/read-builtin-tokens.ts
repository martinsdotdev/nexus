// Read a built-in theme's resolved token values out of the CSS cascade, so the
// theme builder can "duplicate to edit": seed a custom theme with exactly what
// the built-in renders. Browser-only (getComputedStyle); call it from an event
// handler, never at module scope. Mounts a hidden, off-screen [data-theme] probe.

import { THEME_TOKENS } from '../model/tokens';
import type { ThemeTokens } from '$lib/shared/crdt/workspace-view';

export function readBuiltinTokens(builtinId: string): ThemeTokens {
	const probe = document.createElement('div');
	probe.setAttribute('data-theme', builtinId);
	probe.style.position = 'absolute';
	probe.style.visibility = 'hidden';
	probe.style.pointerEvents = 'none';
	document.body.appendChild(probe);

	const computed = getComputedStyle(probe);
	const tokens: ThemeTokens = {};
	for (const token of THEME_TOKENS) {
		const value = computed.getPropertyValue(`--${token}`).trim();
		if (value) tokens[token] = value;
	}

	document.body.removeChild(probe);
	return tokens;
}
