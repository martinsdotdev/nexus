<script lang="ts">
	import type { Snippet } from 'svelte';
	import { X } from 'lucide-svelte';

	interface Props {
		/** Whether the drawer is open. */
		open: boolean;
		/** Called when the drawer should close (Escape, scrim, or the close button). */
		onClose: () => void;
		/** Which edge the drawer slides from. */
		side?: 'left' | 'right';
		/** Title shown in the drawer header; also labels the dialog and the close button. */
		title: string;
		/** Drawer body content (typically a StudioPanel in its drawer variant). */
		children: Snippet;
	}

	let { open, onClose, side = 'left', title, children }: Props = $props();

	let panelEl = $state<HTMLElement | null>(null);
	let previousFocus: HTMLElement | null = null;

	// Focus management: capture on open, move focus into the dialog, restore on close.
	// Mirrors CommandPalette. The drawer stays mounted, so visibility (not {#if}) gates it.
	$effect(() => {
		if (open) {
			previousFocus = document.activeElement as HTMLElement | null;
			panelEl?.focus();
		} else if (previousFocus) {
			previousFocus.focus();
			previousFocus = null;
		}
	});

	const FOCUSABLE =
		'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

	function focusable(): HTMLElement[] {
		if (!panelEl) return [];
		return Array.from(panelEl.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
			(el) => el.offsetParent !== null
		);
	}

	// Escape closes; Tab is trapped within the panel. On svelte:window so the handler
	// does not live on the non-interactive dialog element (keeps a11y lint clean), and
	// it no-ops unless this drawer is the open one.
	function onWindowKeydown(event: KeyboardEvent) {
		if (!open || !panelEl) return;
		if (event.key === 'Escape') {
			event.preventDefault();
			onClose();
			return;
		}
		if (event.key !== 'Tab') return;
		const items = focusable();
		const active = document.activeElement as HTMLElement | null;
		if (items.length === 0) {
			event.preventDefault();
			panelEl.focus();
			return;
		}
		const first = items[0];
		const last = items[items.length - 1];
		if (!active || !panelEl.contains(active)) {
			event.preventDefault();
			first.focus();
		} else if (event.shiftKey && active === first) {
			event.preventDefault();
			last.focus();
		} else if (!event.shiftKey && active === last) {
			event.preventDefault();
			first.focus();
		}
	}
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div class="backdrop" class:open class:left={side === 'left'} class:right={side === 'right'}>
	<!-- Pointer dismissal behind the dialog; a real button for a11y, kept out of the tab
	     order (the header close button is the in-trap keyboard affordance). -->
	<button class="scrim" tabindex="-1" aria-label="Close {title}" onclick={onClose}></button>
	<div
		class="drawer"
		role="dialog"
		aria-modal="true"
		aria-label={title}
		tabindex="-1"
		bind:this={panelEl}
	>
		<header class="drawer-header">
			<span class="drawer-title">{title}</span>
			<button class="drawer-close" aria-label="Close {title}" onclick={onClose}>
				<X size={18} strokeWidth={1.75} />
			</button>
		</header>
		<div class="drawer-body">
			{@render children()}
		</div>
	</div>
</div>

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 100;
		display: flex;
		visibility: hidden;
		pointer-events: none;
		/* Delay the visibility flip to hidden until the slide-out finishes. */
		transition: visibility 0s linear var(--dur-slow);
	}

	.backdrop.left {
		justify-content: flex-start;
	}

	.backdrop.right {
		justify-content: flex-end;
	}

	.backdrop.open {
		visibility: visible;
		pointer-events: auto;
		transition: visibility 0s linear 0s;
	}

	.scrim {
		position: absolute;
		inset: 0;
		background: oklch(0% 0 0 / 0.5);
		opacity: 0;
		transition: opacity var(--dur-normal) var(--ease-out);
		cursor: default;
	}

	.backdrop.open .scrim {
		opacity: 1;
	}

	.drawer {
		position: relative;
		z-index: 1;
		display: flex;
		flex-direction: column;
		width: var(--drawer-width);
		max-width: 100%;
		height: 100%;
		background: var(--card);
		color: var(--card-foreground);
		box-shadow: var(--shadow-popover);
		outline: none;
		transition: transform var(--dur-slow) var(--ease-out);
	}

	.backdrop.left .drawer {
		transform: translateX(-100%);
	}

	.backdrop.right .drawer {
		transform: translateX(100%);
	}

	/* Side-specific so the open state outranks the closed .left/.right rules above by
	   specificity, not merely source order. */
	.backdrop.left.open .drawer,
	.backdrop.right.open .drawer {
		transform: translateX(0);
	}

	.drawer-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
		height: var(--titlebar-height);
		padding: 0 var(--space-2) 0 var(--space-3);
		border-bottom: var(--stroke-thin) solid var(--divider);
	}

	.drawer-title {
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--foreground);
	}

	.drawer-close {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--radius-md);
		color: var(--muted-foreground);
		transition: background var(--dur-fast) var(--ease-out);
	}

	.drawer-close:active {
		transform: scale(var(--press-scale));
	}

	.drawer-close:focus-visible {
		box-shadow: var(--focus-ring);
	}

	.drawer-body {
		flex: 1;
		overflow: auto;
	}

	/* Hover affordance only where a hover-capable pointer exists. */
	@media (hover: hover) {
		.drawer-close:hover {
			background: var(--accent);
			color: var(--accent-foreground);
		}
	}

	/* Touch: expand the close target to the 44px minimum via an overlay, so the glyph
	   and the 40px header keep their size. */
	@media (pointer: coarse) {
		.drawer-close {
			position: relative;
		}

		.drawer-close::after {
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
