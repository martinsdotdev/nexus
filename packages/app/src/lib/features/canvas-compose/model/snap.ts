// Snap-to-edge for the editor canvas (spec §11.3): align a moving widget's
// edges/center to the canvas edges/center and to other widgets' edges/centers
// when within a small threshold, emitting guide lines for the snapped axes.
// Pure + O(n) over the scene's other widgets. The driver computes neighbor
// rects once per drag and skips this call entirely while Alt is held.

import { VIRTUAL_H, VIRTUAL_W, type Size } from './geometry';

export interface Rect {
	x: number;
	y: number;
	w: number;
	h: number;
}

export interface Guide {
	axis: 'x' | 'y';
	pos: number;
}

export interface SnapResult {
	x: number;
	y: number;
	guides: Guide[];
}

const CANVAS: Size = { w: VIRTUAL_W, h: VIRTUAL_H };
const THRESHOLD = 8;

// Snap one axis: the moving rect's three anchors (start edge, center, far edge)
// against candidate lines; pick the nearest within threshold.
function snapAxis(
	pos: number,
	size: number,
	lines: number[],
	threshold: number
): { value: number; guide: number | null } {
	const anchors = [0, size / 2, size];
	let best: { value: number; guide: number; dist: number } | null = null;
	for (const offset of anchors) {
		const edge = pos + offset;
		for (const line of lines) {
			const dist = Math.abs(edge - line);
			if (dist <= threshold && (best === null || dist < best.dist)) {
				best = { value: line - offset, guide: line, dist };
			}
		}
	}
	return best ? { value: best.value, guide: best.guide } : { value: pos, guide: null };
}

export function snapRect(
	moving: Rect,
	neighbors: Rect[],
	threshold = THRESHOLD,
	canvas: Size = CANVAS
): SnapResult {
	const xLines = [0, canvas.w / 2, canvas.w];
	const yLines = [0, canvas.h / 2, canvas.h];
	for (const n of neighbors) {
		xLines.push(n.x, n.x + n.w / 2, n.x + n.w);
		yLines.push(n.y, n.y + n.h / 2, n.y + n.h);
	}

	const x = snapAxis(moving.x, moving.w, xLines, threshold);
	const y = snapAxis(moving.y, moving.h, yLines, threshold);

	const guides: Guide[] = [];
	if (x.guide !== null) guides.push({ axis: 'x', pos: x.guide });
	if (y.guide !== null) guides.push({ axis: 'y', pos: y.guide });
	return { x: x.value, y: y.value, guides };
}
