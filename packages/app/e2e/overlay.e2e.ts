import { expect, test } from '@playwright/test';

// The overlay renders the active scene's themed composition and reflects editor
// scene switches live (the editor -> OBS loop), re-theming with zero JS via the
// [data-theme] cascade. Default scenes: live=cozy, starting_soon=cyber,
// brb=editorial, ending=sticker; the live scene seeds the eight v1 widgets.
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
	await expect(canvas).toHaveAttribute('data-theme', 'cozy');
	await expect(overlay.locator('.widget-slot')).toHaveCount(8);

	// Switch the editor to "starting_soon" (cyber); the overlay re-themes live.
	await editorCards.nth(1).click();
	await expect(canvas).toHaveAttribute('data-theme', 'cyber');
});
