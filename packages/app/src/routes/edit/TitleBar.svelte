<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Undo2, Redo2, MonitorPlay, PanelLeft, PanelRight, Search } from 'lucide-svelte';
	import Roster, { type RosterPerson } from '$lib/features/presence/ui/Roster.svelte';
	import SyncPill from '$lib/features/presence/ui/SyncPill.svelte';
	import ShareButton from '$lib/features/collaboration/ui/ShareButton.svelte';
	import type { ConnectionState } from '$lib/shared/crdt/sync-bridge';

	interface Props {
		/** Whether the left widgets drawer is open (narrow viewports). */
		leftDrawerOpen: boolean;
		/** Whether the right inspector drawer is open (medium and narrower). */
		rightDrawerOpen: boolean;
		/** Collapse the docked left panel, or toggle its drawer when off-canvas. */
		onToggleLeftDock: () => void;
		/** Collapse the docked right panel, or toggle its drawer when off-canvas. */
		onToggleRightDock: () => void;
		/** Undo/redo availability + handlers (Loro UndoManager, this peer's edits). */
		canUndo: boolean;
		canRedo: boolean;
		onUndo: () => void;
		onRedo: () => void;
		/** Open the command palette (same target as the Cmd/Ctrl-K shortcut). */
		onOpenPalette: () => void;
		/** Everyone in this workspace, you first (shown as the avatar stack). */
		people?: RosterPerson[];
		/** The live relay connection state (shown as the sync pill). */
		syncState?: ConnectionState;
		/** Open the Share / collaborators panel (wires the stack + sync-pill click). */
		onOpenShare?: () => void;
		/** Whether the workspace is actively shared (drives the Share button state). */
		shareLive?: boolean;
	}

	let {
		leftDrawerOpen,
		rightDrawerOpen,
		onToggleLeftDock,
		onToggleRightDock,
		canUndo,
		canRedo,
		onUndo,
		onRedo,
		onOpenPalette,
		people = [],
		onOpenShare,
		syncState = 'syncing',
		shareLive = false
	}: Props = $props();
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
		<SyncPill state={syncState} onClick={onOpenShare} />
		<span class="chip" title={m['editor.titlebar.draft']()}>{m['editor.titlebar.draft']()}</span>
	</div>

	<!-- Centered command-palette trigger. Styled like a search field so the action is
	     self-evident for non-technical streamers, with a key hint that teaches the
	     shortcut. Opens the same palette as Cmd/Ctrl-K. -->
	<button
		class="command-bar"
		aria-label={m['editor.palette.search']()}
		aria-keyshortcuts="Control+K"
		onclick={onOpenPalette}
	>
		<Search size={15} />
		<span class="command-bar-label">{m['editor.palette.search']()}</span>
		<kbd class="command-bar-kbd">Ctrl K</kbd>
	</button>

	<div class="actions">
		{#if onOpenShare}
			<div class="cluster">
				<Roster {people} onOpen={onOpenShare} />
				<ShareButton live={shareLive} onClick={onOpenShare} />
			</div>
			<span class="divider-v"></span>
		{:else}
			<Roster {people} />
		{/if}

		<!-- Shown when the inspector is off-canvas (medium and narrower); opens its drawer. -->
		<button
			class="ghost toggle-right"
			aria-label={m['editor.drawer.toggle_inspector']()}
			aria-expanded={rightDrawerOpen}
			onclick={onToggleRightDock}
		>
			<PanelRight size={16} />
		</button>

		<button class="ghost" aria-label="Undo" disabled={!canUndo} onclick={onUndo}>
			<Undo2 size={16} />
		</button>
		<button class="ghost" aria-label="Redo" disabled={!canRedo} onclick={onRedo}>
			<Redo2 size={16} />
		</button>
		<!-- Use in OBS lands with the obs-websocket integration (deferred). -->
		<button class="primary" aria-label={m['editor.command.use_in_obs']()} disabled>
			<MonitorPlay size={15} />
			<span class="primary-label">{m['editor.command.use_in_obs']()}</span>
		</button>
	</div>
</header>

<style>
	.titlebar {
		grid-area: titlebar;
		/* Three zones (brand | command bar | actions). Equal 1fr sides keep the auto
		   center column at the true horizontal center of the bar regardless of how wide
		   either end is; a flex space-between would only center it in leftover space. */
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		align-items: center;
		column-gap: var(--space-3);
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
		justify-self: start;
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

	/* The centered command-palette trigger, dressed as a recessed search field. */
	.command-bar {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		justify-self: center;
		width: min(360px, 38vw);
		height: 28px;
		padding: 0 var(--space-2) 0 var(--space-3);
		border-radius: var(--radius-md);
		background: var(--muted);
		color: var(--muted-foreground);
		border: var(--stroke-thin) solid var(--border-subtle);
		font-size: var(--text-sm);
		cursor: pointer;
		transition:
			border-color var(--dur-fast) var(--ease-out),
			color var(--dur-fast) var(--ease-out);
	}

	.command-bar-label {
		flex: 1;
		min-width: 0;
		text-align: left;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.command-bar-kbd {
		flex: none;
		padding: 1px var(--space-1);
		border-radius: var(--radius-sm);
		background: var(--background);
		border: var(--stroke-thin) solid var(--border);
		color: var(--muted-foreground);
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		line-height: 1.4;
	}

	.actions {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		justify-self: end;
	}

	/* The presence cluster (avatar stack + Share), set off from the editor actions. */
	.cluster {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}

	.divider-v {
		width: 1px;
		height: 20px;
		background: var(--divider);
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

	/* The widgets panel goes off-canvas in the narrow zone: show its toggle. The
	   command bar drops its key hint here to reclaim width as the bar tightens. */
	@media (max-width: 959px) {
		.toggle-left {
			display: inline-flex;
		}

		.command-bar-kbd {
			display: none;
		}
	}

	/* Below the floor, drop the draft chip and OBS label to reclaim width, and collapse
	   the command bar to an icon-only button so it stops competing for the row. */
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

		.command-bar {
			width: 28px;
			padding: 0;
			justify-content: center;
		}

		.command-bar-label {
			display: none;
		}
	}

	@media (hover: hover) {
		.ghost:not(:disabled):hover {
			background: var(--accent);
			color: var(--accent-foreground);
		}

		.command-bar:hover {
			border-color: var(--border);
			color: var(--foreground);
		}
	}

	.ghost:not(:disabled):active,
	.command-bar:active {
		transform: scale(var(--press-scale));
	}

	.ghost:focus-visible,
	.primary:focus-visible,
	.command-bar:focus-visible {
		outline: none;
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
		.primary:not(:disabled),
		.command-bar {
			position: relative;
		}

		.ghost:not(:disabled)::after,
		.primary:not(:disabled)::after,
		.command-bar::after {
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
