<script lang="ts">
	// Peer-colored outlines around widgets other collaborators have selected, drawn in the
	// canvas's virtual space (a sibling of the themed canvas). The border, corner radius, and
	// name tag are counter-scaled by 1/scale so they stay screen-sized at any zoom (like the
	// cursors), while the box itself rides the scaled virtual coordinates. Presentational and
	// inert; uses committed widget rects (what remote peers see), not the local drag preview.
	import type { PeerPresence } from '$lib/shared/crdt/presence';
	import type { WidgetView } from '$lib/shared/crdt/workspace-view';
	import { peerColor } from '$lib/shared/lib/peer-color';

	interface Props {
		peers: PeerPresence[];
		widgets: WidgetView[];
		/** Canvas fit-scale, so the border + tag counter-scale to stay screen-sized. */
		scale?: number;
	}
	let { peers, widgets, scale = 1 }: Props = $props();

	const inverse = $derived(scale > 0 ? 1 / scale : 1);

	// One outline per (peer, selected widget that exists on this scene).
	const outlines = $derived(
		peers.flatMap((peer) =>
			peer.selection
				.map((id) => widgets.find((widget) => widget.id === id))
				.filter((widget): widget is WidgetView => Boolean(widget))
				.map((widget) => ({
					key: `${peer.user.id}:${widget.id}`,
					name: peer.user.name,
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
				.w}px; height: {outline.rect.h}px; --peer: {outline.color}; --inv: {inverse};"
		>
			<span class="tag">{outline.name}</span>
		</div>
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
		box-sizing: border-box;
		/* Counter-scale the border, radius, and ring so they read at a constant screen size
		   whatever the canvas zoom (the box itself is in scaled virtual space). */
		border: calc(2px * var(--inv)) solid var(--peer);
		border-radius: calc(4px * var(--inv));
		box-shadow: 0 0 0 calc(1px * var(--inv)) oklch(0% 0 0 / 0.25);
	}

	.tag {
		position: absolute;
		bottom: 100%;
		left: 0;
		transform-origin: bottom left;
		transform: scale(var(--inv));
		display: inline-flex;
		align-items: center;
		height: 18px;
		padding: 0 6px;
		border-radius: var(--radius-sm);
		border-bottom-left-radius: 0;
		background: var(--peer);
		color: oklch(18% 0.01 264);
		font-size: 11px;
		font-weight: 600;
		white-space: nowrap;
	}
</style>
