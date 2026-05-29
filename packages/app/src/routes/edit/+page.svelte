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

	// Static command list for the prototype. Selecting one just closes the palette.
	const commands: CommandItem[] = [
		{ id: 'add-widget', label: m['editor.command.add_widget']() },
		{ id: 'switch-theme', label: m['editor.command.switch_theme']() },
		{ id: 'toggle-mode', label: m['editor.command.toggle_mode'](), hint: 'Ctrl L' },
		{ id: 'open-layout', label: m['editor.command.open_layout'](), hint: 'Ctrl O' },
		{ id: 'use-in-obs', label: m['editor.command.use_in_obs'](), hint: 'Ctrl E' },
		{ id: 'fire-test-alert', label: m['editor.command.fire_test_alert']() }
	];

	// Cmd/Ctrl-K toggles the command palette. Attached on mount (never at module
	// scope, which would run during prerender where window is undefined).
	$effect(() => {
		const onKey = (event: KeyboardEvent) => {
			if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
				event.preventDefault();
				shell.togglePalette();
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
	<ul class="placeholder-list">
		<li>Webcam frame</li>
		<li>Chat box</li>
		<li>Alerts</li>
		<li>Goal bar</li>
		<li>Now playing</li>
	</ul>
{/snippet}

{#snippet inspectorBody()}
	<Inspector
		widget={selectedWidget}
		scene={activeScene}
		onSetGeometry={(id, geom) => workspace?.setWidgetGeometry(id, geom)}
		onSetProp={(id, key, value) => workspace?.setWidgetProp(id, key, value)}
		onSetVisible={(id, visible) => workspace?.setWidgetVisible(id, visible)}
		onSetSceneTheme={(sceneId, themeId) => workspace?.setSceneTheme(sceneId, themeId)}
		onSetSceneOverride={(sceneId, key, value) => workspace?.setSceneOverride(sceneId, key, value)}
	/>
{/snippet}

<div class="shell" inert={shell.leftDrawerOpen || shell.rightDrawerOpen}>
	<TitleBar
		leftDrawerOpen={shell.leftDrawerOpen}
		rightDrawerOpen={shell.rightDrawerOpen}
		onToggleLeftDock={() => shell.toggleLeftDock()}
		onToggleRightDock={() => shell.toggleRightDock()}
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

	.placeholder-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		color: var(--muted-foreground);
	}

	.placeholder-list li {
		padding: var(--space-2);
		border-radius: var(--radius-sm);
		background: var(--muted);
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
