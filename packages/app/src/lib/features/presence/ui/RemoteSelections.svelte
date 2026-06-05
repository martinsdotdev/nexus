<script lang="ts">
	// Peer-colored outlines around the widgets other collaborators have selected, drawn in
	// the canvas's virtual space (a sibling of the themed canvas). The color matches that
	// peer's cursor and roster chip. Presentational and inert. Uses committed widget rects
	// (what remote peers see), not the local drag preview.
	import type { PeerPresence } from '$lib/shared/crdt/presence';
	import type { WidgetView } from '$lib/shared/crdt/workspace-view';
	import { peerColor } from '../lib/peer-color';

	interface Props {
		peers: PeerPresence[];
		widgets: WidgetView[];
	}
	let { peers, widgets }: Props = $props();

	// One outline per (peer, selected widget that exists on this scene).
	const outlines = $derived(
		peers.flatMap((peer) =>
			peer.selection
				.map((id) => widgets.find((widget) => widget.id === id))
				.filter((widget): widget is WidgetView => Boolean(widget))
				.map((widget) => ({
					key: `${peer.user.id}:${widget.id}`,
					color: peerColor(peer.user.id),
					rect: { x: widget.x, y: widget.y, w: widget.w, h: widget.h }
				}))
		)
	);
</script>

<div class="remote-selections" aria-hidden="true" data-testid="remote-selections">
	{#each outlines as outline (outline.key)}
		<div
			class="outline"
			style="left: {outline.rect.x}px; top: {outline.rect.y}px; width: {outline.rect
				.w}px; height: {outline.rect.h}px; border-color: {outline.color};"
		></div>
	{/each}
</div>

<style>
	.remote-selections {
		position: absolute;
		inset: 0;
		overflow: hidden;
		pointer-events: none;
	}

	.outline {
		position: absolute;
		border: var(--stroke-thicker) solid;
		border-radius: var(--radius-sm);
		box-sizing: border-box;
	}
</style>
