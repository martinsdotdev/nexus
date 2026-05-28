<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** Accessible label, also used as the tooltip. */
		label: string;
		/** Whether this button represents the active selection. */
		active?: boolean;
		/** Click handler. */
		onclick?: () => void;
		/** The glyph (typically a lucide icon). */
		children: Snippet;
	}

	let { label, active = false, onclick, children }: Props = $props();
</script>

<button
	class="icon-button"
	class:active
	title={label}
	aria-label={label}
	aria-pressed={active}
	{onclick}
>
	{@render children()}
</button>

<style>
	.icon-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--radius-md);
		color: var(--muted-foreground);
		transition:
			background var(--dur-fast) var(--ease-out),
			color var(--dur-fast) var(--ease-out);
	}

	/* Hover affordance only where a hover-capable pointer exists (skips sticky
	   hover on touch). */
	@media (hover: hover) {
		.icon-button:hover {
			background: var(--accent);
			color: var(--accent-foreground);
		}
	}

	.icon-button:active {
		transform: scale(var(--press-scale));
	}

	.icon-button:focus-visible {
		box-shadow: var(--focus-ring);
	}

	.icon-button.active {
		background: var(--accent);
		color: var(--foreground);
	}

	/* Touch: expand the tap target to 44px via an overlay, keeping the 32px visual
	   so the tool rail stays dense. */
	@media (pointer: coarse) {
		.icon-button {
			position: relative;
		}

		.icon-button::after {
			content: '';
			position: absolute;
			top: 50%;
			left: 50%;
			width: var(--touch-target-min);
			height: var(--touch-target-min);
			transform: translate(-50%, -50%);
		}
	}
</style>
