import { expect, test, vi } from 'vitest';
import {
	createEditorShortcuts,
	type ShortcutScene,
	type ShortcutShell,
	type ShortcutWorkspace
} from './editor-shortcuts';

function setup(selectedWidgetId: string | null = null, mode: 'live' | 'draft' = 'live') {
	const shell: ShortcutShell = {
		togglePalette: vi.fn(),
		clearSelection: vi.fn(),
		selectWidget: vi.fn(),
		selectedWidgetId
	};
	const workspace: ShortcutWorkspace = {
		scenes: [{ id: 's1' }, { id: 's2' }],
		undo: vi.fn(),
		redo: vi.fn(),
		deleteWidget: vi.fn(),
		duplicateWidget: vi.fn(),
		setWidgetGeometry: vi.fn(),
		activate: vi.fn(),
		mode,
		enterDraft: vi.fn(),
		publish: vi.fn()
	};
	const scene: ShortcutScene = { widgets: [{ id: 'w1', x: 100, y: 50 }] };
	const handle = createEditorShortcuts(
		shell,
		() => workspace,
		() => scene
	);
	return { shell, workspace, handle };
}

function key(init: Partial<KeyboardEvent> & { key: string }): KeyboardEvent {
	return {
		preventDefault: () => {},
		target: null,
		metaKey: false,
		ctrlKey: false,
		shiftKey: false,
		...init
	} as unknown as KeyboardEvent;
}

test('Ctrl+K toggles the command palette', () => {
	const { shell, handle } = setup();
	handle(key({ key: 'k', ctrlKey: true }));
	expect(shell.togglePalette).toHaveBeenCalled();
});

test('Ctrl+Z undoes and Ctrl+Shift+Z redoes', () => {
	const { workspace, handle } = setup();
	handle(key({ key: 'z', ctrlKey: true }));
	expect(workspace.undo).toHaveBeenCalled();
	handle(key({ key: 'z', ctrlKey: true, shiftKey: true }));
	expect(workspace.redo).toHaveBeenCalled();
});

test('Ctrl+L enters draft from live and publishes from draft', () => {
	const live = setup();
	live.handle(key({ key: 'l', ctrlKey: true }));
	expect(live.workspace.enterDraft).toHaveBeenCalled();

	const draft = setup(null, 'draft');
	draft.handle(key({ key: 'l', ctrlKey: true }));
	expect(draft.workspace.publish).toHaveBeenCalled();
});

test('Delete removes the selected widget and clears selection', () => {
	const { workspace, shell, handle } = setup('w1');
	handle(key({ key: 'Delete' }));
	expect(workspace.deleteWidget).toHaveBeenCalledWith('w1');
	expect(shell.clearSelection).toHaveBeenCalled();
});

test('Ctrl+D duplicates the selected widget and selects the copy', () => {
	const { workspace, shell, handle } = setup('w1');
	vi.mocked(workspace.duplicateWidget).mockReturnValue('w2');
	handle(key({ key: 'd', ctrlKey: true }));
	expect(workspace.duplicateWidget).toHaveBeenCalledWith('w1');
	expect(shell.selectWidget).toHaveBeenCalledWith('w2');
});

test('an arrow nudges the selected widget from its current position', () => {
	const { workspace, handle } = setup('w1');
	handle(key({ key: 'ArrowRight' }));
	expect(workspace.setWidgetGeometry).toHaveBeenCalledWith('w1', { x: 101, y: 50 });
});

test('a number key activates the matching scene', () => {
	const { workspace, handle } = setup();
	handle(key({ key: '2' }));
	expect(workspace.activate).toHaveBeenCalledWith('s2');
});

test('does nothing destructive while typing in a field', () => {
	const { workspace, handle } = setup('w1');
	const input = { tagName: 'INPUT', isContentEditable: false } as HTMLElement;
	handle(key({ key: 'Delete', target: input }));
	expect(workspace.deleteWidget).not.toHaveBeenCalled();
});
