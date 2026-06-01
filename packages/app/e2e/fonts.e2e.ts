import { expect, test } from '@playwright/test';

// The editor self-hosts Geist (vendored variable woff2) so the chrome renders the
// real typeface offline, not a system fallback. We grab the CSS-connected FontFace
// objects, force them to load, and assert they reach 'loaded' — a broken asset path
// would leave them in 'error', proving the woff2 actually serves from the relay.
test('the editor self-hosts and loads Geist Sans + Mono', async ({ browser }) => {
	const editor = await browser.newPage();
	await editor.goto('/edit');

	const status = await editor.evaluate(async () => {
		const faces = [...document.fonts];
		const sans = faces.find((face) => face.family === 'Geist');
		const mono = faces.find((face) => face.family === 'Geist Mono');
		await Promise.allSettled([sans?.load(), mono?.load()]);
		return { sans: sans?.status, mono: mono?.status };
	});

	expect(status.sans).toBe('loaded');
	expect(status.mono).toBe('loaded');
});
