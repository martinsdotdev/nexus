<script lang="ts">
	// A disclosure group backed by Ark UI's Collapsible (animated, accessible
	// trigger/content), replacing native <details>/<summary>. Plain {title, open?,
	// children} API. Editor chrome; the chevron rotates on open via [data-state].
	import type { Snippet } from 'svelte';
	import { Collapsible } from '@ark-ui/svelte/collapsible';
	import { ChevronRight } from 'lucide-svelte';

	interface Props {
		title: string;
		/** Expanded on first render. */
		open?: boolean;
		children: Snippet;
	}
	let { title, open = false, children }: Props = $props();
</script>

<Collapsible.Root defaultOpen={open}>
	<Collapsible.Trigger>
		<Collapsible.Indicator><ChevronRight size={12} /></Collapsible.Indicator>
		<span>{title}</span>
	</Collapsible.Trigger>
	<Collapsible.Content>{@render children()}</Collapsible.Content>
</Collapsible.Root>

<style>
	:global([data-scope='collapsible'][data-part='trigger']) {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		width: 100%;
		padding: var(--space-1) 0;
		background: none;
		border: none;
		cursor: pointer;
		font: inherit;
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		text-align: left;
	}

	:global([data-scope='collapsible'][data-part='trigger']:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
		border-radius: var(--radius-sm);
	}

	:global([data-scope='collapsible'][data-part='indicator']) {
		display: inline-flex;
		transition: transform var(--dur-fast) var(--ease-out);
	}

	:global([data-scope='collapsible'][data-part='indicator'][data-state='open']) {
		transform: rotate(90deg);
	}
</style>
