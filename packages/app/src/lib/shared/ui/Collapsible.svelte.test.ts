import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { createRawSnippet } from 'svelte';
import Collapsible from './Collapsible.svelte';

const body = createRawSnippet(() => ({ render: () => '<p>Group body</p>' }));

test('renders the content when open, with the title on the trigger', async () => {
	render(Collapsible, { title: 'Surfaces', open: true, children: body });
	await expect.element(page.getByText('Group body')).toBeInTheDocument();
	await expect.element(page.getByRole('button', { name: 'Surfaces' })).toBeInTheDocument();
});
