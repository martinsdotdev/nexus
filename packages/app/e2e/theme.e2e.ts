import { expect, test } from '@playwright/test';

// The theme builder loop: duplicate a built-in into a custom theme, edit a token,
// and confirm the custom theme (its inline CSS vars) reaches the overlay live.
test('a custom theme authored in the editor renders on the overlay', async ({ browser }) => {
	const editor = await browser.newPage();
	// Wide so the inspector/theme panel is docked (not an off-canvas drawer).
	await editor.setViewportSize({ width: 1440, height: 900 });
	const overlay = await browser.newPage();
	await editor.goto('/edit');
	await overlay.goto('/overlay');

	// Activate live (the overlay mirrors the active scene); no widget is selected,
	// so the inspector shows scene controls with the "Customize theme" entry.
	const cards = editor.locator('.scene-card');
	await expect(cards).toHaveCount(4);
	await cards.nth(0).click();

	const panel = editor.locator('.dock-right');
	await panel.getByRole('button', { name: 'Customize theme' }).click();
	await panel.getByRole('button', { name: 'Duplicate to customize' }).click();
	await panel.getByRole('textbox', { name: 'primary', exact: true }).fill('rgb(1, 2, 3)');

	// The overlay's canvas carries the custom theme inline (base data-theme + vars).
	await expect(overlay.locator('.overlay-canvas')).toHaveAttribute(
		'style',
		/--primary:\s*rgb\(1, 2, 3\)/
	);
});
