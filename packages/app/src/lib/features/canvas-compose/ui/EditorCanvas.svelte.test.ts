import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import EditorCanvas from './EditorCanvas.svelte';
import type { WorkspaceView } from '$lib/shared/crdt/workspace-view';

function fixtureView(): WorkspaceView {
	const widgets = [
		{
			id: 'w1',
			widgetType: 'stream-info',
			x: 40,
			y: 900,
			w: 560,
			h: 96,
			z: 2,
			visible: true,
			props: { title: 'My Stream', game: 'Just Chatting' }
		},
		{
			id: 'w2',
			widgetType: 'goal-bar',
			x: 620,
			y: 940,
			w: 520,
			h: 56,
			z: 2,
			visible: true,
			props: { label: 'Goal', current: 1, target: 10 }
		}
	];
	const scene = {
		id: 's1',
		kind: 'live',
		name: 'live',
		themeId: 'cozy',
		overridesAccent: '',
		overridesDensity: '',
		widgets
	};
	return {
		activeLayoutId: 'l1',
		activeSceneId: 's1',
		scenes: [scene],
		layouts: [{ id: 'l1', name: 'Main', activeSceneId: 's1', scenes: [scene] }]
	};
}

test('renders a slot per widget and reports the clicked widget id', async () => {
	const onSelectWidget = vi.fn();
	render(EditorCanvas, { view: fixtureView(), selectedWidgetId: null, onSelectWidget });

	await expect.element(page.getByRole('button', { name: 'stream-info' })).toBeVisible();
	await expect.element(page.getByRole('button', { name: 'goal-bar' })).toBeVisible();

	await page.getByRole('button', { name: 'goal-bar' }).click();
	expect(onSelectWidget).toHaveBeenCalledWith('w2');
});

test('renders the selection outline for the selected widget', async () => {
	render(EditorCanvas, { view: fixtureView(), selectedWidgetId: 'w1', onSelectWidget: vi.fn() });
	await expect.element(page.getByTestId('selection')).toBeVisible();
});
