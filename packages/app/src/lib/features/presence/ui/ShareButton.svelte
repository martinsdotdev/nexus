<script lang="ts">
	// The Share trigger in the titlebar actions cluster. Reads "Share" with a user-plus icon
	// normally; once the workspace is actively shared (others are in, or a watch link is out)
	// it switches to "Sharing" with a success dot.
	import { UserPlus } from 'lucide-svelte';

	interface Props {
		/** The workspace is actively shared (others present, or a watch link exists). */
		live?: boolean;
		onClick?: () => void;
	}
	let { live = false, onClick }: Props = $props();
</script>

<button class="share" class:is-live={live} type="button" onclick={onClick}>
	{#if live}
		<span class="dot"></span>
	{:else}
		<UserPlus size={15} />
	{/if}
	<span>{live ? 'Sharing' : 'Share'}</span>
</button>

<style>
	.share {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		height: 28px;
		padding: 0 var(--space-3);
		border-radius: var(--radius-md);
		background: var(--secondary);
		color: var(--secondary-foreground);
		font-size: var(--text-sm);
		font-weight: 600;
		border: 1px solid var(--border-subtle);
		cursor: pointer;
		transition:
			background var(--dur-fast) var(--ease-out),
			border-color var(--dur-fast) var(--ease-out);
	}

	.share:hover {
		background: var(--accent);
		border-color: var(--border);
	}

	.share:active {
		transform: scale(var(--press-scale));
	}

	.is-live {
		background: color-mix(in oklch, var(--success) 16%, var(--secondary));
		border-color: color-mix(in oklch, var(--success) 35%, var(--border-subtle));
	}

	.dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--success);
	}
</style>
