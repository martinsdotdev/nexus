<script lang="ts">
	// An overflow ("...") action menu over Ark UI's Menu (real menu/menuitem ARIA, roving
	// keyboard nav, typeahead, portaled positioning). Plain data API: a list of items and an
	// onSelect. The trigger is a baked icon button, so call sites stay declarative and the
	// wrapper is testable without snippet props. Styled with editor tokens via :global() on
	// Ark's [data-part] anatomy. Editor chrome only.
	import { Menu } from '@ark-ui/svelte/menu';
	import { MoreHorizontal } from 'lucide-svelte';

	interface Item {
		value: string;
		label: string;
		/** Renders in the destructive colour (e.g. Delete). */
		destructive?: boolean;
	}
	interface Props {
		items: Item[];
		onSelect: (value: string) => void;
		/** Accessible name for the trigger button. */
		label: string;
	}
	let { items, onSelect, label }: Props = $props();
</script>

<Menu.Root onSelect={(details) => onSelect(details.value)} lazyMount unmountOnExit>
	<Menu.Trigger class="menu-trigger" aria-label={label}>
		<MoreHorizontal size={16} />
	</Menu.Trigger>
	<Menu.Positioner>
		<Menu.Content>
			{#each items as item (item.value)}
				<Menu.Item value={item.value} data-destructive={item.destructive ? '' : undefined}>
					{item.label}
				</Menu.Item>
			{/each}
		</Menu.Content>
	</Menu.Positioner>
</Menu.Root>

<style>
	:global(.menu-trigger) {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: var(--radius-md);
		color: var(--muted-foreground);
		cursor: pointer;
		transition: color var(--dur-fast) var(--ease-out);
	}

	@media (hover: hover) {
		:global(.menu-trigger:hover) {
			background: var(--accent);
			color: var(--foreground);
		}
	}

	:global(.menu-trigger:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	:global([data-scope='menu'][data-part='content']) {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 9rem;
		padding: var(--space-1);
		background: var(--popover);
		color: var(--popover-foreground);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-popover);
		z-index: 100;
	}

	:global([data-scope='menu'][data-part='item']) {
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-sm);
		font-size: var(--text-sm);
		color: var(--foreground);
		cursor: pointer;
	}

	:global([data-scope='menu'][data-part='item'][data-highlighted]) {
		background: var(--accent);
		color: var(--accent-foreground);
	}

	:global([data-scope='menu'][data-part='item'][data-destructive]) {
		color: var(--destructive);
	}
</style>
