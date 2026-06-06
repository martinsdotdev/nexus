import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import LayoutSwitcher from './LayoutSwitcher.svelte';

function base() {
	return {
		layouts: [
			{ id: 'l1', name: 'Main' },
			{ id: 'l2', name: 'Vertical' }
		],
		activeLayoutId: 'l1',
		onSwitch: vi.fn(),
		onCreate: vi.fn(),
		onDuplicate: vi.fn(),
		onArchive: vi.fn(),
		open: true,
		onOpenChange: vi.fn()
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
