import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ThemeBuilder from './ThemeBuilder.svelte';
import type { SceneView, Theme } from '$lib/shared/crdt/workspace-view';

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

const theme = (id: string, opts: Partial<Theme> = {}): Theme => ({
	id,
	name: opts.name ?? id,
	base: opts.base ?? '',
	protected: opts.protected ?? false,
	tokens: opts.tokens ?? { primary: 'red', accent: 'blue' }
});

const cozy = theme('cozy', { name: 'Cozy', protected: true });

test('editing a token in place reports the new value', async () => {
	const h = handlers();
	render(ThemeBuilder, { scene: scene('cozy'), themes: [cozy], ...h });
	const el = (await page
		.getByRole('textbox', { name: 'primary', exact: true })
		.element()) as HTMLInputElement;
	el.value = 'green';
	el.dispatchEvent(new Event('input', { bubbles: true }));
	expect(h.onSetThemeToken).toHaveBeenCalledWith('cozy', 'primary', 'green');
});

test('duplicate forks the active built-in, deriving the copy from it', async () => {
	const h = handlers();
	render(ThemeBuilder, { scene: scene('cozy'), themes: [cozy], ...h });
	await page.getByRole('button', { name: 'Duplicate' }).click();
	expect(h.onCreateTheme).toHaveBeenCalledWith(
		expect.stringContaining('Cozy'),
		'cozy',
		expect.objectContaining({ primary: 'red' })
	);
	expect(h.onSetSceneTheme).toHaveBeenCalledWith('s1', 'theme-new');
});

test('a protected built-in hides Delete', async () => {
	const h = handlers();
	render(ThemeBuilder, { scene: scene('cozy'), themes: [cozy], ...h });
	await expect.element(page.getByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
});

test('a protected built-in hides the link control (built-ins hold literals only)', async () => {
	const h = handlers();
	render(ThemeBuilder, { scene: scene('cozy'), themes: [cozy], ...h });
	await expect
		.element(page.getByRole('combobox', { name: 'Link primary', exact: true }))
		.not.toBeInTheDocument();
});

test('deleting a custom theme removes it and falls the scene back', async () => {
	const h = handlers();
	render(ThemeBuilder, { scene: scene('theme-1'), themes: [theme('theme-1')], ...h });
	await page.getByRole('button', { name: 'Delete' }).click();
	expect(h.onDeleteTheme).toHaveBeenCalledWith('theme-1');
	expect(h.onSetSceneTheme).toHaveBeenCalledWith('s1', 'cozy');
});

test('linking a token writes a link: reference', async () => {
	const h = handlers();
	render(ThemeBuilder, { scene: scene('theme-1'), themes: [theme('theme-1')], ...h });
	// Ark Select (compact): open the ring token's link picker and choose primary.
	await page.getByRole('combobox', { name: 'Link ring', exact: true }).click();
	await page.getByRole('option', { name: 'primary', exact: true }).click();
	expect(h.onSetThemeToken).toHaveBeenCalledWith('theme-1', 'ring', 'link:primary');
});
