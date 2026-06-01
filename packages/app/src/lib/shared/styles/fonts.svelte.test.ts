import { expect, test } from 'vitest';
import './fonts.css';

// The editor self-hosts Geist so --font-sans / --font-mono (which name 'Geist' and
// 'Geist Mono') render the real typeface offline, not a system fallback. Importing
// the global stylesheet registers its @font-face rules as CSS-connected FontFace
// entries in document.fonts; we assert both families are present.
test('registers Geist Sans and Geist Mono via @font-face', () => {
	const families = new Set([...document.fonts].map((face) => face.family));
	expect(families.has('Geist')).toBe(true);
	expect(families.has('Geist Mono')).toBe(true);
});
