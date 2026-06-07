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

// The move-to-scene targets (the active layout's other scenes).
const moveTargets = [{ id: 's2', label: 'BRB' }];

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
	onDuplicate: vi.fn(),
	onMoveToScene: vi.fn(),
	onSetSceneTheme: vi.fn(),
	onSetSceneOverride: vi.fn()
});

// The inspector uses controlled inputs whose value resets when the spy does not
// write back, so we set value + dispatch the event directly rather than fill().
async function input(name: string) {
	return (await page.getByRole('textbox', { name }).element()) as HTMLInputElement;
}

test('editing a text prop reports the new value', async () => {
	const h = handlers();
	render(Inspector, { widget: widget(), scene: null, themes, moveTargets, ...h });
	const el = await input('Title');
	el.value = 'New Title';
	el.dispatchEvent(new Event('input', { bubbles: true }));
	expect(h.onSetProp).toHaveBeenCalledWith('w1', 'title', 'New Title');
});

test('editing X reports the geometry change', async () => {
	const h = handlers();
	render(Inspector, { widget: widget(), scene: null, themes, moveTargets, ...h });
	// Ark NumberInput (role spinbutton) owns its value; type into it via fill.
	await page.getByRole('spinbutton', { name: 'X' }).fill('100');
	expect(h.onSetGeometry).toHaveBeenCalledWith('w1', { x: 100 });
});

test('toggling visibility reports it', async () => {
	const h = handlers();
	render(Inspector, { widget: widget(), scene: null, themes, moveTargets, ...h });
	// Ark Switch: the checkbox is sr-only; click the visible label/track to toggle.
	await page.getByText('Visible').click();
	expect(h.onSetVisible).toHaveBeenCalledWith('w1', false);
});

test('Duplicate reports the widget id', async () => {
	const h = handlers();
	render(Inspector, { widget: widget(), scene: null, themes, moveTargets, ...h });
	await page.getByRole('button', { name: 'Duplicate' }).click();
	expect(h.onDuplicate).toHaveBeenCalledWith('w1');
});

test('choosing a move-to-scene target reports the move', async () => {
	const h = handlers();
	render(Inspector, { widget: widget(), scene: null, themes, moveTargets, ...h });
	// Ark Select: open the listbox from the combobox trigger, then pick the scene.
	await page.getByRole('combobox', { name: 'Move to scene' }).click();
	await page.getByRole('option', { name: 'BRB' }).click();
	expect(h.onMoveToScene).toHaveBeenCalledWith('w1', 's2');
});

test('with no widget, changing the scene theme reports it', async () => {
	const h = handlers();
	render(Inspector, { widget: null, scene: scene(), themes, moveTargets, ...h });
	// Ark Select: open the listbox from the combobox trigger, then pick an option.
	await page.getByRole('combobox', { name: 'Theme' }).click();
	await page.getByRole('option', { name: 'Cyber' }).click();
	expect(h.onSetSceneTheme).toHaveBeenCalledWith('s1', 'cyber');
});
