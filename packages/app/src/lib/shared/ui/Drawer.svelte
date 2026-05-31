<script lang="ts">
	// An off-canvas drawer: Ark UI's Dialog (modal focus trap, Escape, scrim
	// dismissal, focus restore, background inert) behind the same public API the
	// shell already uses. We supply only the presentation: a side-pinned panel that
	// slides via CSS keyframes keyed on Ark's [data-state], so it animates both in
	// and out (Ark keeps the content mounted through the closing animation). Editor
	// chrome; styled with tokens via :global on the portaled dialog parts.
	import type { Snippet } from 'svelte';
	import { Dialog } from '@ark-ui/svelte/dialog';
	import { X } from 'lucide-svelte';

	interface Props {
		/** Whether the drawer is open. */
		open: boolean;
		/** Called when the drawer should close (Escape, scrim, or the close button). */
		onClose: () => void;
		/** Which edge the drawer slides from. */
		side?: 'left' | 'right';
		/** Title shown in the drawer header; also labels the dialog and close button. */
		title: string;
		/** Drawer body content (typically a StudioPanel in its drawer variant). */
		children: Snippet;
	}
	let { open, onClose, side = 'left', title, children }: Props = $props();
</script>

<Dialog.Root
	{open}
	onOpenChange={(details) => {
		if (!details.open) onClose();
	}}
	lazyMount
	unmountOnExit
>
	<Dialog.Backdrop class="drawer-scrim" />
	<Dialog.Positioner class="drawer-positioner" data-side={side}>
		<Dialog.Content class="drawer" data-side={side}>
			<header class="drawer-header">
				<Dialog.Title class="drawer-title">{title}</Dialog.Title>
				<Dialog.CloseTrigger class="drawer-close" aria-label="Close {title}">
					<X size={18} strokeWidth={1.75} />
				</Dialog.CloseTrigger>
			</header>
			<div class="drawer-body">{@render children()}</div>
		</Dialog.Content>
	</Dialog.Positioner>
</Dialog.Root>

<style>
	:global(.drawer-scrim) {
		position: fixed;
		inset: 0;
		z-index: 100;
		background: oklch(0% 0 0 / 0.5);
	}
	:global(.drawer-scrim[data-state='open']) {
		animation: drawer-fade-in var(--dur-normal) var(--ease-out);
	}
	:global(.drawer-scrim[data-state='closed']) {
		animation: drawer-fade-out var(--dur-normal) var(--ease-out);
	}

	:global(.drawer-positioner) {
		position: fixed;
		inset: 0;
		z-index: 101;
		display: flex;
	}
	:global(.drawer-positioner[data-side='left']) {
		justify-content: flex-start;
	}
	:global(.drawer-positioner[data-side='right']) {
		justify-content: flex-end;
	}

	:global(.drawer) {
		display: flex;
		flex-direction: column;
		width: var(--drawer-width);
		max-width: 100%;
		height: 100%;
		background: var(--card);
		color: var(--card-foreground);
		box-shadow: var(--shadow-popover);
		outline: none;
	}
	:global(.drawer[data-side='left'][data-state='open']) {
		animation: drawer-in-left var(--dur-slow) var(--ease-out);
	}
	:global(.drawer[data-side='left'][data-state='closed']) {
		animation: drawer-out-left var(--dur-slow) var(--ease-out);
	}
	:global(.drawer[data-side='right'][data-state='open']) {
		animation: drawer-in-right var(--dur-slow) var(--ease-out);
	}
	:global(.drawer[data-side='right'][data-state='closed']) {
		animation: drawer-out-right var(--dur-slow) var(--ease-out);
	}

	:global(.drawer-header) {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
		height: var(--titlebar-height);
		padding: 0 var(--space-2) 0 var(--space-3);
		border-bottom: var(--stroke-thin) solid var(--divider);
	}
	:global(.drawer-title) {
		margin: 0;
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--foreground);
	}
	:global(.drawer-close) {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--radius-md);
		color: var(--muted-foreground);
		cursor: pointer;
		transition: background var(--dur-fast) var(--ease-out);
	}
	:global(.drawer-close:active) {
		transform: scale(var(--press-scale));
	}
	:global(.drawer-close:focus-visible) {
		box-shadow: var(--focus-ring);
	}
	:global(.drawer-body) {
		flex: 1;
		overflow: auto;
	}

	@media (hover: hover) {
		:global(.drawer-close:hover) {
			background: var(--accent);
			color: var(--accent-foreground);
		}
	}

	@media (pointer: coarse) {
		:global(.drawer-close) {
			position: relative;
		}
		:global(.drawer-close::after) {
			content: '';
			position: absolute;
			top: 50%;
			left: 50%;
			width: var(--touch-target-min);
			height: var(--touch-target-min);
			transform: translate(-50%, -50%);
		}
	}

	/* Global keyframes (the -global- prefix keeps the names literal so the :global
	   rules above can reference them; Ark's Presence keeps the content mounted while
	   the closing animation runs, then unmounts). */
	@keyframes -global-drawer-fade-in {
		from {
			opacity: 0;
		}
	}
	@keyframes -global-drawer-fade-out {
		to {
			opacity: 0;
		}
	}
	@keyframes -global-drawer-in-left {
		from {
			transform: translateX(-100%);
		}
	}
	@keyframes -global-drawer-out-left {
		to {
			transform: translateX(-100%);
		}
	}
	@keyframes -global-drawer-in-right {
		from {
			transform: translateX(100%);
		}
	}
	@keyframes -global-drawer-out-right {
		to {
			transform: translateX(100%);
		}
	}
</style>
