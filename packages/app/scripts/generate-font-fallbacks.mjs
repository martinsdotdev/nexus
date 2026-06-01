/*
 * Generates metric-adjusted fallback @font-face descriptors for the vendored Geist
 * fonts. With font-display: swap, the browser paints a fallback first, then swaps in
 * Geist; if their metrics differ the text reflows (layout shift / CLS). An adjusted
 * fallback @font-face (size-adjust + ascent/descent/line-gap overrides) makes the
 * fallback occupy exactly Geist's space, so the swap is invisible.
 *
 * Run: `node scripts/generate-font-fallbacks.mjs`
 * Then paste the printed @font-face blocks into src/lib/shared/styles/fonts.css and
 * the printed font-family stacks into --font-sans / --font-mono in tokens.css.
 *
 * Capsize does the real work: unpack reads each font's metrics (incl. the average
 * advance width) from the woff2, and createFontStack derives the override descriptors
 * against a system fallback (Arial for sans, Courier New for mono).
 */
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { fromFile } from '@capsizecss/unpack/fs';
import { createFontStack } from '@capsizecss/core';
import arial from '@capsizecss/metrics/arial';
import courierNew from '@capsizecss/metrics/courierNew';

const here = path.dirname(fileURLToPath(import.meta.url));
const fontsDir = path.join(here, '..', 'src', 'lib', 'shared', 'styles', 'fonts');

const geist = await fromFile(path.join(fontsDir, 'Geist-Variable.woff2'));
const geistMono = await fromFile(path.join(fontsDir, 'GeistMono-Variable.woff2'));

const sans = createFontStack([geist, arial]);
const mono = createFontStack([geistMono, courierNew]);

console.log('==== SANS (Geist) ====');
console.log('font-family:', sans.fontFamily);
console.log(sans.fontFaces);
console.log('\n==== MONO (Geist Mono) ====');
console.log('font-family:', mono.fontFamily);
console.log(mono.fontFaces);
