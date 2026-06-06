<script lang="ts">
	// The sync-status pill in the titlebar: a quiet capsule that reads the live link to the
	// relay. Connected states show a colored dot (green Synced, pulsing amber Reconnecting);
	// the transient/severed states show an icon (Syncing, Working offline). Becomes a button
	// when given an open handler (it doubles as the Share trigger), a span otherwise.
	import { RefreshCw, WifiOff } from 'lucide-svelte';
	import type { ConnectionState } from '$lib/shared/crdt/sync-bridge';

	interface Props {
		state: ConnectionState;
		/** Open Share (makes the pill an interactive button). */
		onClick?: () => void;
	}
	let { state, onClick }: Props = $props();

	const LABELS: Record<ConnectionState, string> = {
		synced: 'Synced',
		syncing: 'Syncing…',
		reconnecting: 'Reconnecting…',
		offline: 'Working offline'
	};
	const showIcon = $derived(state === 'syncing' || state === 'offline');
</script>

{#snippet body()}
	{#if showIcon}
		{#if state === 'syncing'}
			<RefreshCw size={13} />
		{:else}
			<WifiOff size={13} />
		{/if}
	{:else}
		<span class="dot"></span>
	{/if}
	<span>{LABELS[state]}</span>
{/snippet}

{#if onClick}
	<button
		type="button"
		class="sync"
		class:is-synced={state === 'synced'}
		class:is-syncing={state === 'syncing'}
		class:is-reconnecting={state === 'reconnecting'}
		class:is-offline={state === 'offline'}
		data-testid="sync-pill"
		title={LABELS[state]}
		onclick={onClick}
	>
		{@render body()}
	</button>
{:else}
	<span
		class="sync"
		class:is-synced={state === 'synced'}
		class:is-syncing={state === 'syncing'}
		class:is-reconnecting={state === 'reconnecting'}
		class:is-offline={state === 'offline'}
		data-testid="sync-pill"
		title={LABELS[state]}
	>
		{@render body()}
	</span>
{/if}

<style>
	.sync {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		height: 24px;
		padding: 0 var(--space-2);
		border-radius: var(--radius-circular);
		background: var(--muted);
		border: 1px solid var(--border-subtle);
		color: var(--muted-foreground);
		font-size: var(--text-xs);
		font-weight: 600;
		white-space: nowrap;
		transition:
			background var(--dur-fast) var(--ease-out),
			color var(--dur-fast) var(--ease-out),
			border-color var(--dur-fast) var(--ease-out);
	}

	button.sync {
		cursor: pointer;
	}

	.sync:hover {
		border-color: var(--border);
		color: var(--foreground);
	}

	.dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: currentColor;
		flex: none;
	}

	.is-synced {
		color: var(--success);
	}
	.is-syncing {
		color: var(--info);
	}
	.is-reconnecting {
		color: var(--warning);
	}
	.is-reconnecting .dot {
		animation: pulse 0.7s var(--ease-out) infinite;
	}
	.is-offline {
		color: var(--muted-foreground);
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.3;
		}
	}
</style>
