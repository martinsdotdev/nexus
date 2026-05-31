import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { createRawSnippet } from 'svelte';
import Drawer from './Drawer.svelte';

const body = createRawSnippet(() => ({ render: () => '<p>Drawer body</p>' }));

// The wrapper wires Ark Dialog behind {open, onClose, title, children}: the focus
// trap, Escape, scrim, and restore-focus are Ark's; we verify our wiring.
test('renders title + body when open and closes via the close button', async () => {
	const onClose = vi.fn();
	render(Drawer, { open: true, onClose, side: 'left', title: 'Widgets', children: body });

	await expect.element(page.getByText('Drawer body')).toBeInTheDocument();
	await page.getByRole('button', { name: 'Close Widgets' }).click();
	expect(onClose).toHaveBeenCalled();
});
