<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { StudioPanel, CommandPalette, Drawer, type CommandItem } from '$lib/shared/ui';
	import { createShellState } from './shell-state.svelte';
	import TitleBar from './TitleBar.svelte';
	import ToolRail from './ToolRail.svelte';
	import SceneStrip from './SceneStrip.svelte';
	import { createWorkspaceClient, type WorkspaceClient } from '$lib/shared/crdt/client.svelte';
	import type { PeerIdentity } from '$lib/shared/crdt/presence';
	import SharePopover from '$lib/features/collaboration/ui/SharePopover.svelte';
	import Toaster from '$lib/shared/ui/Toaster.svelte';
	import {
		createCollaboration,
		type Collaboration
	} from '$lib/features/collaboration/model/collaboration.svelte';
	import { fetchSelf } from '$lib/shared/auth/me';
	import { createEditorShortcuts } from './editor-shortcuts';
	import EditorCanvas from '$lib/features/canvas-compose/ui/EditorCanvas.svelte';
	import { createEventBus } from '$lib/shared/events/event-bus';
	import { testEventAt } from '$lib/shared/events/test-events';
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
	let selfIdentity = $state<PeerIdentity | undefined>(undefined);
	let collaboration = $state<Collaboration | null>(null);
	let shareOpen = $state(false);
	$effect(() => {
		// In cloud mode the workspace is chosen by the picker (/edit?workspace=<id>); local
		// mode ignores it and serves its single workspace.
		const workspaceId = new URLSearchParams(location.search).get('workspace');
		const base = `${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/sync`;
		const url = workspaceId ? `${base}?workspace=${encodeURIComponent(workspaceId)}` : base;
		let client: WorkspaceClient | null = null;
		let disposed = false;
		void (async () => {
			const identity = await fetchSelf();
			if (disposed) return;
			selfIdentity = identity;
			client = createWorkspaceClient(url, { identity });
			workspace = client;
			// Cloud mode: a collaboration store owns this workspace's members + sharing.
			if (workspaceId && identity) {
				const collab = createCollaboration(workspaceId, identity.id);
				collaboration = collab;
				void collab.load();
			}
		})();
		return () => {
			disposed = true;
			client?.dispose();
			collaboration = null;
		};
	});
	// A scene shows its name; an un-renamed seeded scene (its name still equals its kind)
	// shows the localized kind label instead, so the four defaults stay translated while
	// renamed and freeform ('custom'-kind) scenes show the name the streamer chose.
	function sceneDisplayName(scene: { kind: string; name: string }): string {
		return scene.name === scene.kind ? (sceneLabels[scene.kind]?.() ?? scene.name) : scene.name;
	}
	const sceneCards = $derived(
		(workspace?.scenes ?? []).map((scene) => ({
			id: scene.id,
			label: sceneDisplayName(scene)
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

	// The titlebar avatar stack: you first (when signed in), then the live remote peers, each
	// annotated as host when they own the workspace. This bridges presence (live cursors) with
	// collaboration (member roles) — the page's job, since a feature may not import another.
	const rosterPeople = $derived.by(() => {
		const members = collaboration?.members ?? [];
		const roleOf = (id: string) => members.find((member) => member.user_id === id)?.role;
		const others = (workspace?.remotePeers ?? []).map((p) => ({
			id: p.user.id,
			name: p.user.name,
			host: roleOf(p.user.id) === 'owner'
		}));
		return selfIdentity
			? [
					{
						id: selfIdentity.id,
						name: selfIdentity.name,
						you: true,
						host: roleOf(selfIdentity.id) === 'owner'
					},
					...others
				]
			: others;
	});

	function openShare() {
		shareOpen = true;
		void collaboration?.load();
	}

	// Broadcast this editor's selection to collaborators whenever it changes (covers
	// canvas clicks, keyboard nudges, and clear-on-delete alike). A no-op until presence
	// is enabled (cloud mode with a signed-in account).
	$effect(() => {
		const id = shell.selectedWidgetId;
		workspace?.setSelection(id ? [id] : []);
	});

	// The editor's local event bus: the canvas widgets subscribe to it, and "Fire test event"
	// publishes synthetic events so the alert / goal / now-playing widgets can be QA'd live
	// without a platform connection. Cycles through the vocabulary on repeated fires.
	const editorBus = createEventBus();
	let testEventIndex = 0;

	// The titlebar layout switcher: switch among active layouts, or create / duplicate /
	// archive one. Switching clears the selection (the old layout's widget left the canvas).
	// `open` is controlled so the open-layout command can pop the menu.
	let layoutMenuOpen = $state(false);
	const layoutNav = $derived.by(() => {
		if (!workspace) return undefined;
		const client = workspace;
		const layouts = client.workspace.layouts.filter((layout) => layout.status !== 'archived');
		const activeId = client.workspace.activeLayoutId;
		const switchTo = (id: string) => {
			client.activateLayout(id);
			shell.clearSelection();
		};
		return {
			layouts: layouts.map((layout) => ({ id: layout.id, name: layout.name })),
			activeLayoutId: activeId,
			onSwitch: switchTo,
			onCreate: () => {
				const id = client.createLayout(`Layout ${client.workspace.layouts.length + 1}`);
				if (id) switchTo(id);
			},
			onDuplicate: () => {
				const name = layouts.find((layout) => layout.id === activeId)?.name ?? 'Layout';
				const id = client.duplicateLayout(activeId, `${name} copy`);
				if (id) switchTo(id);
			},
			onArchive: () => {
				const next = layouts.find((layout) => layout.id !== activeId);
				if (!next) return; // never archive the last layout
				client.archiveLayout(activeId);
				switchTo(next.id);
			},
			open: layoutMenuOpen,
			onOpenChange: (open: boolean) => (layoutMenuOpen = open)
		};
	});

	// The command palette's commands; runCommand dispatches by id.
	const commands: CommandItem[] = [
		{ id: 'add-widget', label: m['editor.command.add_widget']() },
		{ id: 'add-scene', label: m['editor.command.add_scene']() },
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

	// Add a fresh freeform scene to the active layout and switch to it.
	function addScene() {
		const id = workspace?.createScene(`Scene ${(workspace?.scenes.length ?? 0) + 1}`);
		if (id) workspace?.activate(id);
	}

	function runCommand(id: string) {
		if (id === 'add-widget') addWidget('stream-info');
		else if (id === 'add-scene') addScene();
		else if (id === 'switch-theme') shell.openThemeEditor();
		else if (id === 'fire-test-alert') editorBus.emit(testEventAt(testEventIndex++, Date.now()));
		else if (id === 'open-layout') layoutMenuOpen = true;
		else if (id === 'toggle-mode') {
			if (workspace?.mode === 'draft') workspace.publish();
			else workspace?.enterDraft();
		}
		// use-in-obs: deferred to its own increment (OBS).
	}

	// Editor keyboard shortcuts (client-only; never at module scope). The dispatch lives in
	// editor-shortcuts.ts; the handler reads the current workspace + active scene via getters.
	$effect(() => {
		const onKey = createEditorShortcuts(
			shell,
			() => workspace,
			() => activeScene
		);
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
		onOpenPalette={() => shell.togglePalette()}
		people={rosterPeople}
		syncState={workspace?.connection ?? 'syncing'}
		onOpenShare={selfIdentity ? openShare : undefined}
		shareLive={collaboration?.shareLive ?? false}
		{layoutNav}
		mode={workspace?.mode ?? 'live'}
		onEnterDraft={() => workspace?.enterDraft()}
		onPublish={() => workspace?.publish()}
		onDiscard={() => workspace?.discard()}
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
		remotePeers={workspace?.remotePeers ?? []}
		onCursorMove={(x, y) => workspace?.setCursor(x, y)}
		bus={editorBus}
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

	<SceneStrip
		scenes={sceneCards}
		{activeSceneId}
		onSelect={(id) => workspace?.activate(id)}
		onAdd={addScene}
		onRename={(id, name) => workspace?.renameScene(id, name)}
		onDuplicate={(id) => {
			const source = workspace?.scenes.find((scene) => scene.id === id);
			const copyId = workspace?.duplicateScene(
				id,
				source ? `${sceneDisplayName(source)} copy` : 'Scene copy'
			);
			if (copyId) workspace?.activate(copyId);
		}}
		onDelete={(id) => workspace?.deleteScene(id)}
		onReorder={(id, index) => workspace?.reorderScene(id, index)}
	/>
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

{#if shareOpen && selfIdentity && collaboration}
	<SharePopover
		members={collaboration.members}
		selfId={selfIdentity.id}
		myRole={collaboration.myRole}
		watchLink={collaboration.watchLink}
		onClose={() => (shareOpen = false)}
		onInvite={collaboration.invite}
		onSetRole={collaboration.setRole}
		onRemove={collaboration.remove}
		onCreateWatchLink={collaboration.createWatchLink}
	/>
{/if}

<Toaster />

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
		/* Cap the dock to its grid row so a tall panel (e.g. the theme builder's
		   full token rail) scrolls inside the panel body instead of overflowing. */
		min-height: 0;
		min-width: 0;
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
