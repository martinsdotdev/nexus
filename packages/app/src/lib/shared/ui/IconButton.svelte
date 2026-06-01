<script lang="ts">
	// An icon-only button with an Ark UI Tooltip (the Trigger IS the button, so it
	// keeps the click/active/aria semantics). Icon-only controls need a tooltip to
	// be legible, so it is baked in rather than optional. Editor chrome.
	import type { Snippet } from 'svelte';
	import { Tooltip } from '@ark-ui/svelte/tooltip';

	interface Props {
		/** Accessible label, also shown as the tooltip. */
		label: string;
		/** Whether this button represents the active selection. */
		active?: boolean;
		/** Click handler. */
		onclick?: () => void;
		/** Tooltip placement relative to the button. */
		placement?: 'top' | 'right' | 'bottom' | 'left';
		/** The glyph (typically a lucide icon). */
		children: Snippet;
	}
	let { label, active = false, onclick, placement = 'right', children }: Props = $props();
</script>

<Tooltip.Root openDelay={350} closeDelay={80} positioning={{ placement }} lazyMount unmountOnExit>
	<Tooltip.Trigger
		class="icon-button {active ? 'active' : ''}"
		aria-label={label}
		aria-pressed={active}
		{onclick}
	>
		{@render children()}
	</Tooltip.Trigger>
	<Tooltip.Positioner>
		<Tooltip.Content>{label}</Tooltip.Content>
	</Tooltip.Positioner>
</Tooltip.Root>

<style>
	/* The class sits on Ark's Trigger element, outside this component's CSS scope. */
	:global(.icon-button) {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--radius-md);
		color: var(--muted-foreground);
		cursor: pointer;
		transition:
			background var(--dur-fast) var(--ease-out),
			color var(--dur-fast) var(--ease-out);
	}

	@media (hover: hover) {
		:global(.icon-button:hover) {
			background: var(--accent);
			color: var(--accent-foreground);
		}
	}

	:global(.icon-button:active) {
		transform: scale(var(--press-scale));
	}

	:global(.icon-button:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	:global(.icon-button.active) {
		background: var(--accent);
		color: var(--foreground);
	}

	@media (pointer: coarse) {
		:global(.icon-button) {
			position: relative;
		}
		:global(.icon-button::after) {
			content: '';
			position: absolute;
			top: 50%;
			left: 50%;
			width: var(--touch-target-min);
			height: var(--touch-target-min);
			transform: translate(-50%, -50%);
		}
	}

	:global([data-scope='tooltip'][data-part='content']) {
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-sm);
		background: var(--popover);
		color: var(--popover-foreground);
		border: var(--stroke-thin) solid var(--border);
		box-shadow: var(--shadow-popover);
		font-size: var(--text-xs);
		z-index: 200;
	}
</style>
