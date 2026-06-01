<script lang="ts">
	// The scene selector: an Ark UI ToggleGroup (single-select, roving focus, real
	// group semantics) so the active scene is chosen with proper keyboard navigation
	// across the scrollable strip. Controlled by activeSceneId; a deselect (clicking
	// the active scene) is ignored so a scene is always active.
	import { ToggleGroup } from '@ark-ui/svelte/toggle-group';

	interface Scene {
		id: string;
		label: string;
	}

	interface Props {
		scenes: Scene[];
		activeSceneId: string;
		onSelect: (id: string) => void;
	}

	let { scenes, activeSceneId, onSelect }: Props = $props();
</script>

<ToggleGroup.Root
	class="scenestrip"
	aria-label="Scenes"
	value={[activeSceneId]}
	onValueChange={(details) => {
		if (details.value[0]) onSelect(details.value[0]);
	}}
>
	{#each scenes as scene (scene.id)}
		<ToggleGroup.Item value={scene.id} class="scene-card">
			<span class="thumb" aria-hidden="true"></span>
			<span class="scene-label">{scene.label}</span>
		</ToggleGroup.Item>
	{/each}
</ToggleGroup.Root>

<style>
	/* The ToggleGroup root + items are Ark elements (outside this CSS scope). */
	:global(.scenestrip) {
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

	:global(.scene-card) {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		padding: var(--space-1);
		border-radius: var(--radius-md);
		border: var(--stroke-thicker) solid transparent;
		background: none;
		cursor: pointer;
		transition: border-color var(--dur-fast) var(--ease-out);
		scroll-snap-align: start;
	}

	:global(.scene-card[data-state='on']) {
		border-color: var(--ring);
		background: var(--accent);
	}

	:global(.scene-card:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	:global(.scene-card .thumb) {
		height: 54px;
		aspect-ratio: 16 / 9;
		border-radius: var(--radius-sm);
		background: var(--background);
		border: var(--stroke-thin) solid var(--border-subtle);
	}

	:global(.scene-card .scene-label) {
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		text-align: center;
	}

	:global(.scene-card[data-state='on'] .scene-label) {
		color: var(--foreground);
	}

	@media (hover: hover) {
		:global(.scene-card:hover .thumb) {
			border-color: var(--border);
		}
	}

	@media (max-width: 639px) {
		:global(.scene-card .thumb) {
			height: 36px;
		}
	}
</style>
