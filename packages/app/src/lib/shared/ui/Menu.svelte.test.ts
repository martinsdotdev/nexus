import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Menu from './Menu.svelte';

const items = [
	{ value: 'duplicate', label: 'Duplicate' },
	{ value: 'delete', label: 'Delete', destructive: true }
];

test('opens on the trigger and reports the chosen item', async () => {
	const onSelect = vi.fn();
	render(Menu, { items, onSelect, label: 'Theme actions' });

	await page.getByRole('button', { name: 'Theme actions' }).click();
	await page.getByRole('menuitem', { name: 'Duplicate' }).click();

	expect(onSelect).toHaveBeenCalledWith('duplicate');
});
