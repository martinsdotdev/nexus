import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Field from './Field.svelte';

test('renders the label and the current value', async () => {
	render(Field, { label: 'Email', value: 'a@b.com', oninput: () => {} });
	await expect.element(page.getByText('Email')).toBeVisible();
	await expect.element(page.getByRole('textbox')).toHaveValue('a@b.com');
});

test('shows the error text and marks the control invalid', async () => {
	render(Field, { label: 'Email', value: 'x', oninput: () => {}, error: 'Enter a valid email' });
	await expect.element(page.getByText('Enter a valid email')).toBeVisible();
	await expect.element(page.getByRole('textbox')).toHaveAttribute('aria-invalid', 'true');
});

test('falls back to the hint when there is no error', async () => {
	render(Field, { label: 'Email', value: '', oninput: () => {}, hint: 'We never share it' });
	await expect.element(page.getByText('We never share it')).toBeVisible();
});

test('reports typed input through oninput', async () => {
	const oninput = vi.fn();
	render(Field, { label: 'Name', value: '', oninput });
	await page.getByRole('textbox').fill('hi');
	expect(oninput).toHaveBeenCalled();
	expect(oninput).toHaveBeenLastCalledWith('hi');
});
