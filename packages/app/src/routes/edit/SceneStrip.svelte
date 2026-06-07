<script lang="ts">
	// The scene selector + per-scene CRUD. Each scene is a toggle button (aria-pressed
	// drives the active state) so selection has real button semantics; a sibling Ark Menu
	// (kebab) carries rename / duplicate / move / delete without nesting a button inside
	// the toggle. Rename is inline (an input swaps in for the label, committing on blur or
	// Enter). Delete and the move items are hidden when they would break an invariant (a
	// layout keeps at least one scene; ends cannot move further). Editor chrome.
	import { Menu } from '@ark-ui/svelte/menu';
	import { Ellipsis, Plus, Pencil, Copy, Trash2, ChevronLeft, ChevronRight } from 'lucide-svelte';

	interface Scene {
		id: string;
		label: string;
	}

	interface Props {
		scenes: Scene[];
		activeSceneId: string;
		onSelect: (id: string) => void;
		onAdd: () => void;
		onRename: (id: string, name: string) => void;
		onDuplicate: (id: string) => void;
		onDelete: (id: string) => void;
		onReorder: (id: string, index: number) => void;
	}

	let {
		scenes,
		activeSceneId,
		onSelect,
		onAdd,
		onRename,
		onDuplicate,
		onDelete,
		onReorder
	}: Props = $props();

	// Inline rename state: the scene whose label is being edited, and the draft value.
	let editingId = $state<string | null>(null);
	let editingValue = $state('');

	function startRename(scene: Scene) {
		editingValue = scene.label;
		editingId = scene.id;
	}

	function commitRename() {
		const id = editingId;
		if (!id) return;
		editingId = null;
		const name = editingValue.trim();
		if (name) onRename(id, name);
	}

	// Focus the rename field once it mounts, so the user can type immediately.
	function focusSoon(node: HTMLInputElement) {
		const timer = setTimeout(() => {
			node.focus();
			node.select();
		});
		return { destroy: () => clearTimeout(timer) };
	}

	// Ark restores focus to the action-menu trigger when it closes, firing a synthetic
	// blur on the freshly-opened rename field (its relatedTarget is the trigger, or null
	// in headless). Keep editing in that case (re-focus); commit only when focus genuinely
	// moves to another control (clicking away) or on Enter.
	function onRenameBlur(event: FocusEvent) {
		const to = event.relatedTarget as HTMLElement | null;
		if (!to || to.closest('[data-scope="menu"][data-part="trigger"]')) {
			(event.currentTarget as HTMLInputElement).focus();
			return;
		}
		commitRename();
	}

	function onMenuSelect(scene: Scene, index: number, value: string) {
		if (value === 'rename') startRename(scene);
		else if (value === 'duplicate') onDuplicate(scene.id);
		else if (value === 'delete') onDelete(scene.id);
		else if (value === 'move-left') onReorder(scene.id, index - 1);
		else if (value === 'move-right') onReorder(scene.id, index + 1);
	}
</script>

