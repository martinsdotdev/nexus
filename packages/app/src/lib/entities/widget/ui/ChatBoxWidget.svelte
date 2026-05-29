<script lang="ts">
	// Chat box: appends incoming chat.message events to a capped live list and
	// renders newest at the bottom. Styling is theme-variable only.
	import type { WidgetProps } from '../model/contract';
	import type { ChatMessage } from '$lib/shared/events/events';

	let { instance, bus }: WidgetProps = $props();

	const MAX_MESSAGES = 8;

	// Each message carries a monotonic seq so the keyed each block has a
	// collision-free key: chat timestamps are display values and can repeat
	// within a millisecond (bursts), which would crash a timestamp-keyed each.
	type Entry = { seq: number; message: ChatMessage };
	let entries = $state<Entry[]>([]);
	let seq = 0;
	const title = $derived(String(instance.props.title ?? 'Chat'));

	$effect(() =>
		bus.subscribe('chat.message', (event) => {
			if (event.kind !== 'chat.message') return;
			entries = [...entries, { seq: seq++, message: event.payload }].slice(-MAX_MESSAGES);
		})
	);
</script>

<div class="chat-box">
	<div class="header">{title}</div>
	<div class="messages">
		{#each entries as entry (entry.seq)}
			<div class="message">
				<span class="user">{entry.message.user}</span>
				<span class="text">{entry.message.text}</span>
			</div>
		{/each}
	</div>
</div>

<style>
	.chat-box {
		box-sizing: border-box;
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px 16px;
		border-radius: var(--radius);
		background: var(--card);
		color: var(--card-foreground);
		box-shadow: var(--shadow-widget);
		font-family: var(--font-body);
		overflow: hidden;
	}
	.header {
		font-family: var(--font-display);
		font-size: 14px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--muted-foreground);
	}
	.messages {
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		gap: 4px;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}
	.message {
		font-size: 15px;
		line-height: 1.35;
		word-break: break-word;
	}
	.user {
		font-weight: 700;
		color: var(--primary);
	}
	.user::after {
		content: ':';
		margin-right: 6px;
		color: var(--muted-foreground);
	}
	.text {
		color: var(--card-foreground);
	}
</style>
