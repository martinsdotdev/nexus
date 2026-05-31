<script lang="ts">
	// The command palette: Ark UI's Dialog supplies the modal machinery (focus
	// trap, Escape, scrim dismissal, focus restore, background inert) behind the
	// same public API; the search input, fuzzy filter, and Arrow/Enter navigation
	// stay custom (composing Ark Combobox inside a modal Dialog fights over the
	// Escape key and inline-list rendering). unmountOnExit means a fresh mount per
	// open, so query/highlight reset for free. Editor chrome (editor tokens).
	import { Dialog } from '@ark-ui/svelte/dialog';
	import { Search } from 'lucide-svelte';
	import type { CommandItem } from './types';

	interface Props {
		/** Whether the palette is open. */
		open: boolean;
		/** Called when the palette should close (Escape, scrim, or selection). */
		onClose: () => void;
		/** Placeholder for the search input. */
		placeholder?: string;
		/** The command list. */
		items: CommandItem[];
		/** Called with the selected command's id (the palette then closes). */
		onSelect: (id: string) => void;
	}
	let { open, onClose, placeholder = 'Type a command', items, onSelect }: Props = $props();

	let query = $state('');
	let highlighted = $state(0);

	const filtered = $derived(
		items.filter((item) => item.label.toLowerCase().includes(query.trim().toLowerCase()))
	);
	const safeIndex = $derived(Math.max(0, Math.min(highlighted, filtered.length - 1)));

	// Arrow/Enter navigate the list; Escape is handled by the Dialog.
	function onInputKeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			if (filtered.length > 0) highlighted = (safeIndex + 1) % filtered.length;
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			if (filtered.length > 0) highlighted = (safeIndex - 1 + filtered.length) % filtered.length;
		} else if (event.key === 'Enter') {
			event.preventDefault();
			if (filtered.length > 0) {
				onSelect(filtered[safeIndex].id);
				onClose();
			}
		}
	}
</script>

<Dialog.Root
	{open}
	onOpenChange={(details) => {
		if (!details.open) onClose();
	}}
	lazyMount
	unmountOnExit
>
	<Dialog.Backdrop class="palette-scrim" />
	<Dialog.Positioner class="palette-positioner">
		<Dialog.Content class="palette" aria-label="Command palette">
			<div class="palette-search">
				<Search size={16} />
				<input
					bind:value={query}
					type="text"
					{placeholder}
					autocomplete="off"
					spellcheck="false"
					aria-label="Command search"
					onkeydown={onInputKeydown}
				/>
			</div>
			<ul class="palette-results">
				{#each filtered as item, index (item.id)}
					<li>
						<button
							class="palette-result"
							class:highlighted={index === safeIndex}
							onmouseenter={() => (highlighted = index)}
							onclick={() => {
								onSelect(item.id);
								onClose();
							}}
						>
							<span class="result-label">{item.label}</span>
							{#if item.hint}
								<span class="result-hint">{item.hint}</span>
							{/if}
						</button>
					</li>
				{:else}
					<li class="palette-empty">No matching commands</li>
				{/each}
			</ul>
		</Dialog.Content>
	</Dialog.Positioner>
</Dialog.Root>

<style>
	:global(.palette-scrim) {
		position: fixed;
		inset: 0;
		z-index: 100;
		background: oklch(0% 0 0 / 0.5);
	}

	:global(.palette-positioner) {
		position: fixed;
		inset: 0;
		z-index: 101;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 12vh;
	}

	:global(.palette) {
		width: min(560px, 90vw);
		background: var(--popover);
		color: var(--popover-foreground);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-popover);
		overflow: hidden;
		outline: none;
	}

	.palette-search {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: 0 var(--space-3);
		height: 44px;
		border-bottom: var(--stroke-thin) solid var(--border-subtle);
		color: var(--muted-foreground);
	}

	.palette-search input {
		flex: 1;
		height: 100%;
		border: none;
		background: transparent;
		color: var(--foreground);
		font: inherit;
		font-size: var(--text-base);
		outline: none;
	}

	.palette-results {
		max-height: 320px;
		overflow: auto;
		padding: var(--space-1);
		margin: 0;
		list-style: none;
	}

	.palette-result {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-md);
		text-align: left;
		color: var(--foreground);
		background: transparent;
		border: none;
		cursor: pointer;
		font: inherit;
	}

	.palette-result.highlighted {
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

	.palette-empty {
		padding: var(--space-3);
		font-size: var(--text-sm);
		color: var(--muted-foreground);
		text-align: center;
	}
</style>
