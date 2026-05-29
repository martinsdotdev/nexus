// Token-link helpers for the theme builder + the runtime resolver: follow a
// token's `link:` references to a literal (cycle-safe), and check whether a
// proposed link would create a cycle (so the builder can refuse it). Pure. This
// is the single owner of the link-following + cycle-guard traversal.

import { LINK_PREFIX } from '../model/tokens';
import type { ThemeTokens } from '$lib/shared/crdt/workspace-view';

/** Follow `link:` references to a literal value; undefined on a dangling
 *  reference or a cycle (a caller that needs to drop the token uses this). */
export function resolveToken(
	tokens: ThemeTokens,
	name: string,
	seen: Set<string> = new Set()
): string | undefined {
	const value: string | undefined = tokens[name];
	if (value === undefined) return undefined;
	if (!value.startsWith(LINK_PREFIX)) return value;
	if (seen.has(name)) return undefined; // cycle
	seen.add(name);
	return resolveToken(tokens, value.slice(LINK_PREFIX.length), seen);
}

/** Like {@link resolveToken} but yields '' instead of undefined (for the builder
 *  to display or to freeze a token at its current value). */
export function resolveTokenValue(tokens: ThemeTokens, name: string): string {
	return resolveToken(tokens, name) ?? '';
}

/** Would linking `from` -> `to` create a cycle? Follow `to`'s chain; a cycle
 *  exists if it returns to `from` (or `from === to`). */
export function wouldCycle(tokens: ThemeTokens, from: string, to: string): boolean {
	if (from === to) return true;
	const seen = new Set<string>([from]);
	let current: string | undefined = to;
	while (current !== undefined) {
		if (seen.has(current)) return true;
		seen.add(current);
		const value: string | undefined = tokens[current];
		current = value?.startsWith(LINK_PREFIX) ? value.slice(LINK_PREFIX.length) : undefined;
	}
	return false;
}
