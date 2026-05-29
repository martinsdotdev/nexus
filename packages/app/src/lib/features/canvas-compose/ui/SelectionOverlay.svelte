<script lang="ts">
	// Draws the selection chrome for the currently selected widget, positioned in
	// virtual (1920x1080) coordinates so it shares the scaled stage's space. A
	// sibling of the themed canvas (not a child), so it uses editor-chrome tokens,
	// not the overlay theme. Phase 4 adds resize handles here; for now, an outline.
	import type { WidgetView } from '$lib/shared/crdt/workspace-view';

	interface Props {
		widget: WidgetView | null;
	}
	let { widget }: Props = $props();
</script>

{#if widget}
	<div
		class="selection"
		data-testid="selection"
		style="left: {widget.x}px; top: {widget.y}px; width: {widget.w}px; height: {widget.h}px;"
	></div>
{/if}

<style>
	.selection {
		position: absolute;
		pointer-events: none;
		outline: var(--stroke-thicker) solid var(--editor-selection);
		outline-offset: 2px;
		border-radius: 2px;
	}
</style>
