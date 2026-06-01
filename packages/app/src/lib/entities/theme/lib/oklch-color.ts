// The bridge between the oklch token format (the theme vocabulary, ADR-0007) and
// the sRGB space Ark UI's Color Picker speaks (ADR-0008). Zag has no oklch support,
// so we convert at the picker boundary with culori. Converting oklch -> hex clamps
// to the sRGB gamut, so a wide-gamut token (e.g. a neon oklch) loses saturation
// when edited via the visual picker (the cost of "visual picker only"). Pure.
import { formatHex, oklch as toOklch } from 'culori';

/** An oklch token string -> an sRGB hex string for the picker (gamut-clamped). */
export function oklchToHex(oklchString: string): string {
	return formatHex(oklchString) ?? '#000000';
}

/** Any CSS color string from the picker -> an oklch token string with percentage
 *  L (matching the seeded built-ins); an achromatic color folds to hue 0. An
 *  unparseable input is returned unchanged. */
export function colorToOklch(colorString: string): string {
	const color = toOklch(colorString);
	if (!color) return colorString;
	const l = Math.round((color.l ?? 0) * 1000) / 10;
	const c = Math.round((color.c ?? 0) * 1000) / 1000;
	const h = Math.round((color.h ?? 0) * 10) / 10;
	return `oklch(${l}% ${c} ${h})`;
}
