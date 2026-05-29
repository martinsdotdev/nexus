import { describe, expect, test } from 'vitest';
import { snapRect } from './snap';

const R = (x: number, y: number, w: number, h: number) => ({ x, y, w, h });

describe('snapRect', () => {
	test('snaps the left edge to the canvas left within threshold', () => {
		const r = snapRect(R(5, 500, 100, 100), []);
		expect(r.x).toBe(0);
		expect(r.guides).toContainEqual({ axis: 'x', pos: 0 });
	});

	test('does not snap when nothing is within threshold', () => {
		const r = snapRect(R(500, 500, 100, 100), []);
		expect(r.x).toBe(500);
		expect(r.y).toBe(500);
		expect(r.guides).toEqual([]);
	});

	test('snaps the right edge to the canvas right', () => {
		// right edge = 1815 + 100 = 1915, canvas right 1920, distance 5 -> snap.
		expect(snapRect(R(1815, 500, 100, 100), []).x).toBe(1820);
	});

	test('snaps the horizontal center to the canvas center', () => {
		// center = 906 + 50 = 956, canvas center 960, distance 4 -> snap; x = 960 - 50.
		expect(snapRect(R(906, 500, 100, 100), []).x).toBe(910);
	});

	test('snaps to a neighbor left edge', () => {
		const neighbor = R(300, 0, 200, 200);
		expect(snapRect(R(303, 700, 100, 100), [neighbor]).x).toBe(300);
	});

	test('snaps both axes independently and emits a guide per axis', () => {
		const r = snapRect(R(5, 5, 100, 100), []);
		expect(r.x).toBe(0);
		expect(r.y).toBe(0);
		expect(r.guides).toContainEqual({ axis: 'x', pos: 0 });
		expect(r.guides).toContainEqual({ axis: 'y', pos: 0 });
	});

	test('respects the threshold boundary (<= snaps, > does not)', () => {
		expect(snapRect(R(8, 500, 100, 100), []).x).toBe(0);
		expect(snapRect(R(9, 500, 100, 100), []).x).toBe(9);
	});
});
