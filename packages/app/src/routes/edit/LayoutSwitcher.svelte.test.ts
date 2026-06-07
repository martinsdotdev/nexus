import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import LayoutSwitcher from './LayoutSwitcher.svelte';

function base(overrides: Record<string, unknown> = {}) {
	return {
		layouts: [
			{ id: 'l1', name: 'Main' },
			{ id: 'l2', name: 'Vertical' }
		],
		activeLayoutId: 'l1',
		onSwitch: vi.fn(),
		onCreate: vi.fn(),
		onDuplicate: vi.fn(),
		onRename: vi.fn(),
		onArchive: vi.fn(),
		onDelete: vi.fn(),
		open: true,
		onOpenChange: vi.fn(),
		...overrides
	};
}

test('choosing a layout reports the switch', async () => {
	const props = base();
	render(LayoutSwitcher, props);
	await page.getByRole('menuitem', { name: 'Vertical' }).click();
	expect(props.onSwitch).toHaveBeenCalledWith('l2');
});

test('the New layout action is offered', async () => {
	const props = base();
	render(LayoutSwitcher, props);
	await page.getByRole('menuitem', { name: 'New layout' }).click();
	expect(props.onCreate).toHaveBeenCalled();
});

test('the Delete action removes the active layout', async () => {
	const props = base();
	render(LayoutSwitcher, props);
	await page.getByRole('menuitem', { name: 'Delete' }).click();
	expect(props.onDelete).toHaveBeenCalled();
});

test('Delete is not offered with a single layout', async () => {
	const props = base({ layouts: [{ id: 'l1', name: 'Main' }] });
	render(LayoutSwitcher, props);
	await expect.element(page.getByRole('menuitem', { name: 'Delete' })).not.toBeInTheDocument();
});

test('Rename opens an inline editor for the active layout', async () => {
	const props = base();
	render(LayoutSwitcher, props);
	await page.getByRole('menuitem', { name: 'Rename' }).click();
	await expect.element(page.getByRole('textbox', { name: 'Layout name' })).toHaveValue('Main');
});
