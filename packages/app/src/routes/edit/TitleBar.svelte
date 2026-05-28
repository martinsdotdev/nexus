<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Undo2, Redo2, MonitorPlay, PanelLeft, PanelRight } from 'lucide-svelte';

	interface Props {
		/** Whether the left widgets drawer is open (narrow viewports). */
		leftDrawerOpen: boolean;
		/** Whether the right inspector drawer is open (medium and narrower). */
		rightDrawerOpen: boolean;
		/** Collapse the docked left panel, or toggle its drawer when off-canvas. */
		onToggleLeftDock: () => void;
		/** Collapse the docked right panel, or toggle its drawer when off-canvas. */
		onToggleRightDock: () => void;
	}

	let { leftDrawerOpen, rightDrawerOpen, onToggleLeftDock, onToggleRightDock }: Props = $props();
</script>

<header class="titlebar">
	<div class="brand">
		<!-- Shown only when the widgets panel is off-canvas (narrow); opens its drawer. -->
		<button
			class="ghost toggle-left"
			aria-label={m['editor.drawer.toggle_widgets']()}
			aria-expanded={leftDrawerOpen}
			onclick={onToggleLeftDock}
		>
			<PanelLeft size={16} />
		</button>
		<span class="mark">{m['app.name']()}</span>
		<span class="chip" title={m['editor.titlebar.draft']()}>{m['editor.titlebar.draft']()}</span>
	</div>

	<div class="actions">
		<!-- Shown when the inspector is off-canvas (medium and narrower); opens its drawer. -->
		<button
			class="ghost toggle-right"
			aria-label={m['editor.drawer.toggle_inspector']()}
			aria-expanded={rightDrawerOpen}
			onclick={onToggleRightDock}
		>
			<PanelRight size={16} />
		</button>

		<!-- Inert chrome in this prototype: visual only, wired to nothing. -->
		<button class="ghost" aria-label="Undo" disabled><Undo2 size={16} /></button>
		<button class="ghost" aria-label="Redo" disabled><Redo2 size={16} /></button>
		<button class="primary" aria-label={m['editor.command.use_in_obs']()} disabled>
			<MonitorPlay size={15} />
			<span class="primary-label">{m['editor.command.use_in_obs']()}</span>
		</button>
	</div>
</header>

<style>
	.titlebar {
		grid-area: titlebar;
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: var(--titlebar-height);
		padding: 0 var(--space-3);
		background: var(--titlebar);
		color: var(--titlebar-foreground);
		border-bottom: var(--stroke-thin) solid var(--divider);
	}

	.brand {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		min-width: 0;
	}

	.mark {
		font-weight: 700;
		font-size: var(--text-base);
		color: var(--foreground);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.chip {
		flex: none;
		padding: 2px var(--space-2);
		border-radius: var(--radius-circular);
		background: var(--invert);
		color: var(--invert-foreground);
		font-size: var(--text-xs);
		font-weight: 600;
		letter-spacing: 0.02em;
	}

	.actions {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.ghost {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: var(--radius-md);
		color: var(--muted-foreground);
		transition: background var(--dur-fast) var(--ease-out);
	}

	.primary {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		height: 28px;
		padding: 0 var(--space-3);
		border-radius: var(--radius-md);
		background: var(--primary);
		color: var(--primary-foreground);
		font-size: var(--text-sm);
		font-weight: 600;
	}

	/*
	 * Panel-toggle visibility. Each toggle targets its OWN class for both the hidden
	 * default and the shown override (symmetric specificity), so the media-query rule
	 * wins purely by source order. Avoids a shared base class whose Svelte-scoped
	 * specificity could outrank the override. The toggles only do anything via JS, so
	 * hiding them by default is the correct no-JS state too.
	 */
	.toggle-left,
	.toggle-right {
		display: none;
	}

	/* The inspector goes off-canvas first (medium and narrower): show its toggle. */
	@media (max-width: 1279px) {
		.toggle-right {
			display: inline-flex;
		}
	}

	/* The widgets panel goes off-canvas in the narrow zone: show its toggle. */
	@media (max-width: 959px) {
		.toggle-left {
			display: inline-flex;
		}
	}

	/* Below the floor, drop the draft chip and OBS label to reclaim width. */
	@media (max-width: 639px) {
		.chip {
			display: none;
		}

		.primary-label {
			display: none;
		}

		.primary {
			padding: 0 var(--space-2);
		}
	}

	@media (hover: hover) {
		.ghost:not(:disabled):hover {
			background: var(--accent);
			color: var(--accent-foreground);
		}
	}

	.ghost:not(:disabled):active {
		transform: scale(var(--press-scale));
	}

	.ghost:focus-visible,
	.primary:focus-visible {
		box-shadow: var(--focus-ring);
	}

	/* Inert in the prototype: dim the disabled chrome but keep it visible. */
	.ghost:disabled,
	.primary:disabled {
		opacity: 0.55;
		cursor: default;
	}

	/* Touch: expand interactive targets to 44px via an overlay, so the 40px title bar
	   and the dense button visuals are unaffected. */
	@media (pointer: coarse) {
		.ghost:not(:disabled),
		.primary:not(:disabled) {
			position: relative;
		}

		.ghost:not(:disabled)::after,
		.primary:not(:disabled)::after {
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
