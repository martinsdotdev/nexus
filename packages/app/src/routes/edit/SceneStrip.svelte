<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	interface Props {
		activeSceneId: string;
		onSelect: (id: string) => void;
	}

	let { activeSceneId, onSelect }: Props = $props();

	// Maps to Nexus scenes (the Resolve-style page strip).
	const scenes = [
		{ id: 'live', label: m['editor.scene.live']() },
		{ id: 'starting_soon', label: m['editor.scene.starting_soon']() },
		{ id: 'brb', label: m['editor.scene.brb']() },
		{ id: 'ending', label: m['editor.scene.ending']() }
	];
</script>

<footer class="scenestrip" aria-label="Scenes">
	{#each scenes as scene (scene.id)}
		<button
			class="scene-card"
			class:active={scene.id === activeSceneId}
			aria-pressed={scene.id === activeSceneId}
			onclick={() => onSelect(scene.id)}
		>
			<span class="thumb" aria-hidden="true"></span>
			<span class="scene-label">{scene.label}</span>
		</button>
	{/each}
</footer>

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

	.scene-card {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		padding: var(--space-1);
		border-radius: var(--radius-md);
		border: var(--stroke-thicker) solid transparent;
		transition: border-color var(--dur-fast) var(--ease-out);
		scroll-snap-align: start;
	}

	.scene-card.active {
		border-color: var(--ring);
		background: var(--accent);
	}

	.scene-card:focus-visible {
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
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		text-align: center;
	}

	.scene-card.active .scene-label {
		color: var(--foreground);
	}

	@media (hover: hover) {
		.scene-card:hover .thumb {
			border-color: var(--border);
		}
	}

	/* Below the floor the strip thins to 64px; shrink the thumbnail to match. */
	@media (max-width: 639px) {
		.thumb {
			height: 36px;
		}
	}
</style>
