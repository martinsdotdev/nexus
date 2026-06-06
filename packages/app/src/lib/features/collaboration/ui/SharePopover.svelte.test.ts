import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import SharePopover from './SharePopover.svelte';
import type { Member } from '../api/share';

const members: Member[] = [
	{ user_id: 'u1', display: 'mara', role: 'owner' },
	{ user_id: 'u2', display: 'dub', role: 'editor' }
];

const noop = () => {};
const asyncNoop = async () => null;

function base() {
	return {
		members,
		watchLink: null,
		onClose: noop,
		onInvite: asyncNoop,
		onSetRole: noop,
		onRemove: noop,
		onCreateWatchLink: noop
	};
}

test('lists members, tags you, and gives the owner role controls', async () => {
	render(SharePopover, { ...base(), selfId: 'u1', myRole: 'owner' });

	const pop = (await page.getByTestId('share-popover').element()) as HTMLElement;
	expect(pop.textContent).toContain('mara');
	expect(pop.textContent).toContain('dub');
	expect(pop.textContent).toContain('you');
	// The owner can change the editor's role and invite by email.
	expect(pop.querySelector('select')).not.toBeNull();
	expect(pop.querySelector('input[type="email"]')).not.toBeNull();
});

test('a non-owner sees roles but no management controls', async () => {
	render(SharePopover, { ...base(), selfId: 'u2', myRole: 'editor' });

	const pop = (await page.getByTestId('share-popover').element()) as HTMLElement;
	expect(pop.querySelector('select')).toBeNull();
	expect(pop.querySelector('input[type="email"]')).toBeNull();
});

test('shows the watch link when one exists', async () => {
	render(SharePopover, {
		...base(),
		selfId: 'u1',
		myRole: 'owner',
		watchLink: 'http://x/overlay?token=abc'
	});

	const pop = (await page.getByTestId('share-popover').element()) as HTMLElement;
	expect(pop.textContent).toContain('http://x/overlay?token=abc');
});
