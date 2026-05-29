import { describe, expect, test } from 'vitest';
import { clamp, clientToVirtual, fitScale, VIRTUAL_H, VIRTUAL_W } from './geometry';

describe('fitScale', () => {
	test('native-size container fits at scale 1', () => {
		expect(fitScale({ w: VIRTUAL_W, h: VIRTUAL_H })).toBe(1);
	});

	test('a half-size container scales to 0.5', () => {
		expect(fitScale({ w: 960, h: 540 })).toBe(0.5);
	});

	test('letterboxes to the limiting (smaller) dimension', () => {
		// A very wide container is bound by its height.
		expect(fitScale({ w: 99999, h: 540 })).toBe(0.5);
	});

	test('a zero or negative container yields 0 (pre-layout guard)', () => {
		expect(fitScale({ w: 0, h: 0 })).toBe(0);
	});
});

describe('clientToVirtual', () => {
	test('maps the canvas origin to (0,0)', () => {
		expect(clientToVirtual({ x: 100, y: 50 }, { x: 100, y: 50 }, 0.5)).toEqual({ x: 0, y: 0 });
	});

	test('divides the client offset by the scale', () => {
		expect(clientToVirtual({ x: 150, y: 100 }, { x: 100, y: 50 }, 0.5)).toEqual({ x: 100, y: 100 });
	});

	test('a zero scale is safe (returns the origin point)', () => {
		expect(clientToVirtual({ x: 5, y: 5 }, { x: 0, y: 0 }, 0)).toEqual({ x: 0, y: 0 });
	});
});

describe('clamp', () => {
	test('bounds a value within the range', () => {
		expect(clamp(5, 0, 10)).toBe(5);
		expect(clamp(-1, 0, 10)).toBe(0);
		expect(clamp(11, 0, 10)).toBe(10);
	});
});
