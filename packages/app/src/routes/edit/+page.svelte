<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { StudioPanel, CommandPalette, Drawer, type CommandItem } from '$lib/shared/ui';
	import { createShellState } from './shell-state.svelte';
	import TitleBar from './TitleBar.svelte';
	import ToolRail from './ToolRail.svelte';
	import SceneStrip from './SceneStrip.svelte';
	import { createWorkspaceClient, type WorkspaceClient } from '$lib/shared/crdt/client.svelte';
	import EditorCanvas from '$lib/features/canvas-compose/ui/EditorCanvas.svelte';
	import Inspector from '$lib/features/inspector/ui/Inspector.svelte';
	import ThemeBuilder from '$lib/features/theme-builder/ui/ThemeBuilder.svelte';
	import {
		WIDGET_TYPES,
		WIDGET_LABELS,
		DEFAULT_WIDGET_SIZE,
		DEFAULT_WIDGET_PROPS
	} from '$lib/entities/widget';
	import { VIRTUAL_W, VIRTUAL_H } from '$lib/shared/config/canvas';

	const shell = createShellState();

	// The collaborative workspace lives in a local Loro replica synced to the
	// relay. Created in an $effect (client-only) since it touches WASM + WebSocket;
	// the scene strip reads scenes + active scene from it. See ADR-0005.
	const sceneLabels: Record<string, () => string> = {
		live: m['editor.scene.live'],
		starting_soon: m['editor.scene.starting_soon'],
		brb: m['editor.scene.brb'],
		ending: m['editor.scene.ending']
	};
	let workspace = $state<WorkspaceClient | null>(null);
	$effect(() => {
		const url = `${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/sync`;
		const client = createWorkspaceClient(url);
		workspace = client;
		return () => client.dispose();
	});
	const sceneCards = $derived(
		(workspace?.scenes ?? []).map((scene) => ({
			id: scene.id,
			label: sceneLabels[scene.kind]?.() ?? scene.kind
		}))
	);
	const activeSceneId = $derived(workspace?.activeSceneId ?? '');

	// The active scene + selected widget drive the inspector.
	const activeScene = $derived.by(() => {
		const scenes = workspace?.workspace.scenes ?? [];
		return scenes.find((scene) => scene.id === activeSceneId) ?? scenes[0] ?? null;
	});
	const selectedWidget = $derived(
		activeScene?.widgets.find((widget) => widget.id === shell.selectedWidgetId) ?? null
	);
	const themes = $derived(workspace?.workspace.themes ?? []);

	// The command palette's commands; runCommand dispatches by id.
	const commands: CommandItem[] = [
		{ id: 'add-widget', label: m['editor.command.add_widget']() },
		{ id: 'switch-theme', label: m['editor.command.switch_theme']() },
		{ id: 'toggle-mode', label: m['editor.command.toggle_mode'](), hint: 'Ctrl L' },
		{ id: 'open-layout', label: m['editor.command.open_layout'](), hint: 'Ctrl O' },
		{ id: 'use-in-obs', label: m['editor.command.use_in_obs'](), hint: 'Ctrl E' },
		{ id: 'fire-test-alert', label: m['editor.command.fire_test_alert']() }
	];

	// Add a widget of the given type to the active scene, centered, and select it.
	function addWidget(widgetType: string) {
		const scene = activeScene;
		if (!workspace || !scene) return;
		const size = DEFAULT_WIDGET_SIZE[widgetType] ?? { w: 400, h: 200 };
		const geom = {
			x: Math.round((VIRTUAL_W - size.w) / 2),
			y: Math.round((VIRTUAL_H - size.h) / 2),
			w: size.w,
			h: size.h,
			z: 5
		};
		const id = workspace.createWidget(
			scene.id,
			widgetType,
			geom,
			DEFAULT_WIDGET_PROPS[widgetType] ?? {}
		);
		if (id) shell.selectWidget(id);
	}

	function runCommand(id: string) {
		if (id === 'add-widget') addWidget('stream-info');
		else if (id === 'switch-theme') shell.openThemeEditor();
		// toggle-mode, open-layout, use-in-obs, fire-test-alert: deferred to their own
		// increments (live/draft, layout switching, OBS, real event sources).
	}

	// Editor keyboard shortcuts (client-only; never at module scope). Cmd/Ctrl-K
	// opens the palette; undo/redo + canvas shortcuts apply only when NOT typing in a
	// field, so they never hijack text editing.
	$effect(() => {
		const onKey = (event: KeyboardEvent) => {
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

			const nudges: Record<string, [number, number]> = {
				ArrowLeft: [-1, 0],
				ArrowRight: [1, 0],
				ArrowUp: [0, -1],
				ArrowDown: [0, 1]
			};
			const nudge = nudges[event.key];
			if (selected && nudge) {
				event.preventDefault();
				const widget = activeScene?.widgets.find((w) => w.id === selected);
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
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	// Track the viewport zone (client-only) so a panel toggle knows whether to collapse
	// the docked panel or open its off-canvas drawer. Layout itself is owned by CSS.
	$effect(() => shell.initViewportTracking());
</script>

<svelte:head><title>{m['app.name']()} editor</title></svelte:head>

<!-- Panel bodies are defined once and rendered in both the docked StudioPanel and its
     off-canvas drawer twin. Scoped styles below follow the snippet to either mount. -->
{#snippet widgetsBody()}
	<ul class="widget-list">
		{#each WIDGET_TYPES as type (type)}
			<li>
				<button onclick={() => addWidget(type)}>{WIDGET_LABELS[type]}</button>
			</li>
		{/each}
	</ul>
{/snippet}

{#snippet inspectorBody()}
	{#if shell.themeEditorOpen}
		<ThemeBuilder
			scene={activeScene}
			{themes}
			onCreateTheme={(name, base, tokens) => workspace?.createTheme(name, base, tokens) ?? ''}
			onRenameTheme={(id, name) => workspace?.renameTheme(id, name)}
			onSetThemeToken={(id, token, value) => workspace?.setThemeToken(id, token, value)}
			onDeleteTheme={(id) => workspace?.deleteTheme(id)}
			onExportTheme={(id) => workspace?.exportTheme(id) ?? null}
			onSetSceneTheme={(sceneId, themeId) => workspace?.setSceneTheme(sceneId, themeId)}
			onClose={() => shell.closeThemeEditor()}
		/>
	{:else}
		<Inspector
			widget={selectedWidget}
			scene={activeScene}
			{themes}
			onSetGeometry={(id, geom) => workspace?.setWidgetGeometry(id, geom)}
			onSetProp={(id, key, value) => workspace?.setWidgetProp(id, key, value)}
			onSetVisible={(id, visible) => workspace?.setWidgetVisible(id, visible)}
			onSetSceneTheme={(sceneId, themeId) => workspace?.setSceneTheme(sceneId, themeId)}
			onSetSceneOverride={(sceneId, key, value) => workspace?.setSceneOverride(sceneId, key, value)}
			onCustomizeTheme={() => shell.openThemeEditor()}
		/>
	{/if}
{/snippet}

<div class="shell" inert={shell.leftDrawerOpen || shell.rightDrawerOpen}>
	<TitleBar
		leftDrawerOpen={shell.leftDrawerOpen}
		rightDrawerOpen={shell.rightDrawerOpen}
		onToggleLeftDock={() => shell.toggleLeftDock()}
		onToggleRightDock={() => shell.toggleRightDock()}
		canUndo={workspace?.canUndo ?? false}
		canRedo={workspace?.canRedo ?? false}
		onUndo={() => workspace?.undo()}
		onRedo={() => workspace?.redo()}
	/>
	<ToolRail activeToolId={shell.activeToolId} onSelect={(id) => shell.setActiveTool(id)} />

	<div class="dock dock-left">
		<StudioPanel
			title={m['editor.panel.widgets']()}
			collapsed={shell.leftPanelCollapsed}
			onToggleCollapse={() => shell.toggleLeftPanel()}
			side="left"
		>
			{@render widgetsBody()}
		</StudioPanel>
	</div>

	<EditorCanvas
		view={workspace?.workspace ?? null}
		selectedWidgetId={shell.selectedWidgetId}
		onSelectWidget={(id) => shell.selectWidget(id)}
		onCommitGeometry={(id, rect) => workspace?.setWidgetGeometry(id, rect)}
	/>

	<div class="dock dock-right">
		<StudioPanel
			title={m['editor.panel.inspector']()}
			collapsed={shell.rightPanelCollapsed}
			onToggleCollapse={() => shell.toggleRightPanel()}
			side="right"
		>
			{@render inspectorBody()}
		</StudioPanel>
	</div>

	<SceneStrip scenes={sceneCards} {activeSceneId} onSelect={(id) => workspace?.activate(id)} />
</div>

<!-- Off-canvas drawers live OUTSIDE .shell so the inert binding above never disables
     the open drawer. Each reuses the same body snippet as its docked twin. -->
<Drawer
	side="left"
	open={shell.leftDrawerOpen}
	onClose={() => shell.closeLeftDrawer()}
	title={m['editor.panel.widgets']()}
>
	<StudioPanel title={m['editor.panel.widgets']()} variant="drawer" side="left">
		{@render widgetsBody()}
	</StudioPanel>
</Drawer>

<Drawer
	side="right"
	open={shell.rightDrawerOpen}
	onClose={() => shell.closeRightDrawer()}
	title={m['editor.panel.inspector']()}
>
	<StudioPanel title={m['editor.panel.inspector']()} variant="drawer" side="right">
		{@render inspectorBody()}
	</StudioPanel>
</Drawer>

<CommandPalette
	open={shell.paletteOpen}
	onClose={() => shell.closePalette()}
	placeholder={m['editor.palette.placeholder']()}
	items={commands}
	onSelect={runCommand}
/>

<style>
	.shell {
		display: grid;
		grid-template-areas:
			'titlebar titlebar titlebar titlebar'
			'toolrail dock-left canvas dock-right'
			'scenestrip scenestrip scenestrip scenestrip';
		grid-template-columns: var(--toolrail-width) auto 1fr auto;
		grid-template-rows: var(--titlebar-height) 1fr var(--scenestrip-height);
		height: 100dvh;
		background: var(--background);
		color: var(--foreground);
	}

	.dock {
		display: flex;
		padding: var(--space-2);
	}

	.dock-left {
		grid-area: dock-left;
	}

	.dock-right {
		grid-area: dock-right;
	}

	.widget-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.widget-list button {
		width: 100%;
		text-align: left;
		padding: var(--space-2);
		border-radius: var(--radius-sm);
		background: var(--muted);
		color: var(--foreground);
		font: inherit;
		font-size: var(--text-sm);
		border: var(--stroke-thin) solid transparent;
		cursor: pointer;
		transition: border-color var(--dur-fast) var(--ease-out);
	}

	.widget-list button:hover {
		border-color: var(--border);
	}

	.widget-list button:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	/*
	 * Responsive zone overrides. These MUST come after the base rules above: a media
	 * query adds no specificity, so an override only wins if it is later in source than
	 * the base rule it overrides (the .dock display:none vs .dock display:flex case).
	 */

	/* Zone B (medium, <= 1279px): the right inspector leaves the grid for a drawer. */
	@media (max-width: 1279px) {
		.shell {
			grid-template-areas:
				'titlebar titlebar titlebar'
				'toolrail dock-left canvas'
				'scenestrip scenestrip scenestrip';
			grid-template-columns: var(--toolrail-width) auto 1fr;
		}

		.dock-right {
			display: none;
		}
	}

	/* Zone C (narrow, <= 959px): the left widgets panel also leaves the grid. */
	@media (max-width: 959px) {
		.shell {
			grid-template-areas:
				'titlebar titlebar'
				'toolrail canvas'
				'scenestrip scenestrip';
			grid-template-columns: var(--toolrail-width) 1fr;
		}

		.dock-left {
			display: none;
		}
	}

	/* Zone D (best-effort, <= 639px): tool rail becomes a horizontal bar above the
	   canvas, the canvas goes near-full-bleed, the scene strip thins. */
	@media (max-width: 639px) {
		.shell {
			--scenestrip-height: 76px;
			grid-template-areas:
				'titlebar'
				'toolrail'
				'canvas'
				'scenestrip';
			grid-template-columns: 1fr;
			grid-template-rows: var(--titlebar-height) auto 1fr var(--scenestrip-height);
		}
	}
</style>
