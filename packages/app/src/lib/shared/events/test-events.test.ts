import { expect, test } from 'vitest';
import { TEST_EVENT_COUNT, testEventAt } from './test-events';

test('cycles through distinct representative events and wraps', () => {
	const kinds = new Set(Array.from({ length: TEST_EVENT_COUNT }, (_, i) => testEventAt(i, 0).kind));
	expect(kinds.size).toBe(TEST_EVENT_COUNT);
	// One full cycle later returns the same kind.
	expect(testEventAt(TEST_EVENT_COUNT, 0).kind).toBe(testEventAt(0, 0).kind);
});

test('stamps the follow alert from the injected clock', () => {
	const follow = Array.from({ length: TEST_EVENT_COUNT }, (_, i) => testEventAt(i, 4242)).find(
		(event) => event.kind === 'alert.follow'
	);
	expect(follow?.kind).toBe('alert.follow');
	if (follow?.kind === 'alert.follow') expect(follow.payload.timestamp).toBe(4242);
});
