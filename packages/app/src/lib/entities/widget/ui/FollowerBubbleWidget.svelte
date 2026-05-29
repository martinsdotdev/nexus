<script lang="ts">
	// Follower bubble: seeds with a friendly placeholder, then live-updates from
	// the latest follow alert. Styling is theme-variable only.
	import type { WidgetProps } from '../model/contract';

	let { instance, bus }: WidgetProps = $props();

	// Live override from the bus; falls back to the instance's seeded prop.
	let live = $state<{ user?: string }>({});
	const user = $derived(live.user ?? String(instance.props.user ?? 'someone'));

	$effect(() =>
		bus.subscribe('alert.follow', (event) => {
			if (event.kind !== 'alert.follow') return;
			live = { user: event.payload.user };
		})
	);
</script>

<div class="follower-bubble">
	<span class="user">{user}</span>
	<span class="label">just followed</span>
</div>

<style>
	.follower-bubble {
		box-sizing: border-box;
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 10px 20px;
		border-radius: 999px;
		background: var(--primary);
		color: var(--primary-foreground);
		box-shadow: var(--shadow-widget);
		font-family: var(--font-body);
		font-size: 18px;
		overflow: hidden;
	}
	.user {
		font-family: var(--font-display);
		font-weight: 700;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.label {
		color: var(--accent);
		white-space: nowrap;
	}
</style>
