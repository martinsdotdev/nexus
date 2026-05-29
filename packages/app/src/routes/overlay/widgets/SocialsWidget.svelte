<script lang="ts">
	// Static widget: renders its seeded `handles` prop as a compact strip of muted
	// handle chips. No event subscription. Styling is theme-variable only.
	import type { WidgetProps } from './contract';

	let { instance }: WidgetProps = $props();

	const handles = $derived(
		String(instance.props.handles ?? '')
			.split(',')
			.map((handle) => handle.trim())
			.filter((handle) => handle.length > 0)
	);
</script>

<div class="socials">
	{#each handles as handle (handle)}
		<span class="chip">{handle}</span>
	{/each}
</div>

<style>
	.socials {
		box-sizing: border-box;
		width: 100%;
		height: 100%;
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-radius: var(--radius);
		background: var(--card);
		color: var(--card-foreground);
		box-shadow: var(--shadow-widget);
		font-family: var(--font-body);
		overflow: hidden;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		padding: 4px 10px;
		border-radius: 999px;
		background: var(--muted);
		color: var(--muted-foreground);
		border: 1px solid var(--border);
		font-size: 14px;
		font-weight: 500;
		line-height: 1.2;
		white-space: nowrap;
	}
</style>
