<script lang="ts">
	// The editor canvas: a live WYSIWYG render of the active scene's widgets on the
	// 1920x1080 virtual surface, scaled to fit (letterboxed), themed exactly as the
	// overlay (resolveThemeStyle). Direct manipulation: drag to move, handles to
	// resize, 8px snap-to-edge with guides; the gesture previews locally and commits
	// once on pointer-up (one undo step). Presentational: it takes the workspace
	// VIEW plus selection + commit callbacks; the page wires these to the client.
	import type { WidgetView, WorkspaceView } from '$lib/shared/crdt/workspace-view';
	import { WIDGET_REGISTRY } from '$lib/entities/widget';
	import { resolveThemeStyle } from '$lib/entities/theme';
	import { createEventBus } from '$lib/shared/events/event-bus';
	import { clientToVirtual, fitScale, VIRTUAL_H, VIRTUAL_W } from '../model/geometry';
	import { snapRect, type Guide, type Rect } from '../model/snap';
	import { resizeRect, type ResizeHandle } from '../model/resize';
	import SelectionOverlay from './SelectionOverlay.svelte';

	interface Props {
		view: WorkspaceView | null;
		selectedWidgetId: string | null;
		onSelectWidget: (id: string | null) => void;
		onCommitGeometry: (id: string, rect: Rect) => void;
	}
	let { view, selectedWidgetId, onSelectWidget, onCommitGeometry }: Props = $props();

	const bus = createEventBus();

	const scene = $derived(
		view ? (view.scenes.find((s) => s.id === view.activeSceneId) ?? view.scenes[0] ?? null) : null
	);
	const widgets = $derived((scene?.widgets ?? []).slice().sort((a, b) => a.z - b.z));
	const themeStyle = $derived(
		scene ? resolveThemeStyle(scene.themeId, view?.themes ?? [], scene.overridesAccent) : null
	);

	let areaW = $state(0);
	let areaH = $state(0);
	const scale = $derived(fitScale({ w: areaW, h: areaH }));
	let stageEl: HTMLDivElement | undefined = $state();

	// Live drag/resize state. `interaction` is non-reactive bookkeeping; `preview`
	// and `guides` drive the render. A gesture below DRAG_THRESHOLD virtual px is
	// treated as a click (select only), so clicking never commits a no-op move.
	const DRAG_THRESHOLD = 4;
	interface Interaction {
		kind: 'move' | 'resize';
		handle?: ResizeHandle;
		id: string;
		startRect: Rect;
		startVirtual: { x: number; y: number };
		moved: boolean;
	}
	let interaction: Interaction | null = null;
	let preview = $state<{ id: string; rect: Rect } | null>(null);
	let guides = $state<Guide[]>([]);

	const rectOf = (w: WidgetView): Rect => ({ x: w.x, y: w.y, w: w.w, h: w.h });
	const effectiveRect = (w: WidgetView): Rect =>
		preview && preview.id === w.id ? preview.rect : rectOf(w);
	const selectedWidget = $derived(widgets.find((w) => w.id === selectedWidgetId) ?? null);
	const selectedRect = $derived(selectedWidget ? effectiveRect(selectedWidget) : null);

	function toVirtual(event: PointerEvent) {
		const rect = stageEl?.getBoundingClientRect();
		if (!rect) return { x: 0, y: 0 };
		return clientToVirtual(
			{ x: event.clientX, y: event.clientY },
			{ x: rect.left, y: rect.top },
			scale
		);
	}

	function startMove(event: PointerEvent, widget: WidgetView) {
		event.stopPropagation();
		onSelectWidget(widget.id);
		interaction = {
			kind: 'move',
			id: widget.id,
			startRect: rectOf(widget),
			startVirtual: toVirtual(event),
			moved: false
		};
	}

	function startResize(event: PointerEvent, handle: ResizeHandle) {
		if (!selectedWidget) return;
		event.stopPropagation();
		interaction = {
			kind: 'resize',
			handle,
			id: selectedWidget.id,
			startRect: rectOf(selectedWidget),
			startVirtual: toVirtual(event),
			moved: false
		};
	}

	function onPointerMove(event: PointerEvent) {
		if (!interaction) return;
		const v = toVirtual(event);
		const dx = v.x - interaction.startVirtual.x;
		const dy = v.y - interaction.startVirtual.y;
		if (!interaction.moved && Math.hypot(dx, dy) < DRAG_THRESHOLD) return;
		interaction.moved = true;

		if (interaction.kind === 'move') {
			const proposed: Rect = {
				...interaction.startRect,
				x: interaction.startRect.x + dx,
				y: interaction.startRect.y + dy
			};
			if (event.altKey) {
				preview = { id: interaction.id, rect: proposed };
				guides = [];
			} else {
				const neighbors = widgets.filter((w) => w.id !== interaction!.id).map(rectOf);
				const snapped = snapRect(proposed, neighbors);
				preview = { id: interaction.id, rect: { ...proposed, x: snapped.x, y: snapped.y } };
				guides = snapped.guides;
			}
		} else {
			preview = {
				id: interaction.id,
				rect: resizeRect(interaction.startRect, interaction.handle!, dx, dy)
			};
			guides = [];
		}
	}

	function onPointerUp() {
		if (interaction?.moved && preview) onCommitGeometry(interaction.id, preview.rect);
		interaction = null;
		preview = null;
		guides = [];
	}

	function keySelect(event: KeyboardEvent, id: string) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			onSelectWidget(id);
		}
	}
