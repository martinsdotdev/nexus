<script lang="ts">
	// Alerts: queues incoming alert.* events and shows one at a time (FIFO) for a
	// fixed duration before advancing. Styling is theme-variable only.
	import type { WidgetProps } from './contract';
	import type {
		FollowAlert,
		SubscribeAlert,
		CheerAlert,
		RaidAlert,
		DonationAlert
	} from '$lib/shared/events/events';
	import { fly } from 'svelte/transition';

	let { instance, bus }: WidgetProps = $props();

	type Alert =
		| { kind: 'alert.follow'; payload: FollowAlert }
		| { kind: 'alert.subscribe'; payload: SubscribeAlert }
		| { kind: 'alert.cheer'; payload: CheerAlert }
		| { kind: 'alert.raid'; payload: RaidAlert }
		| { kind: 'alert.donation'; payload: DonationAlert };

	// FIFO queue of pending alerts; the first entry is the one on screen.
	let queue = $state<Alert[]>([]);
	const durationMs = $derived(Number(instance.props.durationMs ?? 4000));
	const current = $derived(queue[0]);
	const text = $derived(current ? format(current) : '');

	function format(alert: Alert): string {
		switch (alert.kind) {
			case 'alert.follow':
				return `${alert.payload.user} followed`;
			case 'alert.subscribe':
				return `${alert.payload.user} subscribed (tier ${alert.payload.tier}, ${alert.payload.months}mo)`;
			case 'alert.cheer':
				return `${alert.payload.user} cheered ${alert.payload.bits} bits`;
			case 'alert.raid':
				return `${alert.payload.fromChannel} raided with ${alert.payload.viewers}`;
			case 'alert.donation':
				return `${alert.payload.user} donated ${alert.payload.amount} ${alert.payload.currency}`;
		}
	}

	$effect(() =>
		bus.subscribe('alert.*', (event) => {
			if (
				event.kind !== 'alert.follow' &&
				event.kind !== 'alert.subscribe' &&
				event.kind !== 'alert.cheer' &&
				event.kind !== 'alert.raid' &&
				event.kind !== 'alert.donation'
			)
				return;
			queue = [...queue, event];
		})
	);

	// Arm the dwell timer off the head item's identity, not the queue length: a
	// new alert appended to the tail leaves queue[0] (current) referentially
	// unchanged, so this effect does not re-run and the visible alert keeps its
	// countdown. Only an advance (or first arrival) changes the head and re-arms.
	$effect(() => {
		if (!current) return;
		const timer = setTimeout(() => {
			queue = queue.slice(1);
		}, durationMs);
		return () => clearTimeout(timer);
	});
</script>

<div class="alerts">
	{#if current}
		{#key current}
			<div class="alert" in:fly={{ y: 24, duration: 300 }}>{text}</div>
		{/key}
	{/if}
</div>

<style>
	.alerts {
		box-sizing: border-box;
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 16px 24px;
		overflow: hidden;
	}
	.alert {
		font-family: var(--font-display);
		font-size: 32px;
		font-weight: 700;
		line-height: 1.15;
		text-align: center;
		color: var(--primary);
		text-shadow: var(--shadow-widget);
		max-width: 100%;
		overflow: hidden;
		text-overflow: ellipsis;
	}
</style>
