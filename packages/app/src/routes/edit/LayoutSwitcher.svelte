<script module lang="ts">
	export interface LayoutNav {
		/** Active (non-archived) layouts, in document order. */
		layouts: { id: string; name: string }[];
		activeLayoutId: string;
		onSwitch: (id: string) => void;
		onCreate: () => void;
		onDuplicate: () => void;
		onRename: (id: string, name: string) => void;
		onArchive: () => void;
		onDelete: () => void;
		/** Controlled open, so the open-layout command (Ctrl-O) can pop the menu. */
		open: boolean;
		onOpenChange: (open: boolean) => void;
	}
</script>

<script lang="ts">
	// The titlebar layout switcher: shows the active layout's name and, on click or the
	// open-layout command (Ctrl-O), a menu to switch between active layouts or create /
	// duplicate / rename / archive / delete one. Rename is inline (the trigger name swaps
	// for an input). Ark Menu (real menu ARIA + roving keyboard nav). Editor chrome.
	import { Menu } from '@ark-ui/svelte/menu';
	import { ChevronDown, Check, Plus, Copy, Pencil, Archive, Trash2 } from 'lucide-svelte';

	let {
		layouts,
		activeLayoutId,
		onSwitch,
		onCreate,
		onDuplicate,
		onRename,
		onArchive,
		onDelete,
		open,
		onOpenChange
	}: LayoutNav = $props();

	const activeName = $derived(layouts.find((l) => l.id === activeLayoutId)?.name ?? 'Layout');

	// Inline rename of the active layout: the trigger name swaps for an input.
	let editing = $state(false);
	let editValue = $state('');

	function startRename() {
		editValue = activeName;
		editing = true;
	}

	function commitRename() {
		if (!editing) return;
		editing = false;
		const name = editValue.trim();
		if (name) onRename(activeLayoutId, name);
	}

	// Focus the rename field once it mounts, so the user can type immediately.
	function focusSoon(node: HTMLInputElement) {
		const timer = setTimeout(() => {
			node.focus();
			node.select();
		});
		return { destroy: () => clearTimeout(timer) };
	}

	// Ark restores focus to the trigger when the menu closes, firing a synthetic blur on
	// the freshly-opened rename field (relatedTarget is the trigger, or null in headless).
	// Keep editing in that case (re-focus); commit only on a genuine blur or Enter.
	function onRenameBlur(event: FocusEvent) {
		const to = event.relatedTarget as HTMLElement | null;
		if (!to || to.closest('[data-scope="menu"][data-part="trigger"]')) {
			(event.currentTarget as HTMLInputElement).focus();
			return;
		}
		commitRename();
	}

	function onSelect(value: string) {
		if (value === '__new') onCreate();
		else if (value === '__duplicate') onDuplicate();
		else if (value === '__rename') startRename();
		else if (value === '__archive') onArchive();
		else if (value === '__delete') onDelete();
		else onSwitch(value);
	}
</script>

{#if editing}
	<input
		class="layout-rename"
		aria-label="Layout name"
		bind:value={editValue}
		use:focusSoon
		onblur={onRenameBlur}
		onkeydown={(event) => {
			if (event.key === 'Enter') {
				event.preventDefault();
				commitRename();
			} else if (event.key === 'Escape') {
				event.preventDefault();
				editing = false;
			}
		}}
	/>
{:else}
	<Menu.Root
		{open}
		onOpenChange={(details) => onOpenChange(details.open)}
		onSelect={(details) => onSelect(details.value)}
		positioning={{ placement: 'bottom-start' }}
		lazyMount
		unmountOnExit
	>
		<Menu.Trigger class="layout-trigger" aria-label="Switch layout">
			<span class="layout-name">{activeName}</span>
			<ChevronDown size={13} />
		</Menu.Trigger>
		<Menu.Positioner>
			<Menu.Content>
				{#each layouts as layout (layout.id)}
					<Menu.Item value={layout.id}>
						<span class="check"
							>{#if layout.id === activeLayoutId}<Check size={13} />{/if}</span
						>
						<span class="item-label">{layout.name}</span>
					</Menu.Item>
				{/each}
				<Menu.Separator />
				<Menu.Item value="__new"
					><Plus size={13} /><span class="item-label">New layout</span></Menu.Item
				>
				<Menu.Item value="__duplicate">
					<Copy size={13} /><span class="item-label">Duplicate</span>
				</Menu.Item>
				<Menu.Item value="__rename">
					<Pencil size={13} /><span class="item-label">Rename</span>
				</Menu.Item>
				{#if layouts.length > 1}
					<Menu.Item value="__archive">
						<Archive size={13} /><span class="item-label">Archive</span>
					</Menu.Item>
					<Menu.Separator />
					<Menu.Item value="__delete" class="danger">
						<Trash2 size={13} /><span class="item-label">Delete</span>
					</Menu.Item>
				{/if}
			</Menu.Content>
		</Menu.Positioner>
	</Menu.Root>
{/if}

<style>
	:global(.layout-trigger) {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		height: 24px;
		max-width: 170px;
		padding: 0 var(--space-1) 0 var(--space-2);
		border-radius: var(--radius-md);
		color: var(--foreground);
		font-size: var(--text-sm);
		font-weight: 600;
		cursor: pointer;
		transition: background var(--dur-fast) var(--ease-out);
	}

	@media (hover: hover) {
		:global(.layout-trigger:hover) {
			background: var(--accent);
		}
	}

	:global(.layout-trigger:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	.layout-name {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.layout-rename {
		height: 24px;
		max-width: 170px;
		padding: 0 var(--space-2);
		border-radius: var(--radius-md);
		border: var(--stroke-thin) solid var(--border);
		background: var(--background);
		color: var(--foreground);
		font-size: var(--text-sm);
		font-weight: 600;
	}

	.layout-rename:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	.check {
		display: inline-flex;
		width: 13px;
		flex: none;
		color: var(--primary);
	}

	.item-label {
		flex: 1;
	}

	/* Ark renders the menu in its own scope (portaled), so its parts are styled globally. */
	:global([data-scope='menu'][data-part='content']) {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 11rem;
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
</style>
