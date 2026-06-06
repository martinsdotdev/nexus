<script lang="ts">
	// Renders the shared toast store (see ./toast). Mount once per app page that needs toasts
	// (editor, login, workspaces); never in the root layout, so the Ark-free `/overlay` bundle
	// stays clean. Each toast is a card tinted on the left by its kind. Styled with editor
	// tokens via :global() on Ark's [data-part] anatomy.
	import { Toast, Toaster } from '@ark-ui/svelte/toast';
	import { X } from 'lucide-svelte';
	import { toaster } from './toast';
</script>

<Toaster {toaster}>
	{#snippet children(toast)}
		<Toast.Root>
			<div class="body">
				<Toast.Title>{toast().title}</Toast.Title>
				{#if toast().description}
					<Toast.Description>{toast().description}</Toast.Description>
				{/if}
			</div>
			<Toast.CloseTrigger aria-label="Dismiss">
				<X size={14} />
			</Toast.CloseTrigger>
		</Toast.Root>
	{/snippet}
</Toaster>

<style>
	:global([data-scope='toast'][data-part='root']) {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		width: min(360px, 86vw);
		padding: var(--space-3) var(--space-4);
		color: var(--popover-foreground);
		background: var(--popover);
		border: var(--stroke-thin) solid var(--border);
		border-left-width: 3px;
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-popover);
	}

	:global([data-scope='toast'][data-part='root'][data-type='success']) {
		border-left-color: var(--success);
	}
	:global([data-scope='toast'][data-part='root'][data-type='error']) {
		border-left-color: var(--destructive);
	}
	:global([data-scope='toast'][data-part='root'][data-type='info']) {
		border-left-color: var(--info);
	}

	.body {
		flex: 1;
		min-width: 0;
	}

	:global([data-scope='toast'][data-part='title']) {
		font-size: var(--text-sm);
		font-weight: 600;
	}

	:global([data-scope='toast'][data-part='description']) {
		margin-top: 2px;
		font-size: var(--text-sm);
		color: var(--muted-foreground);
		line-height: var(--leading-base);
	}

	:global([data-scope='toast'][data-part='close-trigger']) {
		display: inline-flex;
		flex: none;
		color: var(--muted-foreground);
		border-radius: var(--radius-sm);
		transition: color var(--dur-fast) var(--ease-out);
	}

	:global([data-scope='toast'][data-part='close-trigger']:hover) {
		color: var(--foreground);
	}

	:global([data-scope='toast'][data-part='close-trigger']:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
	}
</style>
