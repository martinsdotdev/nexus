<script lang="ts">
	// Decorative webcam frame: a thick border around a transparent center so OBS
	// composites the real webcam underneath. No event subscription; config only.
	// Styling is theme-variable only.
	import type { WidgetProps } from './contract';

	let { instance }: WidgetProps = $props();

	const shape = $derived(String(instance.props.shape ?? 'squircle'));
</script>

<div class="webcam-frame" class:circle={shape === 'circle'} class:rounded={shape === 'rounded'}>
	<span class="label">CAM</span>
</div>

<style>
	.webcam-frame {
		box-sizing: border-box;
		position: relative;
		width: 100%;
		height: 100%;
		background: transparent;
		border: 6px solid var(--primary);
		border-radius: var(--radius);
		box-shadow: var(--shadow-widget);
		overflow: hidden;
	}
	.webcam-frame.rounded {
		border-radius: 16px;
	}
	.webcam-frame.circle {
		border-radius: 50%;
	}
	.label {
		position: absolute;
		bottom: 8px;
		left: 8px;
		padding: 2px 8px;
		border-radius: var(--radius);
		background: var(--primary);
		color: var(--primary-foreground);
		font-family: var(--font-display);
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0.08em;
		line-height: 1;
	}
</style>
