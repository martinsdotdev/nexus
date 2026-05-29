/*
 * Prototype-level UI state for the editor shell. Shallow and client-only: it
 * holds which tool is active, panel collapse flags, off-canvas drawer flags, the
 * derived viewport mode, command-palette visibility, and the active scene. No
 * persistence, no qubit, no business logic. State resets on reload, which is
 * correct for a visual prototype. The real editor state will live in the Rust
 * core (Decider) reached over qubit; this is chrome.
 *
 * Exposed via getters so the $state stays reactive when read inside components.
 */

export type ViewportMode = 'wide' | 'medium' | 'narrow';

/*
 * Viewport thresholds in px, mirrored from --bp-wide / --bp-medium in tokens.css.
 * CSS @media cannot read var(), so the literals live in both places plus each
 * query condition; keep the three in sync. tokens.css holds the canonical note.
 */
export const BREAKPOINTS = {
	wide: 1280,
	medium: 960
} as const;

export function createShellState() {
	let activeToolId = $state('select');
	let leftPanelCollapsed = $state(false);
	let rightPanelCollapsed = $state(false);
	let leftDrawerOpen = $state(false);
	let rightDrawerOpen = $state(false);
	let viewportMode = $state<ViewportMode>('wide');
	let paletteOpen = $state(false);
	// Per-user selection: which widget(s) are selected on the canvas. Local-only
	// (never synced; collaborators select independently). Single-select for now;
	// the array shape leaves multi-select open without a reshape.
	let selectedWidgetIds = $state<string[]>([]);
	// Whether the right panel shows the theme builder instead of the inspector.
	let themeEditorOpen = $state(false);

	return {
		get activeToolId() {
			return activeToolId;
		},
		setActiveTool(id: string) {
			activeToolId = id;
		},

		get leftPanelCollapsed() {
			return leftPanelCollapsed;
		},
		toggleLeftPanel() {
			leftPanelCollapsed = !leftPanelCollapsed;
		},

		get rightPanelCollapsed() {
			return rightPanelCollapsed;
		},
		toggleRightPanel() {
			rightPanelCollapsed = !rightPanelCollapsed;
		},

		get leftDrawerOpen() {
			return leftDrawerOpen;
		},
		openLeftDrawer() {
			leftDrawerOpen = true;
			rightDrawerOpen = false;
		},
		closeLeftDrawer() {
			leftDrawerOpen = false;
		},

		get rightDrawerOpen() {
			return rightDrawerOpen;
		},
		openRightDrawer() {
			rightDrawerOpen = true;
			leftDrawerOpen = false;
		},
		closeRightDrawer() {
			rightDrawerOpen = false;
		},

		get viewportMode() {
			return viewportMode;
		},

		/*
		 * Panel-toggle intent, resolved by viewport. The left widget panel is docked
		 * in wide and medium (toggle collapses it), off-canvas only in narrow. The
		 * right inspector is docked only in wide; in medium and narrower it is a
		 * drawer. Asymmetric by design (the inspector sheds first). Callers do not
		 * branch on mode; they call these.
		 */
		toggleLeftDock() {
			if (viewportMode !== 'narrow') {
				leftPanelCollapsed = !leftPanelCollapsed;
			} else if (leftDrawerOpen) {
				leftDrawerOpen = false;
			} else {
				leftDrawerOpen = true;
				rightDrawerOpen = false;
			}
		},
		toggleRightDock() {
			if (viewportMode === 'wide') {
				rightPanelCollapsed = !rightPanelCollapsed;
			} else if (rightDrawerOpen) {
				rightDrawerOpen = false;
			} else {
				rightDrawerOpen = true;
				leftDrawerOpen = false;
			}
		},

		/*
		 * Client-only viewport tracking. Call inside an $effect (never module scope),
		 * so window is untouched during prerender. Returns a teardown. Layout itself
		 * is owned by CSS media queries; this drives only drawer orchestration and
		 * ARIA. When a panel re-docks (leaving its drawer zone) its drawer is forced
		 * shut so it cannot be left orphaned.
		 */
		initViewportTracking() {
			const wide = window.matchMedia(`(min-width: ${BREAKPOINTS.wide}px)`);
			const medium = window.matchMedia(`(min-width: ${BREAKPOINTS.medium}px)`);
			const sync = () => {
				viewportMode = wide.matches ? 'wide' : medium.matches ? 'medium' : 'narrow';
				if (viewportMode !== 'narrow') leftDrawerOpen = false;
				if (viewportMode === 'wide') rightDrawerOpen = false;
			};
			sync();
			wide.addEventListener('change', sync);
			medium.addEventListener('change', sync);
			return () => {
				wide.removeEventListener('change', sync);
				medium.removeEventListener('change', sync);
			};
		},

		get paletteOpen() {
			return paletteOpen;
		},
		togglePalette() {
			paletteOpen = !paletteOpen;
		},
		closePalette() {
			paletteOpen = false;
		},

		get selectedWidgetIds() {
			return selectedWidgetIds;
		},
		get selectedWidgetId(): string | null {
			return selectedWidgetIds[0] ?? null;
		},
		isWidgetSelected(id: string) {
			return selectedWidgetIds.includes(id);
		},
		selectWidget(id: string | null) {
			selectedWidgetIds = id ? [id] : [];
		},
		clearSelection() {
			selectedWidgetIds = [];
		},

		get themeEditorOpen() {
			return themeEditorOpen;
		},
		openThemeEditor() {
			themeEditorOpen = true;
		},
		closeThemeEditor() {
			themeEditorOpen = false;
		}
	};
}

export type ShellState = ReturnType<typeof createShellState>;
