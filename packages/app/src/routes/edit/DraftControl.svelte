<script lang="ts">
	// The titlebar Live/Draft control. In live mode it offers a "Draft" badge that forks a
	// private draft (edits stop reaching the overlay); in draft mode it shows a highlighted
	// Draft badge plus Publish (merge into live + broadcast) and Discard. Editor chrome.
	import type { EditorMode } from '$lib/shared/crdt/client.svelte';

	interface Props {
		mode: EditorMode;
		onEnterDraft: () => void;
		onPublish: () => void;
		onDiscard: () => void;
	}
	let { mode, onEnterDraft, onPublish, onDiscard }: Props = $props();
</script>

{#if mode === 'draft'}
	<span class="group">
		<span class="badge">Draft</span>
		<button class="pill publish" onclick={onPublish}>Publish</button>
		<button class="pill discard" onclick={onDiscard}>Discard</button>
	</span>
{:else}
	<button
		class="badge muted"
		onclick={onEnterDraft}
		title="Edit privately, then publish to the overlay"
	>
		Draft
	</button>
{/if}

<style>
	.group {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
	}

	.badge {
		flex: none;
		padding: 2px var(--space-2);
		border-radius: var(--radius-circular);
		font-size: var(--text-xs);
		font-weight: 600;
		letter-spacing: 0.02em;
		background: var(--warning);
		color: var(--warning-foreground);
	}

	.badge.muted {
		background: var(--invert);
		color: var(--invert-foreground);
		cursor: pointer;
		transition: opacity var(--dur-fast) var(--ease-out);
	}

	@media (hover: hover) {
		.badge.muted:hover {
			opacity: 0.85;
		}
	}

	.badge.muted:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	.pill {
		height: 22px;
		padding: 0 var(--space-2);
		border-radius: var(--radius-md);
		font-size: var(--text-xs);
		font-weight: 600;
		cursor: pointer;
	}

	.pill.publish {
		background: var(--primary);
		color: var(--primary-foreground);
	}

	.pill.discard {
		color: var(--muted-foreground);
	}

	@media (hover: hover) {
		.pill.discard:hover {
			color: var(--foreground);
			background: var(--muted);
		}
	}

	.pill:not(:disabled):active {
		transform: scale(var(--press-scale));
	}

	.pill:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring);
	}
</style>
