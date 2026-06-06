import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import type { WidgetView } from '$lib/shared/crdt/workspace-view';
import RemoteSelections from './RemoteSelections.svelte';

const widget = (id: string): WidgetView => ({
	id,
	widgetType: 'stream-info',
	x: 0,
	y: 0,
	w: 100,
	h: 50,
	z: 1,
	visible: true,
	props: {}
});

test('outlines each existing widget a peer has selected', async () => {
	render(RemoteSelections, {
		widgets: [widget('w1'), widget('w2')],
		// Alice selects an existing widget and a deleted one; only the existing draws.
		peers: [{ user: { id: 'a', name: 'Alice' }, selection: ['w1', 'ghost'] }]
	});

	const container = (await page.getByTestId('remote-selections').element()) as HTMLElement;
	expect(container.querySelectorAll('.outline')).toHaveLength(1);
	// The outline carries a name tag identifying the collaborator.
	expect(container.querySelector('.outline .tag')?.textContent).toBe('Alice');
});
