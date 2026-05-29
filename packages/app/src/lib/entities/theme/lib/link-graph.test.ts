import { describe, expect, test } from 'vitest';
import { resolveTokenValue, wouldCycle } from './link-graph';

describe('resolveTokenValue', () => {
	test('returns a literal directly', () => {
		expect(resolveTokenValue({ a: 'red' }, 'a')).toBe('red');
	});

	test('follows a link to its literal', () => {
		expect(resolveTokenValue({ a: 'red', b: 'link:a' }, 'b')).toBe('red');
	});

	test('a cycle resolves to empty (no infinite loop)', () => {
		expect(resolveTokenValue({ a: 'link:b', b: 'link:a' }, 'a')).toBe('');
	});

	test('a missing token resolves to empty', () => {
		expect(resolveTokenValue({}, 'a')).toBe('');
	});
});

describe('wouldCycle', () => {
	test('linking a token to itself is a cycle', () => {
		expect(wouldCycle({}, 'a', 'a')).toBe(true);
	});

	test('linking that closes a loop is a cycle', () => {
		// b already links to a; linking a -> b would loop.
		expect(wouldCycle({ b: 'link:a' }, 'a', 'b')).toBe(true);
	});

	test('linking down an open chain is not a cycle', () => {
		// b links to c (not back to a); a -> b is safe.
		expect(wouldCycle({ b: 'link:c' }, 'a', 'b')).toBe(false);
	});
});
