// A stable presence color per collaborator, drawn from the curated 8-hue --mp-* palette
// (shared/styles/tokens.css). The same account id always maps to the same hue, so a peer's
// cursor, selection outline, and roster chip all match. Hashing into a fixed palette (rather
// than a free hue) keeps every color equally legible on dark graphite and clear of the
// editor-selection blue that marks YOUR own selection.

/** The presence palette keys, in the design's assignment order. */
export const PRESENCE_KEYS = [
	'amber',
	'green',
	'violet',
	'pink',
	'cyan',
	'lime',
	'red',
	'teal'
] as const;

export type PresenceKey = (typeof PRESENCE_KEYS)[number];

/** The CSS custom property holding a presence key's color (e.g. `var(--mp-teal)`). */
export function colorVar(key: PresenceKey): string {
	return `var(--mp-${key})`;
}

/** The curated palette key for an account id (stable, deterministic). */
export function peerColorKey(id: string): PresenceKey {
	let hash = 0;
	for (let i = 0; i < id.length; i++) {
		hash = (Math.imul(hash, 31) + id.charCodeAt(i)) | 0;
	}
	return PRESENCE_KEYS[Math.abs(hash) % PRESENCE_KEYS.length];
}

/** The presence color (a CSS var reference) for an account id. */
export function peerColor(id: string): string {
	return colorVar(peerColorKey(id));
}
