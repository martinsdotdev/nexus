import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ThemeBuilder from './ThemeBuilder.svelte';
import type { CustomTheme, SceneView } from '$lib/shared/crdt/workspace-view';
// Load a built-in's tokens so the duplicate probe reads real values.
import '$lib/shared/styles/themes/cozy.css';

const handlers = () => ({
	onCreateTheme: vi.fn(() => 'theme-new'),
	onRenameTheme: vi.fn(),
	onSetThemeToken: vi.fn(),
	onDeleteTheme: vi.fn(),
	onExportTheme: vi.fn(() => null),
	onSetSceneTheme: vi.fn(),
	onClose: vi.fn()
});

const scene = (themeId: string): SceneView => ({
	id: 's1',
	kind: 'live',
	name: 'live',
	themeId,
	overridesAccent: '',
	overridesDensity: '',
	widgets: []
});

const custom = (): CustomTheme => ({
	id: 'theme-1',
	name: 'My Theme',
	base: 'cozy',
	tokens: { primary: 'red', accent: 'blue' }
});

test('duplicate-to-customize creates a theme from the built-in and assigns it', async () => {
	const h = handlers();
	render(ThemeBuilder, { scene: scene('cozy'), customThemes: [], ...h });
	await page.getByRole('button', { name: 'Duplicate to customize' }).click();
	expect(h.onCreateTheme).toHaveBeenCalledWith(
		expect.stringContaining('cozy'),
		'cozy',
		expect.anything()
	);
	expect(h.onSetSceneTheme).toHaveBeenCalledWith('s1', 'theme-new');
});

test('editing a token reports the new value', async () => {
	const h = handlers();
	render(ThemeBuilder, { scene: scene('theme-1'), customThemes: [custom()], ...h });
	const el = (await page
		.getByRole('textbox', { name: 'primary', exact: true })
		.element()) as HTMLInputElement;
	el.value = 'green';
	el.dispatchEvent(new Event('input', { bubbles: true }));
	expect(h.onSetThemeToken).toHaveBeenCalledWith('theme-1', 'primary', 'green');
});

test('linking a token writes a link: reference', async () => {
	const h = handlers();
	render(ThemeBuilder, { scene: scene('theme-1'), customThemes: [custom()], ...h });
	const select = (await page
		.getByRole('combobox', { name: 'Link ring', exact: true })
		.element()) as HTMLSelectElement;
	select.value = 'primary';
	select.dispatchEvent(new Event('change', { bubbles: true }));
	expect(h.onSetThemeToken).toHaveBeenCalledWith('theme-1', 'ring', 'link:primary');
});
