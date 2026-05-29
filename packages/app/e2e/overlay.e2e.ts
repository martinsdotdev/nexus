import { expect, test } from '@playwright/test';

// The overlay renders the active scene's themed composition and reflects editor
// scene switches live (the editor -> OBS loop), re-theming by swapping the
// resolved CSS vars it carries inline (every theme is data, ADR-0007). Default
// scenes: live=cozy, starting_soon=cyber, brb=editorial, ending=sticker; the
// live scene seeds the eight v1 widgets.
test('overlay renders the composition and re-themes when the editor switches scene', async ({
	browser
}) => {
	const editor = await browser.newPage();
	const overlay = await browser.newPage();
	await editor.goto('/edit');
	await overlay.goto('/overlay');

	const editorCards = editor.locator('.scene-card');
	await expect(editorCards).toHaveCount(4);

	// Activate "live" explicitly: the shared relay's active scene may have been
	// left elsewhere by a prior test. Live is cozy and seeds the eight widgets,
	// so the overlay must mirror exactly that composition.
	await editorCards.nth(0).click();
	const canvas = overlay.locator('.overlay-canvas');
	await expect(canvas).toBeVisible();
	// cozy's primary, inlined as a CSS var on the canvas root.
	await expect(canvas).toHaveAttribute('style', /--primary:\s*oklch\(70% 0\.15 50\)/);
	await expect(overlay.locator('.widget-slot')).toHaveCount(8);

	// Switch the editor to "starting_soon" (cyber); the overlay re-themes live.
	await editorCards.nth(1).click();
	await expect(canvas).toHaveAttribute('style', /--primary:\s*oklch\(75% 0\.2 200\)/);
});
