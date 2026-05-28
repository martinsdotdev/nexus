<script lang="ts">
	import { Search } from 'lucide-svelte';
	import type { CommandItem } from './types';

	interface Props {
		/** Whether the palette is open. */
		open: boolean;
		/** Called when the palette should close (Escape, backdrop, or selection). */
		onClose: () => void;
		/** Placeholder for the search input. */
		placeholder?: string;
		/** The (static, in this prototype) command list. */
		items: CommandItem[];
	}

	let { open, onClose, placeholder = 'Type a command', items }: Props = $props();

	let query = $state('');
	let highlighted = $state(0);
	let inputEl = $state<HTMLInputElement | null>(null);
	let previousFocus: HTMLElement | null = null;

	const filtered = $derived(
		items.filter((item) => item.label.toLowerCase().includes(query.trim().toLowerCase()))
	);
	const safeIndex = $derived(Math.max(0, Math.min(highlighted, filtered.length - 1)));

	// Focus management: capture focus on open, restore on close.
	$effect(() => {
		if (open) {
			previousFocus = document.activeElement as HTMLElement | null;
			query = '';
			highlighted = 0;
			inputEl?.focus();
		} else if (previousFocus) {
			previousFocus.focus();
			previousFocus = null;
		}
	});

	function onInputKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			onClose();
		} else if (event.key === 'ArrowDown') {
			event.preventDefault();
			if (filtered.length > 0) highlighted = (safeIndex + 1) % filtered.length;
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			if (filtered.length > 0) highlighted = (safeIndex - 1 + filtered.length) % filtered.length;
		} else if (event.key === 'Enter') {
			event.preventDefault();
			// Prototype: selecting a command just closes. Execution is deferred.
			if (filtered.length > 0) onClose();
		}
	}
</script>

{#if open}
	<div class="backdrop">
		<!-- Full-screen close affordance behind the dialog: a real button so it is
		     keyboard-accessible. Escape (handled in the input) is the primary close. -->
		<button class="scrim" aria-label="Close command palette" onclick={onClose}></button>
		<div class="palette" role="dialog" aria-modal="true" aria-label="Command palette">
			<div class="search">
				<Search size={16} />
				<input
					bind:this={inputEl}
					bind:value={query}
					type="text"
					{placeholder}
					autocomplete="off"
					spellcheck="false"
					aria-label="Command search"
					onkeydown={onInputKeydown}
				/>
			</div>
			<ul class="results">
				{#each filtered as item, index (item.id)}
					<li>
						<button
							class="result"
							class:highlighted={index === safeIndex}
							onmouseenter={() => (highlighted = index)}
							onclick={onClose}
						>
							<span class="result-label">{item.label}</span>
							{#if item.hint}
								<span class="result-hint">{item.hint}</span>
							{/if}
						</button>
					</li>
				{:else}
					<li class="empty">No matching commands</li>
				{/each}
			</ul>
		</div>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 12vh;
		background: oklch(0% 0 0 / 0.5);
		z-index: 100;
	}

	.scrim {
		position: absolute;
		inset: 0;
		background: transparent;
		cursor: default;
	}

	.palette {
		position: relative;
		z-index: 1;
		width: min(560px, 90vw);
		background: var(--popover);
		color: var(--popover-foreground);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-popover);
		overflow: hidden;
	}

	.search {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: 0 var(--space-3);
		height: 44px;
		border-bottom: var(--stroke-thin) solid var(--border-subtle);
		color: var(--muted-foreground);
	}

	.search input {
		flex: 1;
		height: 100%;
		border: none;
		background: transparent;
		color: var(--foreground);
		font-size: var(--text-base);
		outline: none;
	}

	.results {
		max-height: 320px;
		overflow: auto;
		padding: var(--space-1);
	}

	.result {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-md);
		text-align: left;
		color: var(--foreground);
	}

	.result.highlighted {
		background: var(--accent);
		color: var(--accent-foreground);
	}

	.result-label {
		font-size: var(--text-sm);
	}

	.result-hint {
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		color: var(--muted-foreground);
	}

	.empty {
		padding: var(--space-3);
		font-size: var(--text-sm);
		color: var(--muted-foreground);
		text-align: center;
	}
</style>
