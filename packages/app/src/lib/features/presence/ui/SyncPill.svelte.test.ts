import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import SyncPill from './SyncPill.svelte';

test('labels the synced state with a colored dot', async () => {
	render(SyncPill, { state: 'synced' });
	await expect.element(page.getByText('Synced')).toBeVisible();
	const pill = (await page.getByTestId('sync-pill').element()) as HTMLElement;
	expect(pill.querySelector('.dot')).not.toBeNull();
});

test('shows an icon (no dot) when working offline', async () => {
	render(SyncPill, { state: 'offline' });
	await expect.element(page.getByText('Working offline')).toBeVisible();
	const pill = (await page.getByTestId('sync-pill').element()) as HTMLElement;
	expect(pill.querySelector('.dot')).toBeNull();
	expect(pill.querySelector('svg')).not.toBeNull();
});

test('keeps a dot while reconnecting', async () => {
	render(SyncPill, { state: 'reconnecting' });
	const pill = (await page.getByTestId('sync-pill').element()) as HTMLElement;
	expect(pill.querySelector('.dot')).not.toBeNull();
});

test('is a button only when it can open share', async () => {
	render(SyncPill, { state: 'synced', onClick: () => {} });
	const pill = (await page.getByTestId('sync-pill').element()) as HTMLElement;
	expect(pill.tagName).toBe('BUTTON');
});
