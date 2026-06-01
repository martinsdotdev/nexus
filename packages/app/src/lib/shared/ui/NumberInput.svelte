<script lang="ts">
	// A numeric field backed by Ark UI's NumberInput (keyboard stepping, clamping,
	// intermediate-value parsing) presented as a bare input to match the editor's
	// look. The wrapper keeps the plain {value: number, onChange(n)} API; Ark's
	// value is a string, and only finite parses are reported (typing '-', '1.', or
	// clearing never clobbers with NaN). Editor chrome.
	import { NumberInput } from '@ark-ui/svelte/number-input';

	interface Props {
		value: number;
		onChange: (value: number) => void;
		ariaLabel?: string;
		label?: string;
		min?: number;
		max?: number;
		step?: number;
	}
	let { value, onChange, ariaLabel, label, min, max, step }: Props = $props();
</script>

<NumberInput.Root
	value={String(value)}
	onValueChange={(details) => {
		if (Number.isFinite(details.valueAsNumber)) onChange(details.valueAsNumber);
	}}
	{min}
	{max}
	{step}
>
	{#if label}<NumberInput.Label>{label}</NumberInput.Label>{/if}
	<NumberInput.Control>
		<NumberInput.Input aria-label={ariaLabel} />
	</NumberInput.Control>
</NumberInput.Root>

<style>
	:global([data-scope='number-input'][data-part='root']),
	:global([data-scope='number-input'][data-part='control']) {
		display: block;
		width: 100%;
	}

	:global([data-scope='number-input'][data-part='label']) {
		display: block;
		margin-bottom: var(--space-1);
		font-size: var(--text-xs);
		color: var(--muted-foreground);
	}

	:global([data-scope='number-input'][data-part='input']) {
		width: 100%;
		min-width: 0;
		padding: var(--space-1) var(--space-2);
		font: inherit;
		font-size: var(--text-sm);
		color: var(--foreground);
		background: var(--input);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-md);
	}

	:global([data-scope='number-input'][data-part='input']:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
	}
</style>
