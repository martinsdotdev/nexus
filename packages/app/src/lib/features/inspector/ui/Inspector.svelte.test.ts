import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Inspector from './Inspector.svelte';
import type { SceneView, Theme, WidgetView } from '$lib/shared/crdt/workspace-view';

// The scene-theme dropdown lists the registry themes by name (ADR-0007), so the
// option a test selects must exist.
const themes: Theme[] = [
	{ id: 'cozy', name: 'Cozy', base: '', protected: true, tokens: {} },
	{ id: 'cyber', name: 'Cyber', base: '', protected: true, tokens: {} }
];

const widget = (): WidgetView => ({
	id: 'w1',
	widgetType: 'stream-info',
	x: 40,
	y: 900,
	w: 560,
	h: 96,
	z: 2,
	visible: true,
	props: { title: 'My Stream', game: 'Just Chatting' }
});

const scene = (): SceneView => ({
	id: 's1',
	kind: 'live',
	name: 'live',
	themeId: 'cozy',
	overridesAccent: '',
	overridesDensity: '',
	widgets: []
});

const handlers = () => ({
	onSetGeometry: vi.fn(),
	onSetProp: vi.fn(),
	onSetVisible: vi.fn(),
	onSetSceneTheme: vi.fn(),
	onSetSceneOverride: vi.fn()
});

// The inspector uses controlled inputs whose value resets when the spy does not
// write back, so we set value + dispatch the event directly rather than fill().
async function input(name: string) {
	return (await page.getByRole('textbox', { name }).element()) as HTMLInputElement;
}
async function spinbutton(name: string) {
	return (await page.getByRole('spinbutton', { name }).element()) as HTMLInputElement;
}

test('editing a text prop reports the new value', async () => {
	const h = handlers();
	render(Inspector, { widget: widget(), scene: null, themes, ...h });
	const el = await input('Title');
	el.value = 'New Title';
	el.dispatchEvent(new Event('input', { bubbles: true }));
	expect(h.onSetProp).toHaveBeenCalledWith('w1', 'title', 'New Title');
});

test('editing X reports the geometry change', async () => {
	const h = handlers();
	render(Inspector, { widget: widget(), scene: null, themes, ...h });
	const el = await spinbutton('X');
	el.value = '100';
	el.dispatchEvent(new Event('input', { bubbles: true }));
	expect(h.onSetGeometry).toHaveBeenCalledWith('w1', { x: 100 });
});

test('toggling visibility reports it', async () => {
	const h = handlers();
	render(Inspector, { widget: widget(), scene: null, themes, ...h });
	await page.getByRole('checkbox', { name: 'Visible' }).click();
	expect(h.onSetVisible).toHaveBeenCalledWith('w1', false);
});

test('with no widget, changing the scene theme reports it', async () => {
	const h = handlers();
	render(Inspector, { widget: null, scene: scene(), themes, ...h });
	const select = (await page
		.getByRole('combobox', { name: 'Theme' })
		.element()) as HTMLSelectElement;
	select.value = 'cyber';
	select.dispatchEvent(new Event('change', { bubbles: true }));
	expect(h.onSetSceneTheme).toHaveBeenCalledWith('s1', 'cyber');
});
