import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Roster from './Roster.svelte';

test('shows an avatar per collaborator and collapses the overflow', async () => {
	render(Roster, {
		max: 2,
		peers: [
			{ user: { id: 'a', name: 'Alice' }, selection: [] },
			{ user: { id: 'b', name: 'Bob' }, selection: [] },
			{ user: { id: 'c', name: 'Carol' }, selection: [] }
		]
	});

	await expect.element(page.getByText('A')).toBeVisible();
	await expect.element(page.getByText('B')).toBeVisible();
	// The third peer is past `max`, so it collapses into a "+1" badge.
	await expect.element(page.getByText('+1')).toBeVisible();
	await expect.element(page.getByText('C')).not.toBeInTheDocument();
});
