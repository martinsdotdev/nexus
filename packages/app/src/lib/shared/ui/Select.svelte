<script lang="ts">
	// A single-select dropdown backed by Ark UI's Select (a Zag state machine with
	// real listbox/combobox ARIA, keyboard nav, and a portaled popover). The wrapper
	// presents a plain single-string API; it adapts to Ark's `value: string[]` +
	// details-object callbacks at this one boundary, so call sites stay dumb. Styled
	// entirely with editor tokens via :global() on Ark's [data-part] anatomy (the
	// rendered parts live in Ark's component scope + the popover is portaled, so
	// scoped selectors cannot reach them). Editor chrome only.
	import { Select, createListCollection } from '@ark-ui/svelte/select';
	import { ChevronDown, Check } from 'lucide-svelte';

	interface Option {
		value: string;
		label: string;
	}
	interface Props {
		value: string;
		options: Option[];
		onChange: (value: string) => void;
		/** Visible label rendered above the control. */
		label?: string;
		/** Accessible name when there is no visible label. */
		ariaLabel?: string;
		placeholder?: string;
		/** Chevron-only trigger (the theme builder's per-token link picker). */
		compact?: boolean;
	}
	let {
		value,
		options,
		onChange,
		label,
		ariaLabel,
		placeholder = '',
		compact = false
	}: Props = $props();

	const collection = $derived(
		createListCollection({
			items: options,
			itemToValue: (item) => item.value,
			itemToString: (item) => item.label
		})
	);
</script>

<Select.Root
	{collection}
	value={[value]}
	onValueChange={(details) => onChange(details.value[0] ?? '')}
	positioning={{ sameWidth: !compact }}
>
	{#if label}<Select.Label>{label}</Select.Label>{/if}
	<Select.Control>
		<Select.Trigger aria-label={ariaLabel} data-compact={compact ? '' : undefined}>
			{#if !compact}<Select.ValueText {placeholder} />{/if}
			<Select.Indicator><ChevronDown size={14} /></Select.Indicator>
		</Select.Trigger>
	</Select.Control>
	<Select.Positioner>
		<Select.Content>
			{#each options as option (option.value)}
				<Select.Item item={option}>
					<Select.ItemText>{option.label}</Select.ItemText>
					<Select.ItemIndicator><Check size={14} /></Select.ItemIndicator>
				</Select.Item>
			{/each}
		</Select.Content>
	</Select.Positioner>
</Select.Root>

<style>
	/* Ark renders its own elements (in its component scope) and portals the popover,
	   so every part is styled through :global on its data-scope/data-part anatomy. */
	:global([data-scope='select'][data-part='label']) {
		display: block;
		margin-bottom: var(--space-1);
		font-size: var(--text-xs);
		color: var(--muted-foreground);
	}

	:global([data-scope='select'][data-part='trigger']) {
		display: inline-flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-1);
		width: 100%;
		min-width: 0;
		padding: var(--space-1) var(--space-2);
		font: inherit;
		font-size: var(--text-sm);
		color: var(--foreground);
		background: var(--input);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-md);
		cursor: pointer;
	}

	:global([data-scope='select'][data-part='trigger'][data-compact]) {
		width: 2.4ch;
		padding: var(--space-1);
		justify-content: center;
	}

	:global([data-scope='select'][data-part='trigger']:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	:global([data-scope='select'][data-part='indicator']) {
		display: inline-flex;
		color: var(--muted-foreground);
	}

	:global([data-scope='select'][data-part='content']) {
		display: flex;
		flex-direction: column;
		gap: 1px;
		max-height: 18rem;
		overflow-y: auto;
		padding: var(--space-1);
		background: var(--popover);
		color: var(--popover-foreground);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-popover);
		z-index: 50;
	}

	:global([data-scope='select'][data-part='item']) {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-sm);
		font-size: var(--text-sm);
		color: var(--foreground);
		cursor: pointer;
	}

	:global([data-scope='select'][data-part='item'][data-highlighted]) {
		background: var(--accent);
		color: var(--accent-foreground);
	}

	:global([data-scope='select'][data-part='item-indicator']) {
		display: inline-flex;
		color: var(--primary);
	}
</style>
