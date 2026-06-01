<script lang="ts">
	// Canonical widget: seeds from props, then live-updates from its event. The
	// other widgets follow this shape. Styling is theme-variable only.
	import type { WidgetProps } from '../model/contract';

	let { instance, bus }: WidgetProps = $props();

	// Live overrides from the bus; fall back to the instance's seeded props.
	let live = $state<{ title?: string; game?: string; viewers?: number }>({});
	const title = $derived(live.title ?? String(instance.props.title ?? 'Untitled Stream'));
	const game = $derived(live.game ?? String(instance.props.game ?? ''));
	const viewers = $derived(live.viewers ?? Number(instance.props.viewers ?? 0));

	$effect(() =>
		bus.subscribe('stream.info-changed', (event) => {
			if (event.kind !== 'stream.info-changed') return;
			live = {
				title: event.payload.title,
				game: event.payload.game,
				viewers: event.payload.viewerCount
			};
		})
	);
</script>

<div class="stream-info">
	<div class="title">{title}</div>
	<div class="meta">
		{#if game}<span>{game}</span>{/if}
		{#if viewers}<span class="viewers">{viewers} watching</span>{/if}
	</div>
</div>

<style>
	.stream-info {
		box-sizing: border-box;
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 4px;
		padding: 12px 18px;
		border-radius: var(--radius);
		background: var(--card);
		color: var(--card-foreground);
		box-shadow: var(--shadow-widget);
		font-family: var(--font-body);
		overflow: hidden;
	}
	.title {
		font-family: var(--font-display);
		font-size: 28px;
		font-weight: 700;
		line-height: 1.1;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.meta {
		display: flex;
		gap: 12px;
		font-size: 16px;
		color: var(--muted-foreground);
	}
	.viewers {
		color: var(--primary);
		/* Fixed-width digits so the live viewer count does not jitter as it updates. */
		font-variant-numeric: tabular-nums;
	}
</style>
