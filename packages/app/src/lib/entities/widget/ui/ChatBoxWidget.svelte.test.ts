import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { tick } from 'svelte';
import ChatBoxWidget from './ChatBoxWidget.svelte';
import { createEventBus } from '$lib/shared/events/event-bus';
import type { WidgetView } from '$lib/shared/crdt/workspace-view';

function makeInstance(): WidgetView {
	return {
		id: 'chat-1',
		widgetType: 'chat-box',
		x: 0,
		y: 0,
		w: 400,
		h: 300,
		z: 0,
		visible: true,
		props: { title: 'Chat' }
	};
}

// Regression: a real chat burst (or a batched source) can emit two messages in
// the same millisecond. Keying the each block by `timestamp` made those keys
// collide and Svelte threw `each_key_duplicate`, crashing the overlay in OBS.
// The widget must render both messages regardless of timestamp collisions.
test('renders messages that share a timestamp without a duplicate-key crash', async () => {
	const bus = createEventBus();
	render(ChatBoxWidget, { instance: makeInstance(), bus });
	await tick(); // let the subscribe $effect attach before emitting

	const sameTimestamp = 1_700_000_000_000;
	bus.emit({
		kind: 'chat.message',
		payload: {
			user: 'alice',
			text: 'first message',
			color: '',
			badges: [],
			timestamp: sameTimestamp
		}
	});
	bus.emit({
		kind: 'chat.message',
		payload: {
			user: 'bob',
			text: 'second message',
			color: '',
			badges: [],
			timestamp: sameTimestamp
		}
	});

	await expect.element(page.getByText('first message')).toBeVisible();
	await expect.element(page.getByText('second message')).toBeVisible();
});
