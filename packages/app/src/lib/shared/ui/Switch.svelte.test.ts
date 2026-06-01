import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Switch from './Switch.svelte';

test('clicking the switch reports the toggled state', async () => {
	const onChange = vi.fn();
	render(Switch, { checked: false, onChange, label: 'Visible' });
	// The accessible control is a native checkbox (no role="switch"), but it is
	// sr-only; a user clicks the visible track/label, which the <label> root toggles.
	await page.getByText('Visible').click();
	expect(onChange).toHaveBeenCalledWith(true);
});
