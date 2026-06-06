import { describe, expect, it } from 'vitest';
import { peerColor, peerColorKey, colorVar, PRESENCE_KEYS } from './peer-color';

describe('peer-color', () => {
	it('maps an id to a curated palette var', () => {
		expect(peerColor('user-1')).toMatch(
			/^var\(--mp-(amber|green|violet|pink|cyan|lime|red|teal)\)$/
		);
	});

	it('is stable for the same id', () => {
		expect(peerColor('abc')).toBe(peerColor('abc'));
		expect(peerColorKey('abc')).toBe(peerColorKey('abc'));
	});

	it('only ever returns one of the eight palette keys', () => {
		for (let i = 0; i < 200; i++) {
			expect(PRESENCE_KEYS).toContain(peerColorKey(`id-${i}`));
		}
	});

	it('spreads ids across the whole palette', () => {
		const used = new Set(Array.from({ length: 200 }, (_, i) => peerColorKey(`peer-${i}`)));
		expect(used.size).toBe(PRESENCE_KEYS.length);
	});

	it('wraps a key as a CSS custom property', () => {
		expect(colorVar('teal')).toBe('var(--mp-teal)');
	});
});
