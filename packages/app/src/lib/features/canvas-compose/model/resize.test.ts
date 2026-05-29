import { describe, expect, test } from 'vitest';
import { resizeRect } from './resize';

const R = (x: number, y: number, w: number, h: number) => ({ x, y, w, h });

describe('resizeRect', () => {
	test('east handle grows width only', () => {
		expect(resizeRect(R(100, 100, 200, 200), 'e', 50, 0)).toEqual(R(100, 100, 250, 200));
	});

	test('south-east handle grows width and height', () => {
		expect(resizeRect(R(100, 100, 200, 200), 'se', 50, 30)).toEqual(R(100, 100, 250, 230));
	});

	test('west handle moves x and shrinks width', () => {
		expect(resizeRect(R(100, 100, 200, 200), 'w', 20, 0)).toEqual(R(120, 100, 180, 200));
	});

	test('north handle moves y and shrinks height', () => {
		expect(resizeRect(R(100, 100, 200, 200), 'n', 0, 20)).toEqual(R(100, 120, 200, 180));
	});

	test('enforces a minimum size on the east edge', () => {
		expect(resizeRect(R(100, 100, 200, 200), 'e', -1000, 0, 24)).toEqual(R(100, 100, 24, 200));
	});

	test('enforces a minimum size on the west edge without crossing over', () => {
		// right = 300, min 24 -> x clamps to 276, w = 24.
		expect(resizeRect(R(100, 100, 200, 200), 'w', 1000, 0, 24)).toEqual(R(276, 100, 24, 200));
	});
});
