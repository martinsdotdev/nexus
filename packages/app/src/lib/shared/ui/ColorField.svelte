<script lang="ts">
	// A generic visual color input backed by Ark UI's Color Picker: a swatch trigger
	// opening a saturation/brightness area, a hue slider, and a hex field. Stays in
	// sRGB and CSS-color terms (no oklch; the theme feature bridges that, keeping
	// shared/ FSD-clean). Dragging updates the UI live via local state; the doc write
	// fires once on release (onValueChangeEnd = one undo step). Editor chrome.
	import { ColorPicker, parseColor } from '@ark-ui/svelte/color-picker';

	interface Props {
		/** Any CSS color the picker can parse (the theme feeds it hex). */
		value: string;
		/** Emits the chosen color as an rgba() string on release. */
		onChange: (value: string) => void;
		ariaLabel?: string;
	}
	let { value, onChange, ariaLabel }: Props = $props();

	// Normally the picker shows the prop; while dragging, it shows the in-flight
	// color held locally so the UI is live without writing every frame. We commit
	// once on release (one undo step) and drop back to the prop, which by then
	// reflects the committed value (and any external change, e.g. a theme switch).
	let dragValue = $state<string | null>(null);
	const color = $derived(parseColor(dragValue ?? value));
</script>

<ColorPicker.Root
	value={color}
	format="rgba"
	onValueChange={(details) => (dragValue = details.valueAsString)}
	onValueChangeEnd={(details) => {
		onChange(details.valueAsString);
		dragValue = null;
	}}
	positioning={{ placement: 'bottom-start' }}
	lazyMount
	unmountOnExit
>
	<ColorPicker.Control>
		<ColorPicker.Trigger aria-label={ariaLabel ?? 'Edit color'}>
			<ColorPicker.ValueSwatch />
		</ColorPicker.Trigger>
	</ColorPicker.Control>
	<ColorPicker.Positioner>
		<ColorPicker.Content>
			<ColorPicker.Area>
				<ColorPicker.AreaBackground />
				<ColorPicker.AreaThumb />
			</ColorPicker.Area>
			<ColorPicker.ChannelSlider channel="hue">
				<ColorPicker.ChannelSliderTrack />
				<ColorPicker.ChannelSliderThumb />
			</ColorPicker.ChannelSlider>
			<ColorPicker.ChannelInput channel="hex" aria-label="Hex" />
		</ColorPicker.Content>
	</ColorPicker.Positioner>
	<ColorPicker.HiddenInput />
</ColorPicker.Root>

<style>
	:global([data-scope='color-picker'][data-part='trigger']) {
		display: inline-flex;
		width: 16px;
		height: 16px;
		flex: none;
		padding: 0;
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-sm);
		cursor: pointer;
		overflow: hidden;
	}

	:global([data-scope='color-picker'][data-part='trigger']:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	:global([data-scope='color-picker'][data-part='value-swatch']) {
		width: 100%;
		height: 100%;
	}

	:global([data-scope='color-picker'][data-part='content']) {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		width: 200px;
		padding: var(--space-2);
		background: var(--popover);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-md);
		box-shadow: var(--shadow-popover);
		z-index: 200;
	}

	:global([data-scope='color-picker'][data-part='area']) {
		position: relative;
		height: 120px;
		border-radius: var(--radius-sm);
		overflow: hidden;
	}

	:global([data-scope='color-picker'][data-part='area-background']) {
		position: absolute;
		inset: 0;
	}

	:global([data-scope='color-picker'][data-part='area-thumb']) {
		width: 12px;
		height: 12px;
		border-radius: var(--radius-circular);
		border: 2px solid #fff;
		box-shadow: 0 0 0 1px oklch(0% 0 0 / 0.4);
	}

	:global([data-scope='color-picker'][data-part='channel-slider']) {
		position: relative;
		height: 12px;
		border-radius: var(--radius-circular);
	}

	:global([data-scope='color-picker'][data-part='channel-slider-track']) {
		height: 100%;
		border-radius: var(--radius-circular);
	}

	:global([data-scope='color-picker'][data-part='channel-slider-thumb']) {
		width: 14px;
		height: 14px;
		border-radius: var(--radius-circular);
		border: 2px solid #fff;
		box-shadow: 0 0 0 1px oklch(0% 0 0 / 0.4);
	}

	:global([data-scope='color-picker'][data-part='channel-input']) {
		width: 100%;
		padding: var(--space-1) var(--space-2);
		font: inherit;
		font-size: var(--text-xs);
		font-family: var(--font-mono);
		color: var(--foreground);
		background: var(--input);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-sm);
	}

	:global([data-scope='color-picker'][data-part='channel-input']:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
	}
</style>
