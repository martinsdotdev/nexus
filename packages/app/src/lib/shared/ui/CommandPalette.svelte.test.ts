import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import CommandPalette from './CommandPalette.svelte';

const items = [
	{ id: 'add-widget', label: 'Add widget' },
	{ id: 'switch-theme', label: 'Switch theme' }
];

test('clicking a command reports its id and closes the palette', async () => {
	const onSelect = vi.fn();
	const onClose = vi.fn();
	render(CommandPalette, { open: true, onClose, items, onSelect });

	await page.getByRole('button', { name: 'Switch theme' }).click();
	expect(onSelect).toHaveBeenCalledWith('switch-theme');
	expect(onClose).toHaveBeenCalled();
});
