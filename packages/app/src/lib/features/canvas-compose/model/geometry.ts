// Pure coordinate math for the editor canvas: fit the 1920x1080 virtual surface
// into the canvas area (letterboxed) and convert pointer client coordinates into
// virtual pixels. No DOM, the caller passes the live bounding rect + scale, so
// this stays headless-testable and reusable by the drag/resize drivers.

import { VIRTUAL_W, VIRTUAL_H } from '$lib/shared/config/canvas';

export { VIRTUAL_W, VIRTUAL_H };

export interface Size {
	w: number;
	h: number;
}

export interface Point {
	x: number;
	y: number;
}

/** Largest scale that fits the virtual surface inside the container (letterbox). */
export function fitScale(container: Size, virtual: Size = { w: VIRTUAL_W, h: VIRTUAL_H }): number {
	if (container.w <= 0 || container.h <= 0) return 0;
	return Math.min(container.w / virtual.w, container.h / virtual.h);
}

/** Convert a client-space point to virtual pixels given the canvas origin + scale. */
export function clientToVirtual(client: Point, origin: Point, scale: number): Point {
	if (scale <= 0) return { x: 0, y: 0 };
	return { x: (client.x - origin.x) / scale, y: (client.y - origin.y) / scale };
}

export function clamp(value: number, min: number, max: number): number {
	return Math.max(min, Math.min(max, value));
}
