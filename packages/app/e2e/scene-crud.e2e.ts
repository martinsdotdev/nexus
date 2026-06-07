import { expect, test } from '@playwright/test';

// Scene CRUD through the full stack: the editor's SceneStrip drives the local Loro
// replica and the relay. Adds a scene, renames it inline, then deletes it, leaving
// the shared workspace exactly as found (so the count-sensitive tests still see four
// scenes). State-robust: it measures the starting count rather than assuming four.
test('a scene can be added, renamed inline, and deleted from the strip', async ({ browser }) => {
	const editor = await browser.newPage();
	await editor.setViewportSize({ width: 1440, height: 900 });
	await editor.goto('/edit');

	const cards = editor.locator('.scene-card');
	// Wait for the local replica to sync the seeded scenes before measuring.
	await expect(cards.first()).toBeVisible();
	const initial = await cards.count();

	// Add a scene: it appends to the strip and becomes active.
	await editor.getByRole('button', { name: 'Add scene' }).click();
	await expect(cards).toHaveCount(initial + 1);
	await expect(cards.nth(initial)).toHaveAttribute('aria-pressed', 'true');

	// Rename it inline (double-click the card, type, commit with Enter).
	await cards.nth(initial).dblclick();
	const nameField = editor.getByRole('textbox', { name: 'Scene name' });
	await nameField.fill('Encore');
	await nameField.press('Enter');
	await expect(cards.nth(initial)).toHaveText(/Encore/);

	// Delete it via its action menu; the strip returns to the initial count.
	await editor.locator('.scene-kebab').nth(initial).click();
	await editor.getByRole('menuitem', { name: 'Delete' }).click();
	await expect(cards).toHaveCount(initial);
});
