// Token-link helpers for the theme builder: resolve a token to its literal value
// (following `link:` references, cycle-safe) and check whether a proposed link
// would create a cycle (so the builder can refuse it at author time). Pure.

import { LINK_PREFIX } from '../model/tokens';
import type { ThemeTokens } from '$lib/shared/crdt/workspace-view';

/** Follow `link:` references to a literal; empty string on a cycle or a miss. */
export function resolveTokenValue(
	tokens: ThemeTokens,
	name: string,
	seen: Set<string> = new Set()
): string {
	const value: string | undefined = tokens[name];
	if (value === undefined) return '';
	if (!value.startsWith(LINK_PREFIX)) return value;
	if (seen.has(name)) return '';
	seen.add(name);
	return resolveTokenValue(tokens, value.slice(LINK_PREFIX.length), seen);
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
