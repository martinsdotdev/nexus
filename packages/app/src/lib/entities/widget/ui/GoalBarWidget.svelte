<script lang="ts">
	// Goal bar: seeds the current value from props, then live-updates the total
	// from goal.increment events. Styling is theme-variable only.
	import type { WidgetProps } from '../model/contract';

	let { instance, bus }: WidgetProps = $props();

	// Live overrides from the bus; fall back to the instance's seeded props. Both
	// paths route through Number() so a non-numeric total can never reach the CSS.
	let live = $state<{ current?: number }>({});
	const label = $derived(String(instance.props.label ?? 'Goal'));
	const current = $derived(Number(live.current ?? instance.props.current ?? 0));
	const target = $derived(Number(instance.props.target ?? 100));
	const fraction = $derived(
		target > 0 && Number.isFinite(current) ? Math.min(Math.max(current / target, 0), 1) : 0
	);

	$effect(() =>
		bus.subscribe('goal.increment', (event) => {
			if (event.kind !== 'goal.increment') return;
			live = { current: event.payload.total };
		})
	);
</script>

<div class="goal-bar">
	<div class="header">
		<span class="label">{label}</span>
		<span class="count">{current} / {target}</span>
	</div>
	<div class="track">
		<div class="fill" style:width="{fraction * 100}%"></div>
	</div>
</div>

<style>
	.goal-bar {
		box-sizing: border-box;
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 8px;
		padding: 12px 18px;
		border-radius: var(--radius);
		background: var(--card);
		color: var(--card-foreground);
		box-shadow: var(--shadow-widget);
		font-family: var(--font-body);
		overflow: hidden;
	}
	.header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}
	.label {
		font-family: var(--font-display);
		font-size: 18px;
		font-weight: 700;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.count {
		font-size: 14px;
		color: var(--muted-foreground);
		white-space: nowrap;
	}
	.track {
		width: 100%;
		height: 12px;
		border-radius: var(--radius);
		background: var(--muted);
		overflow: hidden;
	}
	.fill {
		height: 100%;
		border-radius: var(--radius);
		background: var(--primary);
	}
</style>
