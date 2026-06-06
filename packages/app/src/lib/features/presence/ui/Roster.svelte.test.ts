import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Roster from './Roster.svelte';

test('shows a monogram per person and collapses the overflow', async () => {
	render(Roster, {
		max: 2,
		people: [
			{ id: 'a', name: 'Alice', you: true },
			{ id: 'b', name: 'Bob' },
			{ id: 'c', name: 'Carol' }
		]
	});

	await expect.element(page.getByText('AL')).toBeVisible();
	await expect.element(page.getByText('BO')).toBeVisible();
	// The third person is past `max`, so they collapse into a "+1" badge.
	await expect.element(page.getByText('+1')).toBeVisible();
	await expect.element(page.getByText('CA')).not.toBeInTheDocument();
});

test('marks you and the host, and is a button when it can open share', async () => {
	render(Roster, {
		onOpen: () => {},
		people: [
			{ id: 'a', name: 'Alice', you: true, host: true },
			{ id: 'b', name: 'Bob' }
		]
	});

	const stack = (await page.getByTestId('roster').element()) as HTMLElement;
	expect(stack.tagName).toBe('BUTTON');
	const you = stack.querySelector('.avatar.you');
	expect(you).not.toBeNull();
	expect(you?.querySelector('.pip')).not.toBeNull();
});

test('renders nothing when no one is present', () => {
	render(Roster, { people: [] });
	expect(page.getByTestId('roster').elements()).toHaveLength(0);
});
