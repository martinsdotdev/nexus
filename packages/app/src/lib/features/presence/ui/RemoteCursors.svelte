<script lang="ts">
	// Live cursors of other collaborators, drawn in the canvas's virtual 1920x1080 space
	// (a sibling of the themed canvas). Each cursor sits at its virtual point; its graphic
	// is counter-scaled by 1/scale so the arrow and name tag stay screen-sized at any zoom,
	// and the position tweens (90ms) so a peer's cursor glides between the throttled updates
	// it sends. Presentational and inert (pointer-events: none).
	import type { PeerPresence } from '$lib/shared/crdt/presence';
	import { peerColor } from '$lib/shared/lib/peer-color';

	/** How a peer's name rides with their cursor. */
	type TagStyle = 'solid' | 'minimal' | 'none';

	interface Props {
		peers: PeerPresence[];
		/** The canvas fit-scale, so cursors counter-scale to stay screen-sized. */
		scale: number;
		/** Name-tag treatment (defaults to the filled "solid" tag). */
		tagStyle?: TagStyle;
		/** Dim the cursors (e.g. while offline). */
		faded?: boolean;
	}
	let { peers, scale, tagStyle = 'solid', faded = false }: Props = $props();

	const cursors = $derived(peers.filter((peer) => peer.cursor));
	const inverse = $derived(scale > 0 ? 1 / scale : 1);
</script>

<div class="remote-cursors" class:faded aria-hidden="true" data-testid="remote-cursors">
	{#each cursors as peer (peer.user.id)}
		<div class="cursor" style="left: {peer.cursor!.x}px; top: {peer.cursor!.y}px;">
			<div class="graphic" style="transform: scale({inverse}); --peer: {peerColor(peer.user.id)};">
				<svg class="arrow" viewBox="0 0 24 24" aria-hidden="true">
					<path
						d="M5 3.5 Q5 2.6 5.8 3.1 L18 13.4 Q18.7 13.9 17.9 14.2 L12.9 14.4 L15.5 20.2 Q15.8 20.9 15 21.2 L13.6 21.8 Q12.9 22 12.6 21.3 L10.1 15.6 L6.3 19 Q5 19.9 5 18.4 Z"
						fill="var(--peer)"
						stroke="oklch(99% 0 0)"
						stroke-width="1.4"
						stroke-linejoin="round"
						stroke-linecap="round"
					/>
				</svg>
				{#if tagStyle !== 'none'}
					<span class="tag {tagStyle}">{peer.user.name}</span>
				{/if}
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

	.remote-cursors.faded {
		opacity: 0.5;
	}

	.cursor {
		position: absolute;
		/* Glide between the throttled position updates a peer sends. */
		transition:
			left 90ms linear,
			top 90ms linear;
	}

	.graphic {
		position: absolute;
		top: 0;
		left: 0;
		/* Explicit size is load-bearing: the graphic is an absolute box inside a zero-width
		   `.cursor`, and an SVG's min-content inline size is 0, so without a definite width
		   its shrink-to-fit collapses to 0 and the whole cursor renders invisible. */
		width: 24px;
		height: 24px;
		transform-origin: top left;
	}

	.arrow {
		display: block;
		width: 24px;
		height: 24px;
		/* The round-joined stroke rides half outside the path; don't let the svg clip it. */
		overflow: visible;
		filter: drop-shadow(0 1px 2px oklch(0% 0 0 / 0.5));
	}

	.tag {
		position: absolute;
		left: 14px;
		top: 16px;
		white-space: nowrap;
		padding: 2px 7px;
		border-radius: var(--radius-md);
		/* A sharp inner corner points the tag back at the cursor tip. */
		border-top-left-radius: 2px;
		font-size: 11px;
		font-weight: 600;
		line-height: 1.4;
	}

	.tag.solid {
		background: var(--peer);
		color: oklch(18% 0.01 264);
		box-shadow: 0 1px 3px oklch(0% 0 0 / 0.45);
	}

	.tag.minimal {
		color: var(--peer);
		padding: 2px 0 2px 2px;
		text-shadow: 0 1px 2px oklch(0% 0 0 / 0.6);
	}
</style>
