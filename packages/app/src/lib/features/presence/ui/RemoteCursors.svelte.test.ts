import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import RemoteCursors from './RemoteCursors.svelte';

test('draws a labeled cursor only for peers that have one', async () => {
	render(RemoteCursors, {
		scale: 1,
		peers: [
			{ user: { id: 'a', name: 'Alice' }, cursor: { x: 100, y: 200 }, selection: [] },
			{ user: { id: 'b', name: 'Bob' }, selection: [] }
		]
	});

	// Alice has a cursor and is drawn; Bob has none and is not.
	await expect.element(page.getByText('Alice')).toBeVisible();
	await expect.element(page.getByText('Bob')).not.toBeInTheDocument();
});
