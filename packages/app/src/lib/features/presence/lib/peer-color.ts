// A stable, distinct color per collaborator, derived from their account id so a peer's
// cursor, selection outline, and roster chip all match. OKLCH at a fixed lightness and
// chroma spreads the hue evenly and keeps every peer color equally legible.
export function peerColor(id: string): string {
	let hash = 0;
	for (let i = 0; i < id.length; i++) {
		hash = (Math.imul(hash, 31) + id.charCodeAt(i)) | 0;
	}
	const hue = Math.abs(hash) % 360;
	return `oklch(65% 0.19 ${hue})`;
}