</script>

<svelte:window onpointermove={onPointerMove} onpointerup={onPointerUp} />

<div class="canvas-area" bind:clientWidth={areaW} bind:clientHeight={areaH}>
	{#if scene && themeStyle}
		<div
			class="canvas-stage"
			bind:this={stageEl}
			style="width: {VIRTUAL_W}px; height: {VIRTUAL_H}px; transform: scale({scale});"
		>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="themed-canvas"
				data-density={scene.overridesDensity || 'normal'}
				style={themeStyle.inlineVars}
				onpointerdown={() => onSelectWidget(null)}
			>
				{#each widgets as widget (widget.id)}
					{@const Widget = WIDGET_REGISTRY[widget.widgetType]}
					{@const r = effectiveRect(widget)}
					{#if Widget}
						<div
							class="widget-slot"
							class:selected={widget.id === selectedWidgetId}
							class:hidden={!widget.visible}
							style="left: {r.x}px; top: {r.y}px; width: {r.w}px; height: {r.h}px; z-index: {widget.z};"
							role="button"
							tabindex="0"
							aria-label={widget.widgetType}
							onpointerdown={(event) => startMove(event, widget)}
							onkeydown={(event) => keySelect(event, widget.id)}
						>
							<Widget instance={widget} {bus} />
						</div>
					{/if}
				{/each}
			</div>

			<div class="guides">
				{#each guides as guide, i (i)}
					{#if guide.axis === 'x'}
						<div class="guide guide-v" style="left: {guide.pos}px;"></div>
					{:else}
						<div class="guide guide-h" style="top: {guide.pos}px;"></div>
					{/if}
				{/each}
			</div>

			<SelectionOverlay rect={selectedRect} onStartResize={startResize} />
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
		cursor: move;
		touch-action: none;
	}

	.widget-slot.hidden {
		opacity: 0.35;
	}

	.guides {
		position: absolute;
		inset: 0;
		pointer-events: none;
		overflow: hidden;
	}

	.guide-v {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 2px;
		background: var(--editor-selection);
	}

	.guide-h {
		position: absolute;
		left: 0;
		right: 0;
		height: 2px;
		background: var(--editor-selection);
	}

	@media (max-width: 639px) {
		.canvas-area {
			padding: var(--space-2);
		}
	}
</style>
