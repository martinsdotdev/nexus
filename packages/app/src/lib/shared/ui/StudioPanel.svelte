<script lang="ts">
	import type { Snippet } from 'svelte';
	import { ChevronLeft, ChevronRight, GripVertical } from 'lucide-svelte';

	interface Props {
		/** Panel title shown in the header band (docked variant only). */
		title: string;
		/** Collapsed state (header only when true). Docked variant only. */
		collapsed?: boolean;
		/** Toggle handler for the collapse chevron. Docked variant only. */
		onToggleCollapse?: () => void;
		/** Which edge the panel docks to; sets the chevron direction. */
		side?: 'left' | 'right';
		/**
		 * 'docked' is the in-grid panel; 'drawer' fills an off-canvas Drawer (no header,
		 * flat, full-width, since the Drawer supplies the surface and close affordance).
		 */
		variant?: 'docked' | 'drawer';
		/** Panel body content. */
		children: Snippet;
	}

	let {
		title,
		collapsed = false,
		onToggleCollapse,
		side = 'left',
		variant = 'docked',
		children
	}: Props = $props();

	const isDocked = $derived(variant === 'docked');

	// The chevron points "outward" (toward the docked edge) to collapse.
	const collapseToward = $derived(side === 'left' ? ChevronLeft : ChevronRight);
	const expandToward = $derived(side === 'left' ? ChevronRight : ChevronLeft);
	const Chevron = $derived(collapsed ? expandToward : collapseToward);
</script>

<section
	class="studio-panel"
	class:collapsed={isDocked && collapsed}
	class:drawer={!isDocked}
	style={isDocked ? `width: ${collapsed ? 'auto' : 'var(--panel-default-width)'}` : ''}
>
	{#if isDocked}
		<header class="panel-header">
			<!-- Faux drag handle: panels look dockable; real drag-docking is deferred. -->
			<span class="drag-handle" aria-hidden="true"><GripVertical size={14} /></span>
			{#if !collapsed}
				<span class="panel-title">{title}</span>
			{/if}
			<button
				class="collapse-toggle"
				title={collapsed ? `Expand ${title}` : `Collapse ${title}`}
				aria-label={collapsed ? `Expand ${title}` : `Collapse ${title}`}
				aria-expanded={!collapsed}
				onclick={onToggleCollapse}
			>
				<Chevron size={16} />
			</button>
		</header>
	{/if}
	{#if !isDocked || !collapsed}
		<div class="panel-body">
			{@render children()}
		</div>
	{/if}
</section>

<style>
	.studio-panel {
		display: flex;
		flex-direction: column;
		min-width: var(--panel-min-width);
		background: var(--card);
		color: var(--card-foreground);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-panel);
		overflow: hidden;
	}

	/*
	 * Establish a size container so the body can adapt to the panel's own width.
	 * Skipped when collapsed: a collapsed panel uses width:auto and must shrink-wrap
	 * its header, which inline-size containment would otherwise prevent.
	 */
	.studio-panel:not(.collapsed) {
		container-type: inline-size;
	}

	.studio-panel.collapsed {
		min-width: 0;
	}

	/*
	 * Drawer variant: the Drawer supplies the surface, header, and close affordance,
	 * so the panel is flat, full-width, and header-less.
	 */
	.studio-panel.drawer {
		width: 100%;
		height: 100%;
		min-width: 0;
		border-radius: 0;
		box-shadow: none;
	}

	.panel-header {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		height: var(--panel-header-height);
		padding: 0 var(--space-2);
		background: var(--panel-header);
		border-bottom: var(--stroke-thin) solid var(--border-subtle);
	}

	.drag-handle {
		display: inline-flex;
		color: var(--muted-foreground);
		cursor: grab;
	}

	.panel-title {
		flex: 1;
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--foreground);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.collapse-toggle {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		border-radius: var(--radius-sm);
		color: var(--muted-foreground);
		transition: background var(--dur-fast) var(--ease-out);
	}

	@media (hover: hover) {
		.collapse-toggle:hover {
			background: var(--accent);
			color: var(--accent-foreground);
		}
	}

	.collapse-toggle:focus-visible {
		box-shadow: var(--focus-ring);
	}

	.panel-body {
		flex: 1;
		padding: var(--space-3);
		overflow: auto;
		font-size: var(--text-sm);
	}

	/*
	 * Container query: when the panel itself is squeezed (a narrow drawer on a tiny
	 * viewport, or a future resized dock below 260px), tighten the body padding.
	 */
	@container (max-width: 260px) {
		.panel-body {
			padding: var(--space-2);
		}
	}

	/* Touch: expand the collapse target to 44px via an overlay, keeping the 22px glyph. */
	@media (pointer: coarse) {
		.collapse-toggle {
			position: relative;
		}

		.collapse-toggle::after {
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
