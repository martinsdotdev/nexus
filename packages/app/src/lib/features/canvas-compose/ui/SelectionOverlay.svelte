<script lang="ts">
	// Selection chrome for the selected widget, positioned in virtual (1920x1080)
	// coordinates so it shares the scaled stage's space. A sibling of the themed
	// canvas (not a child), so it uses editor-chrome tokens, not the overlay theme.
	// The outline is non-interactive; the eight handles capture pointerdown to
	// start a resize (keyboard resize is via the inspector).
	import type { Rect } from '../model/snap';
	import type { ResizeHandle } from '../model/resize';

	interface Props {
		rect: Rect | null;
		onStartResize: (event: PointerEvent, handle: ResizeHandle) => void;
	}
	let { rect, onStartResize }: Props = $props();

	const HANDLES: ResizeHandle[] = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'];
</script>

{#if rect}
	<div
		class="selection"
		data-testid="selection"
		style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px;"
	>
		{#each HANDLES as handle (handle)}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="handle handle-{handle}"
				data-testid="handle-{handle}"
				onpointerdown={(event) => onStartResize(event, handle)}
			></div>
		{/each}
	</div>
{/if}

<style>
	.selection {
		position: absolute;
		pointer-events: none;
		outline: var(--stroke-thicker) solid var(--editor-selection);
		outline-offset: 2px;
		border-radius: 2px;
	}

	.handle {
		position: absolute;
		width: 12px;
		height: 12px;
		background: var(--editor-selection);
		box-shadow: 0 0 0 1.5px var(--editor-selection-foreground);
		border-radius: 2px;
		pointer-events: auto;
		touch-action: none;
	}

	.handle-nw {
		left: 0;
		top: 0;
		transform: translate(-50%, -50%);
		cursor: nwse-resize;
	}
	.handle-n {
		left: 50%;
		top: 0;
		transform: translate(-50%, -50%);
		cursor: ns-resize;
	}
	.handle-ne {
		left: 100%;
		top: 0;
		transform: translate(-50%, -50%);
		cursor: nesw-resize;
	}
	.handle-e {
		left: 100%;
		top: 50%;
		transform: translate(-50%, -50%);
		cursor: ew-resize;
	}
	.handle-se {
		left: 100%;
		top: 100%;
		transform: translate(-50%, -50%);
		cursor: nwse-resize;
	}
	.handle-s {
		left: 50%;
		top: 100%;
		transform: translate(-50%, -50%);
		cursor: ns-resize;
	}
	.handle-sw {
		left: 0;
		top: 100%;
		transform: translate(-50%, -50%);
		cursor: nesw-resize;
	}
	.handle-w {
		left: 0;
		top: 50%;
		transform: translate(-50%, -50%);
		cursor: ew-resize;
	}
</style>
