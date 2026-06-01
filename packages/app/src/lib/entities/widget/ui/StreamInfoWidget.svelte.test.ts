import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import StreamInfoWidget from './StreamInfoWidget.svelte';
import { createEventBus } from '$lib/shared/events/event-bus';
import type { WidgetView } from '$lib/shared/crdt/workspace-view';

function makeInstance(): WidgetView {
	return {
		id: 'info-1',
		widgetType: 'stream-info',
		x: 0,
		y: 0,
		w: 400,
		h: 120,
		z: 0,
		visible: true,
		props: { title: 'My Stream', viewers: 1280 }
	};
}

// The live viewer count updates via stream.info-changed; tabular figures keep the
// digits aligned so the count does not change width as viewers come and go.
test('renders the viewer count with tabular figures', async () => {
	const bus = createEventBus();
	render(StreamInfoWidget, { instance: makeInstance(), bus });

	const viewers = page.getByText('1280 watching');
	await expect.element(viewers).toBeVisible();
	expect(getComputedStyle(viewers.element()).fontVariantNumeric).toBe('tabular-nums');
});
