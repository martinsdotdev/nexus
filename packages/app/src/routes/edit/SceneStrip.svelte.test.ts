import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import SceneStrip from './SceneStrip.svelte';

function base(overrides: Record<string, unknown> = {}) {
	return {
		scenes: [
			{ id: 's1', label: 'Live' },
			{ id: 's2', label: 'BRB' }
		],
		activeSceneId: 's1',
		onSelect: vi.fn(),
		onAdd: vi.fn(),
		onRename: vi.fn(),
		onDuplicate: vi.fn(),
		onDelete: vi.fn(),
		onReorder: vi.fn(),
		...overrides
	};
}

test('renders each scene label', async () => {
	render(SceneStrip, base());
	await expect.element(page.getByText('Live')).toBeInTheDocument();
	await expect.element(page.getByText('BRB')).toBeInTheDocument();
});

test('selecting a scene reports it', async () => {
	const props = base();
	render(SceneStrip, props);
	await page.getByRole('button', { name: 'BRB', exact: true }).click();
	expect(props.onSelect).toHaveBeenCalledWith('s2');
});

test('the add-scene button reports an add', async () => {
	const props = base();
	render(SceneStrip, props);
	await page.getByRole('button', { name: 'Add scene' }).click();
	expect(props.onAdd).toHaveBeenCalled();
});

test('duplicating a scene from its menu reports it', async () => {
	const props = base();
	render(SceneStrip, props);
	await page.getByRole('button', { name: 'Actions for Live' }).click();
	await page.getByRole('menuitem', { name: 'Duplicate' }).click();
	expect(props.onDuplicate).toHaveBeenCalledWith('s1');
});

test('deleting a scene from its menu reports it', async () => {
	const props = base();
	render(SceneStrip, props);
	await page.getByRole('button', { name: 'Actions for BRB' }).click();
	await page.getByRole('menuitem', { name: 'Delete' }).click();
	expect(props.onDelete).toHaveBeenCalledWith('s2');
});

test('delete is not offered when only one scene remains', async () => {
	const props = base({ scenes: [{ id: 's1', label: 'Live' }] });
	render(SceneStrip, props);
	await page.getByRole('button', { name: 'Actions for Live' }).click();
	await expect.element(page.getByRole('menuitem', { name: 'Delete' })).not.toBeInTheDocument();
});

test('moving a scene right reports the new index', async () => {
	const props = base();
	render(SceneStrip, props);
	await page.getByRole('button', { name: 'Actions for Live' }).click();
	await page.getByRole('menuitem', { name: 'Move right' }).click();
	expect(props.onReorder).toHaveBeenCalledWith('s1', 1);
});

test('the rename menu action opens an inline editor that commits on blur', async () => {
	const props = base();
	render(SceneStrip, props);
	await page.getByRole('button', { name: 'Actions for Live' }).click();
	await page.getByRole('menuitem', { name: 'Rename' }).click();
	await page.getByRole('textbox', { name: 'Scene name' }).fill('Encore');
	// Blur the field by selecting the other scene; the rename commits first.
	await page.getByRole('button', { name: 'BRB', exact: true }).click();
	expect(props.onRename).toHaveBeenCalledWith('s1', 'Encore');
});
