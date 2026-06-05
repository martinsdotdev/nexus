<script lang="ts">
	// Live cursors of other collaborators, drawn in the canvas's virtual 1920x1080 space
	// (a sibling of the themed canvas, like SelectionOverlay). Each cursor is positioned
	// at its virtual point but counter-scaled by 1/scale so the arrow and name label stay
	// screen-sized at any canvas zoom. Presentational and inert (pointer-events: none).
	import type { PeerPresence } from '$lib/shared/crdt/presence';
	import { peerColor } from '../lib/peer-color';

	interface Props {
		peers: PeerPresence[];
		/** The canvas fit-scale, so cursors counter-scale to stay screen-sized. */
		scale: number;
	}
	let { peers, scale }: Props = $props();

	const cursors = $derived(peers.filter((peer) => peer.cursor));
	const inverse = $derived(scale > 0 ? 1 / scale : 1);
</script>

<div class="remote-cursors" aria-hidden="true" data-testid="remote-cursors">
	{#each cursors as peer (peer.user.id)}
		<div class="cursor" style="left: {peer.cursor!.x}px; top: {peer.cursor!.y}px;">
			<div class="graphic" style="transform: scale({inverse}); --peer: {peerColor(peer.user.id)};">
				<svg class="arrow" width="20" height="20" viewBox="0 0 20 20" aria-hidden="true">
					<path
						d="M3 2 L3 17 L7 13 L10 19 L13 18 L10 12 L16 12 Z"
						fill="var(--peer)"
						stroke="white"
						stroke-width="1.2"
						stroke-linejoin="round"
					/>
				</svg>
				<span class="label">{peer.user.name}</span>
			</div>
		</div>
	{/each}
</div>

<style>
	.remote-cursors {
		position: absolute;
		inset: 0;
		overflow: hidden;
		pointer-events: none;
	}

	.cursor {
		position: absolute;
	}

	.graphic {
		position: absolute;
		top: 0;
		left: 0;
		transform-origin: top left;
	}

	.arrow {
		display: block;
		filter: drop-shadow(0 1px 1px oklch(0% 0 0 / 0.35));
	}

	.label {
		position: absolute;
		left: 16px;
		top: 12px;
		white-space: nowrap;
		padding: 1px 6px;
		border-radius: var(--radius-sm);
		background: var(--peer);
		color: oklch(99% 0 0);
		font-size: var(--text-xs);
		font-weight: 600;
		line-height: 1.4;
	}
</style>
