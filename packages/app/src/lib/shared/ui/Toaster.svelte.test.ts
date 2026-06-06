import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Toaster from './Toaster.svelte';
import { toast } from './toast';

test('a created toast renders its title and description', async () => {
	render(Toaster);
	toast.success('Workspace created', 'Opening it now');
	await expect.element(page.getByText('Workspace created')).toBeVisible();
	await expect.element(page.getByText('Opening it now')).toBeVisible();
});
