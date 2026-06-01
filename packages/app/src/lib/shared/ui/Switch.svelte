<script lang="ts">
	// A labelled on/off toggle backed by Ark UI's Switch (role="switch", keyboard,
	// a hidden native input for forms). Plain {checked, onChange(checked), label}
	// API. Editor chrome; the track + thumb are styled with tokens.
	import { Switch } from '@ark-ui/svelte/switch';

	interface Props {
		checked: boolean;
		onChange: (checked: boolean) => void;
		label: string;
	}
	let { checked, onChange, label }: Props = $props();
</script>

<Switch.Root {checked} onCheckedChange={(details) => onChange(details.checked)}>
	<Switch.Control>
		<Switch.Thumb />
	</Switch.Control>
	<Switch.Label>{label}</Switch.Label>
	<Switch.HiddenInput />
</Switch.Root>

<style>
	:global([data-scope='switch'][data-part='root']) {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		cursor: pointer;
		font-size: var(--text-sm);
		color: var(--foreground);
	}

	:global([data-scope='switch'][data-part='control']) {
		box-sizing: border-box;
		display: inline-flex;
		align-items: center;
		width: 32px;
		height: 18px;
		padding: 2px;
		border-radius: var(--radius-circular);
		background: var(--muted);
		border: var(--stroke-thin) solid var(--border);
		transition: background var(--dur-fast) var(--ease-out);
	}

	:global([data-scope='switch'][data-part='control'][data-state='checked']) {
		background: var(--primary);
		border-color: transparent;
	}

	:global([data-scope='switch'][data-part='control']:focus-visible) {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	:global([data-scope='switch'][data-part='thumb']) {
		width: 12px;
		height: 12px;
		border-radius: var(--radius-circular);
		background: var(--background);
		transition: transform var(--dur-fast) var(--ease-out);
	}

	:global([data-scope='switch'][data-part='thumb'][data-state='checked']) {
		transform: translateX(14px);
	}
</style>
