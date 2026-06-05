<script lang="ts">
	// A who's-online stack of the other collaborators in this workspace, shown in the
	// titlebar. One overlapping avatar chip per peer (de-duped upstream by account), each
	// in that peer's color so it matches their cursor and selection outline; beyond `max`
	// chips it collapses to a "+N" badge. Hidden when nobody else is here.
	import type { PeerPresence } from '$lib/shared/crdt/presence';
	import { peerColor } from '../lib/peer-color';

	interface Props {
		peers: PeerPresence[];
		/** How many avatar chips to show before collapsing the rest into "+N". */
		max?: number;
	}
	let { peers, max = 4 }: Props = $props();

	const shown = $derived(peers.slice(0, max));
	const overflow = $derived(Math.max(0, peers.length - max));
	const initial = (name: string) => name.trim().charAt(0).toUpperCase() || '?';
</script>

{#if peers.length > 0}
	<div class="roster" data-testid="roster" aria-label="Collaborators online">
		{#each shown as peer (peer.user.id)}
			<span class="chip" style="background: {peerColor(peer.user.id)};" title={peer.user.name}>
				{initial(peer.user.name)}
			</span>
		{/each}
		{#if overflow > 0}
			<span class="chip more" title="{overflow} more">+{overflow}</span>
		{/if}
	</div>
{/if}

<style>
	.roster {
		display: inline-flex;
		align-items: center;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-circular);
		color: oklch(99% 0 0);
		font-size: var(--text-xs);
		font-weight: 700;
		/* A titlebar-colored ring separates overlapping chips. */
		border: var(--stroke-thicker) solid var(--titlebar);
		margin-left: -6px;
	}

	.chip:first-child {
		margin-left: 0;
	}

	.more {
		background: var(--muted);
		color: var(--muted-foreground);
	}
</style>
