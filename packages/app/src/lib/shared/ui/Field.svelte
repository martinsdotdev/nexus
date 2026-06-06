<script lang="ts">
	// A labelled text field over Ark UI's Field: real label / control / helper / error wiring
	// (aria-invalid, aria-describedby, the required indicator) behind a plain single-value API.
	// The form layer (TanStack Form) drives `value` and `error`; this renders them. Styled with
	// editor tokens via :global() on Ark's [data-part] anatomy. Editor chrome only.
	import { Field } from '@ark-ui/svelte/field';
	import type { HTMLInputAttributes } from 'svelte/elements';

	interface Props {
		value: string;
		/** Fires with the input's new value on every keystroke. */
		oninput?: (value: string) => void;
		onblur?: () => void;
		/** Visible label above the control. */
		label?: string;
		/** Accessible name when there is no visible label. */
		ariaLabel?: string;
		/** When non-empty the field is invalid and this is shown as the error text. */
		error?: string;
		/** Supporting text shown when there is no error. */
		hint?: string;
		required?: boolean;
		type?: 'text' | 'email' | 'password' | 'url';
		placeholder?: string;
		name?: string;
		autocomplete?: HTMLInputAttributes['autocomplete'];
		inputmode?: 'text' | 'email' | 'numeric' | 'tel' | 'url' | 'search';
		maxlength?: number;
	}
	let {
		value,
		oninput,
		onblur,
		label,
		ariaLabel,
		error,
		hint,
		required = false,
		type = 'text',
		placeholder,
		name,
		autocomplete,
		inputmode,
		maxlength
	}: Props = $props();

	const invalid = $derived(!!error);
</script>

<Field.Root {invalid} {required}>
	{#if label}<Field.Label>{label}</Field.Label>{/if}
	<Field.Input
		{value}
		{type}
		{name}
		{placeholder}
		{autocomplete}
		{inputmode}
		{maxlength}
		aria-label={label ? undefined : ariaLabel}
		oninput={(event) => oninput?.((event.currentTarget as HTMLInputElement).value)}
		onblur={() => onblur?.()}
	/>
	{#if invalid}
		<Field.ErrorText>{error}</Field.ErrorText>
	{:else if hint}
		<Field.HelperText>{hint}</Field.HelperText>
	{/if}
</Field.Root>

<style>
	/* Ark renders its own elements, so each part is styled through :global on its anatomy. */
	:global([data-scope='field'][data-part='root']) {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
	}

	:global([data-scope='field'][data-part='label']) {
		font-size: var(--text-sm);
		color: var(--muted-foreground);
	}

	:global([data-scope='field'][data-part='input']) {
		height: 36px;
		padding: 0 var(--space-3);
		font: inherit;
		font-size: var(--text-base);
		color: var(--foreground);
		background: var(--input);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-md);
		transition: border-color var(--dur-fast) var(--ease-out);
	}

	:global([data-scope='field'][data-part='input']::placeholder) {
		color: var(--muted-foreground);
	}

	:global([data-scope='field'][data-part='input']:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	:global([data-scope='field'][data-part='input'][data-invalid]) {
		border-color: var(--destructive);
	}

	:global([data-scope='field'][data-part='helper-text']) {
		font-size: var(--text-sm);
		color: var(--muted-foreground);
		line-height: var(--leading-base);
	}

	:global([data-scope='field'][data-part='error-text']) {
		font-size: var(--text-sm);
		color: var(--destructive);
		line-height: var(--leading-base);
	}
</style>
