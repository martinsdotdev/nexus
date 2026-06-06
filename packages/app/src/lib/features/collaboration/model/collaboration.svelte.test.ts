import { afterEach, expect, test, vi } from 'vitest';
import { flushSync } from 'svelte';
import { createCollaboration, type Collaboration } from './collaboration.svelte';
import type { Member } from '../api/share';

const owner: Member = { user_id: 'me', display: 'me', role: 'owner' };
const editor: Member = { user_id: 'b', display: 'bob', role: 'editor' };

function stubFetch(handler: (init?: RequestInit) => Response) {
	vi.stubGlobal('fetch', async (_url: string, init?: RequestInit) => handler(init));
}

function json(body: unknown): Response {
	return new Response(JSON.stringify(body), {
		status: 200,
		headers: { 'content-type': 'application/json' }
	});
}

afterEach(() => vi.unstubAllGlobals());

test('loads members and derives the caller role + shareLive', async () => {
	stubFetch(() => json([owner, editor]));

	let collab!: Collaboration;
	const cleanup = $effect.root(() => {
		collab = createCollaboration('w1', 'me');
	});

	await collab.load();
	flushSync();

	expect(collab.members).toHaveLength(2);
	expect(collab.myRole).toBe('owner');
	expect(collab.shareLive).toBe(true); // two people present
	cleanup();
});

test('an invite to an unknown email returns a message', async () => {
	stubFetch((init) =>
		init?.method === 'POST' ? new Response(null, { status: 404 }) : json([owner])
	);

	let collab!: Collaboration;
	const cleanup = $effect.root(() => {
		collab = createCollaboration('w1', 'me');
	});

	expect(await collab.invite('ghost@example.com', 'editor')).toBe(
		'No Nexus account uses that email yet.'
	);
	cleanup();
});

test('a successful invite reloads the member list', async () => {
	let roster = [owner];
	stubFetch((init) => {
		if (init?.method === 'POST') {
			roster = [owner, editor]; // the invite took
			return new Response(null, { status: 204 });
		}
		return json(roster);
	});

	let collab!: Collaboration;
	const cleanup = $effect.root(() => {
		collab = createCollaboration('w1', 'me');
	});

	expect(await collab.invite('bob@example.com', 'editor')).toBeNull();
	flushSync();
	expect(collab.members).toHaveLength(2);
	cleanup();
});
