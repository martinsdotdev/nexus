<script lang="ts">
	// The overlay: a transparent, read-only renderer OBS loads. It mirrors the
	// active scene of the selected layout (?layout=, else the active layout) from
	// a read-only Loro replica, applies the scene's theme via [data-theme], and
	// renders its widgets absolutely-positioned on a 1920x1080 virtual canvas
	// scaled to the viewport. No editor chrome; live-synced from the relay.
	import { page } from '$app/state';
	import { createReadOnlyClient, type ReadOnlyClient } from '$lib/shared/crdt/client.svelte';
	import { createEventBus } from '$lib/shared/events/event-bus';
	import { startMockSource } from '$lib/shared/events/mock-source';
	import { WIDGET_REGISTRY } from '$lib/entities/widget';
	import { resolveThemeStyle } from '$lib/entities/theme';
	import '$lib/shared/styles/themes/cozy.css';
	import '$lib/shared/styles/themes/cyber.css';
	import '$lib/shared/styles/themes/editorial.css';
	import '$lib/shared/styles/themes/sticker.css';

	const VIRTUAL_W = 1920;
	const VIRTUAL_H = 1080;

	// The event bus + scripted source drive the event-driven widgets.
	const bus = createEventBus();

	let client = $state<ReadOnlyClient | null>(null);
	$effect(() => {
		const url = `${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/sync`;
		const replica = createReadOnlyClient(url);
		client = replica;
		const stopMock = startMockSource(bus);
		return () => {
			stopMock();
			replica.dispose();
		};
	});

	// Scale the virtual canvas to fit the window (1.0 in OBS at native size).
	let scale = $state(1);
	$effect(() => {
		const update = () => {
			scale = Math.min(window.innerWidth / VIRTUAL_W, window.innerHeight / VIRTUAL_H);
		};
		update();
		window.addEventListener('resize', update);
		return () => window.removeEventListener('resize', update);
	});

	const requestedLayoutId = $derived(page.url.searchParams.get('layout') ?? '');
	const workspace = $derived(client?.workspace ?? null);
	const layout = $derived(
		workspace
			? (workspace.layouts.find((l) => l.id === requestedLayoutId) ??
					workspace.layouts.find((l) => l.id === workspace.activeLayoutId) ??
					workspace.layouts[0] ??
					null)
			: null
	);
	const scene = $derived(
		layout
			? (layout.scenes.find((s) => s.id === layout.activeSceneId) ?? layout.scenes[0] ?? null)
			: null
	);
	const widgets = $derived(
		(scene?.widgets ?? [])
			.filter((widget) => widget.visible)
			.slice()
			.sort((a, b) => a.z - b.z)
	);
</script>

<svelte:head><title>Nexus overlay</title></svelte:head>

<div class="overlay-viewport">
	{#if scene}
		{@const themeStyle = resolveThemeStyle(scene.themeId, [], scene.overridesAccent)}
		<div
			class="overlay-canvas"
			data-theme={themeStyle.dataTheme}
			data-density={scene.overridesDensity || 'normal'}
			style="width: {VIRTUAL_W}px; height: {VIRTUAL_H}px; transform: scale({scale}); {themeStyle.inlineVars}"
		>
			{#each widgets as widget (widget.id)}
				{@const Widget = WIDGET_REGISTRY[widget.widgetType]}
				{#if Widget}
					<div
						class="widget-slot"
						style="left: {widget.x}px; top: {widget.y}px; width: {widget.w}px; height: {widget.h}px; z-index: {widget.z};"
					>
						<Widget instance={widget} {bus} />
					</div>
				{/if}
			{/each}
		</div>
	{/if}
</div>

<style>
	.overlay-viewport {
		position: fixed;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		background: transparent;
	}
	.overlay-canvas {
		position: relative;
		flex: none;
		transform-origin: center center;
	}
	.widget-slot {
		position: absolute;
	}
</style>
