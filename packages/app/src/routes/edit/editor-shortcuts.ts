// The editor's keyboard shortcuts, extracted from the page as a pure factory. Given the shell
// and getters for the current workspace client + active scene, it returns a keydown handler the
// page mounts on `window` in one $effect. Cmd/Ctrl-K toggles the command palette; undo/redo and
// the canvas shortcuts (delete, escape, arrow-nudge, 1-4 scene switch) apply only when NOT typing
// in a field, so they never hijack text editing. The deps are minimal structural interfaces, so
// the dispatch is testable with tiny fakes (no full client or shell needed).

/** The slice of editor shell state the shortcuts touch. */
export interface ShortcutShell {
	togglePalette(): void;
	clearSelection(): void;
	readonly selectedWidgetId: string | null;
}

/** The slice of the workspace client the shortcuts drive. */
export interface ShortcutWorkspace {
	readonly scenes: ReadonlyArray<{ id: string }>;
	undo(): void;
	redo(): void;
	deleteWidget(id: string): void;
	setWidgetGeometry(id: string, geom: { x: number; y: number }): void;
	activate(sceneId: string): void;
}

/** The active scene, for resolving a nudged widget's current position. */
export interface ShortcutScene {
	readonly widgets: ReadonlyArray<{ id: string; x: number; y: number }>;
}

const NUDGES: Record<string, [number, number]> = {
	ArrowLeft: [-1, 0],
	ArrowRight: [1, 0],
	ArrowUp: [0, -1],
	ArrowDown: [0, 1]
};

export function createEditorShortcuts(
	shell: ShortcutShell,
	getWorkspace: () => ShortcutWorkspace | null,
	getActiveScene: () => ShortcutScene | null
): (event: KeyboardEvent) => void {
	return (event) => {
		const mod = event.metaKey || event.ctrlKey;
		if (mod && event.key.toLowerCase() === 'k') {
			event.preventDefault();
			shell.togglePalette();
			return;
		}

		const target = event.target as HTMLElement | null;
		const typing =
			!!target &&
			(target.tagName === 'INPUT' ||
				target.tagName === 'TEXTAREA' ||
				target.tagName === 'SELECT' ||
				target.isContentEditable);
		const workspace = getWorkspace();
		if (typing || !workspace) return;

		if (mod && event.key.toLowerCase() === 'z') {
			event.preventDefault();
			if (event.shiftKey) workspace.redo();
			else workspace.undo();
			return;
		}
		if (mod && event.key.toLowerCase() === 'y') {
			event.preventDefault();
			workspace.redo();
			return;
		}

		const selected = shell.selectedWidgetId;
		if ((event.key === 'Delete' || event.key === 'Backspace') && selected) {
			event.preventDefault();
			workspace.deleteWidget(selected);
			shell.clearSelection();
			return;
		}
		if (event.key === 'Escape') {
			shell.clearSelection();
			return;
		}

		const nudge = NUDGES[event.key];
		if (selected && nudge) {
			event.preventDefault();
			const widget = getActiveScene()?.widgets.find((w) => w.id === selected);
			if (widget) {
				const step = event.shiftKey ? 10 : 1;
				workspace.setWidgetGeometry(selected, {
					x: widget.x + nudge[0] * step,
					y: widget.y + nudge[1] * step
				});
			}
			return;
		}

		if (['1', '2', '3', '4'].includes(event.key)) {
			const scene = workspace.scenes[Number(event.key) - 1];
			if (scene) {
				event.preventDefault();
				workspace.activate(scene.id);
			}
		}
	};
}
