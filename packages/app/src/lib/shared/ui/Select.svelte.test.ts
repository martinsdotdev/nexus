import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Select from './Select.svelte';

const options = [
	{ value: 'cozy', label: 'Cozy' },
	{ value: 'cyber', label: 'Cyber' }
];

// The wrapper's contract: a plain {value: string, onChange(value)} API over Ark's
// string[] + details-object model. Driving it as a user (open the listbox, pick an
// option) must report the chosen value as a bare string.
test('selecting an option reports its value through the single-string API', async () => {
	const onChange = vi.fn();
	render(Select, { value: 'cozy', options, onChange, ariaLabel: 'Theme' });

	await page.getByRole('combobox', { name: 'Theme' }).click();
	await page.getByRole('option', { name: 'Cyber' }).click();

	expect(onChange).toHaveBeenCalledWith('cyber');
});
