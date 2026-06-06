// Synthetic overlay events for QA-ing the event-driven widgets from the editor without a live
// platform (the "Fire test event" command). Pure and clock-injected: given an index and a
// timestamp it returns the next representative event, so the command can cycle through them and
// repeated fires get distinct time-bearing keys. Mirrors the vocabulary in ./events.
import type { OverlayEvent } from './events';

const SAMPLES: OverlayEvent[] = [
	{ kind: 'alert.follow', payload: { user: 'testfollower', timestamp: 0 } },
	{
		kind: 'alert.subscribe',
		payload: { user: 'testsub', tier: 2, months: 6, message: 'love the stream!' }
	},
	{ kind: 'alert.cheer', payload: { user: 'testcheer', bits: 1000, message: 'pog' } },
	{ kind: 'alert.raid', payload: { fromChannel: 'testraider', viewers: 142 } },
	{ kind: 'goal.increment', payload: { goalKind: 'follower', by: 1, total: 50 } },
	{
		kind: 'media.track-changed',
		payload: { title: 'Test Track', artist: 'Test Artist', source: 'test' }
	},
	{
		kind: 'stream.info-changed',
		payload: { title: 'Testing widgets', game: 'Just Chatting', viewerCount: 100, uptimeSec: 600 }
	}
];

/** How many distinct test events the cycle offers. */
export const TEST_EVENT_COUNT = SAMPLES.length;

/**
 * The synthetic event at `index` (wraps), with time-bearing payloads stamped from the injected
 * `now` so repeated fires render as distinct entries rather than colliding on a key.
 */
export function testEventAt(index: number, now: number): OverlayEvent {
	const i = ((index % SAMPLES.length) + SAMPLES.length) % SAMPLES.length;
	const sample = SAMPLES[i];
	if (sample.kind === 'alert.follow') {
		return { kind: 'alert.follow', payload: { ...sample.payload, timestamp: now } };
	}
	return sample;
}
