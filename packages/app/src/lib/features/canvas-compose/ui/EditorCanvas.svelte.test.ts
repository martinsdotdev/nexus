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
		layouts: [{ id: 'l1', name: 'Main', status: 'active', activeSceneId: 's1', scenes: [scene] }],
		themes: []
	};
}

test('renders a slot per widget and reports the clicked widget id', async () => {
	const onSelectWidget = vi.fn();
	render(EditorCanvas, {
		view: fixtureView(),
		selectedWidgetId: null,
		onSelectWidget,
		onCommitGeometry: vi.fn()
	});

	await expect.element(page.getByRole('button', { name: 'stream-info' })).toBeVisible();
	await expect.element(page.getByRole('button', { name: 'goal-bar' })).toBeVisible();

	await page.getByRole('button', { name: 'goal-bar' }).click();
	expect(onSelectWidget).toHaveBeenCalledWith('w2');
});

test('renders the selection outline for the selected widget', async () => {
	render(EditorCanvas, {
		view: fixtureView(),
		selectedWidgetId: 'w1',
		onSelectWidget: vi.fn(),
		onCommitGeometry: vi.fn()
	});
	await expect.element(page.getByTestId('selection')).toBeVisible();
});

test('dragging a widget commits the new geometry exactly once (size preserved)', async () => {
	const onCommitGeometry = vi.fn();
	render(EditorCanvas, {
		view: fixtureView(),
		selectedWidgetId: null,
		onSelectWidget: vi.fn(),
		onCommitGeometry
	});

	const el = (await page.getByRole('button', { name: 'goal-bar' }).element()) as HTMLElement;
	const box = el.getBoundingClientRect();
	const cx = box.left + box.width / 2;
	const cy = box.top + box.height / 2;

	el.dispatchEvent(new PointerEvent('pointerdown', { clientX: cx, clientY: cy, bubbles: true }));
	window.dispatchEvent(
		new PointerEvent('pointermove', { clientX: cx + 80, clientY: cy + 60, bubbles: true })
	);
	window.dispatchEvent(
		new PointerEvent('pointerup', { clientX: cx + 80, clientY: cy + 60, bubbles: true })
	);

	expect(onCommitGeometry).toHaveBeenCalledTimes(1);
	// A move preserves size; only x/y change (exact x/y depend on scale + snap).
	expect(onCommitGeometry).toHaveBeenCalledWith('w2', expect.objectContaining({ w: 520, h: 56 }));
});
