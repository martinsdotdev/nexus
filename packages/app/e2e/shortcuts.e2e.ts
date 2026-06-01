import { expect, test } from '@playwright/test';

// The command palette adds widgets; the titlebar undoes; the keyboard deletes.
// Operations are spaced (separate assertions) so undo steps do not merge.
test('palette adds a widget, the titlebar undoes, and Delete removes one', async ({ browser }) => {
	const editor = await browser.newPage();
	await editor.setViewportSize({ width: 1440, height: 900 });
	await editor.goto('/edit');

	const cards = editor.locator('.scene-card');
	await expect(cards).toHaveCount(4);
	await cards.nth(0).click(); // live: the eight seeded widgets
	const slots = editor.locator('.widget-slot');
	await expect(slots).toHaveCount(8);

	// Add a widget via the command palette (Cmd/Ctrl-K -> "Add widget").
	await editor.keyboard.press('Control+k');
	await editor.getByRole('button', { name: 'Add widget' }).click();
	await expect(slots).toHaveCount(9);

	// Undo via the titlebar button.
	await editor.getByRole('button', { name: 'Undo' }).click();
	await expect(slots).toHaveCount(8);

	// Select a widget and delete it with the keyboard.
	await slots.first().click();
	await editor.keyboard.press('Delete');
	await expect(slots).toHaveCount(7);
});

// The centered command bar in the titlebar opens the same palette as Cmd/Ctrl-K.
test('the titlebar command bar opens the palette', async ({ browser }) => {
	const editor = await browser.newPage();
	await editor.setViewportSize({ width: 1440, height: 900 });
	await editor.goto('/edit');

	// Closed to start: the palette's search field is not mounted (unmountOnExit).
	const search = editor.getByRole('textbox', { name: 'Command search' });
	await expect(search).toHaveCount(0);

	// Clicking the command bar opens the palette and focuses its search field.
	await editor.getByRole('button', { name: 'Search commands' }).click();
	await expect(search).toBeVisible();
});
