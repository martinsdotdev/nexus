// A scripted event source so the event-driven overlay widgets show live activity
// without a real platform connection (spec §6.4). Browser-only (uses timers +
// Date.now()); start it from the overlay route inside an $effect.

import type { EventBus } from './event-bus';
import type { OverlayEvent } from './events';

const CHAT = [
	{ user: 'pixelpit', text: 'first!', color: '#f59e0b' },
	{ user: 'lurkerjoe', text: 'love this overlay', color: '#34d399' },
	{ user: 'gg_gamer', text: 'what game is this?', color: '#60a5fa' },
	{ user: 'streamfan', text: 'this theme is clean', color: '#f472b6' },
	{ user: 'modsquad', text: 'welcome everyone', color: '#a78bfa' }
];
const FOLLOWERS = ['newviewer', 'coolcat', 'devdiva', 'nightowl', 'questgiver'];
const ALERTS: OverlayEvent[] = [
	{ kind: 'alert.subscribe', payload: { user: 'bigfan', tier: 1, months: 3 } },
	{ kind: 'alert.cheer', payload: { user: 'generous', bits: 500 } },
	{ kind: 'alert.raid', payload: { fromChannel: 'friendlystreamer', viewers: 87 } }
];
const TRACKS = [
	{ title: 'Neon Skyline', artist: 'Synthwave Co' },
	{ title: 'Coffee & Code', artist: 'Lofi Beats' }
];

const pick = <T>(items: T[], i: number): T => items[i % items.length];

/** Start the scripted source; returns a stop function that clears all timers. */
export function startMockSource(bus: EventBus): () => void {
	let n = 0;
	let goal = 42;
	const timers: ReturnType<typeof setInterval>[] = [];

	timers.push(
		setInterval(() => {
			bus.emit({
				kind: 'chat.message',
				payload: { ...pick(CHAT, n++), badges: [], timestamp: Date.now() }
			});
		}, 2200)
	);
	timers.push(
		setInterval(() => {
			bus.emit({
				kind: 'alert.follow',
				payload: { user: pick(FOLLOWERS, n++), timestamp: Date.now() }
			});
		}, 5200)
	);
	timers.push(
		setInterval(() => {
			goal += 1;
			bus.emit({ kind: 'goal.increment', payload: { goalKind: 'follower', by: 1, total: goal } });
		}, 4000)
	);
	timers.push(
		setInterval(() => {
			bus.emit(pick(ALERTS, n++));
		}, 9000)
	);
	timers.push(
		setInterval(() => {
			bus.emit({ kind: 'media.track-changed', payload: { ...pick(TRACKS, n++), source: 'mock' } });
		}, 12000)
	);
	timers.push(
		setInterval(() => {
			bus.emit({
				kind: 'stream.info-changed',
				payload: {
					title: 'Building Nexus live',
					game: 'Software & Game Dev',
					viewerCount: 120 + (n % 30),
					uptimeSec: n * 30
				}
			});
		}, 8000)
	);

	return () => timers.forEach((timer) => clearInterval(timer));
}
