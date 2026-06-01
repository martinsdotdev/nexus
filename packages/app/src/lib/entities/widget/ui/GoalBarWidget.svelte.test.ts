import { expect, test } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import GoalBarWidget from './GoalBarWidget.svelte';
import { createEventBus } from '$lib/shared/events/event-bus';
import type { WidgetView } from '$lib/shared/crdt/workspace-view';

function makeInstance(): WidgetView {
	return {
		id: 'goal-1',
		widgetType: 'goal-bar',
		x: 0,
		y: 0,
		w: 400,
		h: 120,
		z: 0,
		visible: true,
		props: { label: 'Goal', current: 0, target: 100 }
	};
}

// The live counter ('current / target') ticks up via goal.increment; tabular figures
// give each digit a fixed advance width so the number doesn't jitter as it changes.
test('renders the goal count with tabular figures', async () => {
	const bus = createEventBus();
	render(GoalBarWidget, { instance: makeInstance(), bus });

	const count = page.getByText('0 / 100');
	await expect.element(count).toBeVisible();
	expect(getComputedStyle(count.element()).fontVariantNumeric).toBe('tabular-nums');
});
