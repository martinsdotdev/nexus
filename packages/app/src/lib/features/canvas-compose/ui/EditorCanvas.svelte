<script lang="ts">
	// The editor canvas: a live WYSIWYG render of the active scene's widgets on the
	// 1920x1080 virtual surface, scaled to fit (letterboxed), themed exactly as the
	// overlay (resolveThemeStyle). Presentational: it takes the workspace VIEW plus
	// selection callbacks, the page wires these to the Loro client. Widgets get an
	// editor-local event bus and render their seeded props (event-driven widgets
	// stay empty here; sample preview content is a later concern).
	import type { WorkspaceView } from '$lib/shared/crdt/workspace-view';
	import { WIDGET_REGISTRY } from '$lib/entities/widget';
	import { resolveThemeStyle } from '$lib/entities/theme';
	import { createEventBus } from '$lib/shared/events/event-bus';
	// The canvas is a true overlay preview, so it needs the built-in theme
	// stylesheets ([data-theme=...] cascades) the editor route does not otherwise
	// load. They are scoped by [data-theme], so they never touch the editor chrome.
	import '$lib/shared/styles/themes/cozy.css';
	import '$lib/shared/styles/themes/cyber.css';
	import '$lib/shared/styles/themes/editorial.css';
	import '$lib/shared/styles/themes/sticker.css';
	import { fitScale, VIRTUAL_H, VIRTUAL_W } from '../model/geometry';
	import SelectionOverlay from './SelectionOverlay.svelte';

	interface Props {
		view: WorkspaceView | null;
		selectedWidgetId: string | null;
		onSelectWidget: (id: string | null) => void;
	}
	let { view, selectedWidgetId, onSelectWidget }: Props = $props();

	const bus = createEventBus();

	// Edit the active layout's active scene.
	const scene = $derived(
		view ? (view.scenes.find((s) => s.id === view.activeSceneId) ?? view.scenes[0] ?? null) : null
	);
	const widgets = $derived((scene?.widgets ?? []).slice().sort((a, b) => a.z - b.z));
	const themeStyle = $derived(
		scene ? resolveThemeStyle(scene.themeId, [], scene.overridesAccent) : null
	);
	const selectedWidget = $derived(widgets.find((w) => w.id === selectedWidgetId) ?? null);

	// Measure the canvas area to letterbox the virtual surface into it.
	let areaW = $state(0);
	let areaH = $state(0);
	const scale = $derived(fitScale({ w: areaW, h: areaH }));

	function pickWidget(event: PointerEvent, id: string) {
		event.stopPropagation();
		onSelectWidget(id);
	}

	function keySelect(event: KeyboardEvent, id: string) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			onSelectWidget(id);
		}
	}
</script>

<div class="canvas-area" bind:clientWidth={areaW} bind:clientHeight={areaH}>
	{#if scene && themeStyle}
		<div
			class="canvas-stage"
			style="width: {VIRTUAL_W}px; height: {VIRTUAL_H}px; transform: scale({scale});"
		>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="themed-canvas"
				data-theme={themeStyle.dataTheme}
				data-density={scene.overridesDensity || 'normal'}
				style={themeStyle.inlineVars}
				onpointerdown={() => onSelectWidget(null)}
			>
				{#each widgets as widget (widget.id)}
					{@const Widget = WIDGET_REGISTRY[widget.widgetType]}
					{#if Widget}
						<div
							class="widget-slot"
							class:selected={widget.id === selectedWidgetId}
							class:hidden={!widget.visible}
							style="left: {widget.x}px; top: {widget.y}px; width: {widget.w}px; height: {widget.h}px; z-index: {widget.z};"
							role="button"
							tabindex="0"
							aria-label={widget.widgetType}
							onpointerdown={(event) => pickWidget(event, widget.id)}
							onkeydown={(event) => keySelect(event, widget.id)}
						>
							<Widget instance={widget} {bus} />
						</div>
					{/if}
				{/each}
			</div>
			<SelectionOverlay widget={selectedWidget} />
		</div>
	{/if}
</div>

<style>
	.canvas-area {
		grid-area: canvas;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--space-5);
		overflow: hidden;
		min-width: 0;
		min-height: 0;
	}

	.canvas-stage {
		position: relative;
		flex: none;
		transform-origin: center center;
		border-radius: var(--radius-canvas);
		box-shadow: var(--shadow-panel);
	}

	.themed-canvas {
		position: absolute;
		inset: 0;
		overflow: hidden;
		border-radius: var(--radius-canvas);
		background: var(--background);
	}

	.widget-slot {
		position: absolute;
		cursor: pointer;
	}

	.widget-slot.hidden {
		opacity: 0.35;
	}

	/* The selected slot's chrome lives in SelectionOverlay; nothing extra here. */
	.widget-slot.selected {
		cursor: move;
	}

	@media (max-width: 639px) {
		.canvas-area {
			padding: var(--space-2);
		}
	}
</style>
