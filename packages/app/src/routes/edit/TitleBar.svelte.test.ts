import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import TitleBar from './TitleBar.svelte';

// The drawer/undo/redo props the titlebar needs to render; the palette button is
// the unit under test, so the rest are inert stand-ins.
const baseProps = {
	leftDrawerOpen: false,
	rightDrawerOpen: false,
	onToggleLeftDock: () => {},
	onToggleRightDock: () => {},
	canUndo: false,
	canRedo: false,
	onUndo: () => {},
	onRedo: () => {}
};

test('clicking the centered command bar opens the palette', async () => {
	const onOpenPalette = vi.fn();
	render(TitleBar, { ...baseProps, onOpenPalette });

	await page.getByRole('button', { name: 'Search commands' }).click();
	expect(onOpenPalette).toHaveBeenCalled();
});
