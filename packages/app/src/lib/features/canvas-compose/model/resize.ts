// Pure resize math: apply a pointer delta (in virtual px) to one of the eight
// handles, returning the new rect. Each affected edge moves independently and a
// minimum size is enforced by clamping the moving edge so it never crosses its
// opposite. The component owns the pointer wiring; this is the geometry.

import type { Rect } from './snap';

export type ResizeHandle = 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w';

const MIN_SIZE = 24;

export function resizeRect(
	rect: Rect,
	handle: ResizeHandle,
	dx: number,
	dy: number,
	min = MIN_SIZE
): Rect {
	let { x, y, w, h } = rect;
	const right = x + w;
	const bottom = y + h;

	if (handle.includes('w')) {
		x = Math.min(x + dx, right - min);
		w = right - x;
	}
	if (handle.includes('e')) {
		w = Math.max(min, w + dx);
	}
	if (handle.includes('n')) {
		y = Math.min(y + dy, bottom - min);
		h = bottom - y;
	}
	if (handle.includes('s')) {
		h = Math.max(min, h + dy);
	}
	return { x, y, w, h };
}