<div class="scenestrip" role="group" aria-label="Scenes">
	{#each scenes as scene, index (scene.id)}
		<div class="scene-slot">
			{#if editingId === scene.id}
				<span class="scene-card editing">
					<span class="thumb" aria-hidden="true"></span>
					<input
						class="scene-rename"
						aria-label="Scene name"
						bind:value={editingValue}
						use:focusSoon
						onblur={onRenameBlur}
						onkeydown={(event) => {
							if (event.key === 'Enter') {
								event.preventDefault();
								commitRename();
							} else if (event.key === 'Escape') {
								event.preventDefault();
								editingId = null;
							}
						}}
					/>
				</span>
			{:else}
				<button
					class="scene-card"
					type="button"
					aria-pressed={scene.id === activeSceneId}
					title="Double-click to rename"
					onclick={() => onSelect(scene.id)}
					ondblclick={() => startRename(scene)}
				>
					<span class="thumb" aria-hidden="true"></span>
					<span class="scene-label">{scene.label}</span>
				</button>
			{/if}

			<Menu.Root
				onSelect={(details) => onMenuSelect(scene, index, details.value)}
				positioning={{ placement: 'top-end' }}
				lazyMount
				unmountOnExit
			>
				<Menu.Trigger class="scene-kebab" aria-label={`Actions for ${scene.label}`}>
					<Ellipsis size={13} />
				</Menu.Trigger>
				<Menu.Positioner>
					<Menu.Content>
						<Menu.Item value="rename">
							<Pencil size={13} /><span class="item-label">Rename</span>
						</Menu.Item>
						<Menu.Item value="duplicate">
							<Copy size={13} /><span class="item-label">Duplicate</span>
						</Menu.Item>
						{#if index > 0}
							<Menu.Item value="move-left">
								<ChevronLeft size={13} /><span class="item-label">Move left</span>
							</Menu.Item>
						{/if}
						{#if index < scenes.length - 1}
							<Menu.Item value="move-right">
								<ChevronRight size={13} /><span class="item-label">Move right</span>
							</Menu.Item>
						{/if}
						{#if scenes.length > 1}
							<Menu.Separator />
							<Menu.Item value="delete" class="danger">
								<Trash2 size={13} /><span class="item-label">Delete</span>
							</Menu.Item>
						{/if}
					</Menu.Content>
				</Menu.Positioner>
			</Menu.Root>
		</div>
	{/each}

	<button class="scene-add" type="button" onclick={onAdd} aria-label="Add scene">
		<Plus size={16} />
	</button>
</div>

<style>
	.scenestrip {
		grid-area: scenestrip;
		display: flex;
		align-items: center;
		gap: var(--space-3);
		height: var(--scenestrip-height);
		padding: 0 var(--space-3);
		background: var(--titlebar);
		border-top: var(--stroke-thin) solid var(--divider);
		overflow-x: auto;
		overflow-y: hidden;
		scroll-snap-type: x proximity;
	}

	.scene-slot {
		position: relative;
		flex: none;
		scroll-snap-align: start;
	}

	.scene-card {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		padding: var(--space-1);
		border-radius: var(--radius-md);
		border: var(--stroke-thicker) solid transparent;
		background: none;
		color: inherit;
		font: inherit;
		cursor: pointer;
		transition: border-color var(--dur-fast) var(--ease-out);
	}

	.scene-card[aria-pressed='true'] {
		border-color: var(--ring);
		background: var(--accent);
	}

	.scene-card:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	.thumb {
		height: 54px;
		aspect-ratio: 16 / 9;
		border-radius: var(--radius-sm);
		background: var(--background);
		border: var(--stroke-thin) solid var(--border-subtle);
	}

	.scene-label {
		max-width: 96px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		text-align: center;
	}

	.scene-card[aria-pressed='true'] .scene-label {
		color: var(--foreground);
	}

	.scene-rename {
		width: 96px;
		padding: 2px var(--space-1);
		border-radius: var(--radius-sm);
		border: var(--stroke-thin) solid var(--border);
		background: var(--background);
		color: var(--foreground);
		font-size: var(--text-xs);
		text-align: center;
	}

	.scene-rename:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	/* The kebab is an Ark Menu.Trigger (a component, not a DOM node Svelte can scope),
	   so its class is styled globally, like LayoutSwitcher's trigger. Always present (so
	   it is reachable) but subtle until the slot is hovered/focused. */
	:global(.scene-kebab) {
		position: absolute;
		top: 2px;
		right: 2px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border-radius: var(--radius-sm);
		background: var(--titlebar);
		color: var(--muted-foreground);
		opacity: 0.55;
		cursor: pointer;
		transition:
			opacity var(--dur-fast) var(--ease-out),
			background var(--dur-fast) var(--ease-out);
	}

	@media (hover: hover) {
		:global(.scene-kebab) {
			opacity: 0;
		}

		.scene-slot:hover :global(.scene-kebab) {
			opacity: 1;
		}
	}

	.scene-slot:focus-within :global(.scene-kebab) {
		opacity: 1;
	}

	:global(.scene-kebab:focus-visible) {
		opacity: 1;
		outline: none;
		box-shadow: var(--focus-ring);
	}

	.scene-add {
		flex: none;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 54px;
		border-radius: var(--radius-md);
		border: var(--stroke-thin) dashed var(--border);
		background: none;
		color: var(--muted-foreground);
		cursor: pointer;
		transition:
			color var(--dur-fast) var(--ease-out),
			border-color var(--dur-fast) var(--ease-out);
	}

	.scene-add:hover {
		color: var(--foreground);
		border-color: var(--foreground);
	}

	.scene-add:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	/* Ark renders the menu portaled (outside this component's scope), so its parts are global. */
	:global([data-scope='menu'][data-part='content']) {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 10rem;
		padding: var(--space-1);
		background: var(--popover);
		color: var(--popover-foreground);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-popover);
		z-index: 100;
	}

	:global([data-scope='menu'][data-part='item']) {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-sm);
		font-size: var(--text-sm);
		color: var(--foreground);
		cursor: pointer;
	}

	:global([data-scope='menu'][data-part='item'][data-highlighted]) {
		background: var(--accent);
		color: var(--accent-foreground);
	}

	:global([data-scope='menu'][data-part='item'].danger) {
		color: var(--destructive, var(--foreground));
	}

	:global([data-scope='menu'][data-part='separator']) {
		height: 1px;
		margin: var(--space-1) 0;
		background: var(--border-subtle);
	}

	.item-label {
		flex: 1;
	}
</style>
