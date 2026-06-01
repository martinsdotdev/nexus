import { describe, expect, test } from 'vitest';
import { oklchToHex, colorToOklch } from './oklch-color';

describe('oklch <-> sRGB bridge', () => {
	test('oklchToHex maps the achromatic extremes', () => {
		expect(oklchToHex('oklch(0% 0 0)')).toBe('#000000');
		expect(oklchToHex('oklch(100% 0 0)')).toBe('#ffffff');
	});

	test('oklchToHex clamps an out-of-gamut oklch to a valid hex', () => {
		// cyber's primary is near/outside the sRGB gamut.
		expect(oklchToHex('oklch(75% 0.2 200)')).toMatch(/^#[0-9a-f]{6}$/);
	});

	test('colorToOklch yields a percentage-L oklch string', () => {
		expect(colorToOklch('#000000')).toBe('oklch(0% 0 0)');
		expect(colorToOklch('#ffffff')).toBe('oklch(100% 0 0)');
	});

	test('colorToOklch of a saturated color is a well-formed oklch', () => {
		expect(colorToOklch('#ff0000')).toMatch(/^oklch\([\d.]+% [\d.]+ [\d.]+\)$/);
	});

	test('an in-gamut token round-trips back to a parseable oklch', () => {
		const back = colorToOklch(oklchToHex('oklch(70% 0.15 50)'));
		expect(back).toMatch(/^oklch\(/);
		expect(oklchToHex(back)).toMatch(/^#[0-9a-f]{6}$/);
	});

	test('an unparseable string is returned unchanged', () => {
		expect(colorToOklch('not-a-color')).toBe('not-a-color');
	});
});
