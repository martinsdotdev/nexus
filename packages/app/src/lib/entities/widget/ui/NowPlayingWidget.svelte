<script lang="ts">
	// Now-playing widget: seeds from props, then live-updates from media.track-changed.
	// Styling is theme-variable only.
	import type { WidgetProps } from '../model/contract';

	let { instance, bus }: WidgetProps = $props();

	// Live overrides from the bus; fall back to the instance's seeded props.
	let live = $state<{ title?: string; artist?: string }>({});
	const title = $derived(live.title ?? String(instance.props.track ?? 'Nothing playing'));
	const artist = $derived(live.artist ?? String(instance.props.artist ?? ''));

	$effect(() =>
		bus.subscribe('media.track-changed', (event) => {
			if (event.kind !== 'media.track-changed') return;
			live = {
				title: event.payload.title,
				artist: event.payload.artist
			};
		})
	);
</script>

<div class="now-playing">
	<div class="equalizer" aria-hidden="true">
		<span class="bar"></span>
		<span class="bar"></span>
		<span class="bar"></span>
		<span class="bar"></span>
	</div>
	<div class="track">
		<div class="title">{title}</div>
		{#if artist}<div class="artist">{artist}</div>{/if}
	</div>
</div>

<style>
	.now-playing {
		box-sizing: border-box;
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 12px 18px;
		border-radius: var(--radius);
		background: var(--card);
		color: var(--card-foreground);
		box-shadow: var(--shadow-widget);
		font-family: var(--font-body);
		overflow: hidden;
	}
	.equalizer {
		display: flex;
		align-items: flex-end;
		gap: 3px;
		height: 28px;
		flex-shrink: 0;
	}
	.bar {
		width: 4px;
		height: 100%;
		border-radius: 2px;
		background: var(--primary);
		transform-origin: bottom;
		animation: equalize 900ms ease-in-out infinite;
	}
	.bar:nth-child(1) {
		animation-delay: 0ms;
	}
	.bar:nth-child(2) {
		animation-delay: 150ms;
	}
	.bar:nth-child(3) {
		animation-delay: 300ms;
	}
	.bar:nth-child(4) {
		animation-delay: 450ms;
	}
	.track {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.title {
		font-family: var(--font-display);
		font-size: 20px;
		font-weight: 700;
		line-height: 1.15;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.artist {
		font-size: 14px;
		color: var(--muted-foreground);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	@keyframes equalize {
		0%,
		100% {
			transform: scaleY(0.3);
		}
		50% {
			transform: scaleY(1);
		}
	}
</style>
