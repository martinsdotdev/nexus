<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { MousePointer2, Move, Type, Square, Image } from 'lucide-svelte';
	import { IconButton } from '$lib/shared/ui';

	interface Props {
		activeToolId: string;
		onSelect: (id: string) => void;
	}

	let { activeToolId, onSelect }: Props = $props();

	const tools = [
		{ id: 'select', label: m['editor.tool.select'](), icon: MousePointer2 },
		{ id: 'move', label: m['editor.tool.move'](), icon: Move },
		{ id: 'text', label: m['editor.tool.text'](), icon: Type },
		{ id: 'shape', label: m['editor.tool.shape'](), icon: Square },
		{ id: 'media', label: m['editor.tool.media'](), icon: Image }
	];
</script>

<nav class="toolrail" aria-label="Tools">
	{#each tools as tool (tool.id)}
		{@const Icon = tool.icon}
		<IconButton
			label={tool.label}
			active={tool.id === activeToolId}
			onclick={() => onSelect(tool.id)}
		>
			<Icon size={18} strokeWidth={1.75} />
		</IconButton>
	{/each}
</nav>

<style>
	.toolrail {
		grid-area: toolrail;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-1);
		width: var(--toolrail-width);
		padding: var(--space-2) 0;
		background: var(--toolrail);
		border-right: var(--stroke-thin) solid var(--divider);
	}

	/* Touch: widen the gap so adjacent 44px tap targets do not overlap. */
	@media (pointer: coarse) {
		.toolrail {
			gap: var(--space-3);
		}
	}

	/* Zone D (best-effort, <= 639px): the rail lies down as a horizontal bar above the
	   canvas. Five tools fit within 360px; overflow-x is only a safety net. */
	@media (max-width: 639px) {
		.toolrail {
			flex-direction: row;
			justify-content: center;
			width: 100%;
			padding: var(--space-1) var(--space-2);
			overflow-x: auto;
			border-right: none;
			border-bottom: var(--stroke-thin) solid var(--divider);
		}
	}
</style>
