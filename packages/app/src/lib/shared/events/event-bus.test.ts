import { describe, expect, it, vi } from 'vitest';
import { createEventBus } from './event-bus';
import type { OverlayEvent } from './events';

const follow: OverlayEvent = { kind: 'alert.follow', payload: { user: 'a', timestamp: 0 } };
const chat: OverlayEvent = {
	kind: 'chat.message',
	payload: { user: 'c', text: 'hi', color: '#fff', badges: [], timestamp: 0 }
};

describe('event bus', () => {
	it('delivers only to exact-kind subscribers', () => {
		const bus = createEventBus();
		const handler = vi.fn();
		bus.subscribe('alert.follow', handler);
		bus.emit(follow);
		bus.emit(chat);
		expect(handler).toHaveBeenCalledTimes(1);
	});

	it('matches prefix wildcards and "*"', () => {
		const bus = createEventBus();
		const alerts = vi.fn();
		const all = vi.fn();
		bus.subscribe('alert.*', alerts);
		bus.subscribe('*', all);
		bus.emit(follow);
		bus.emit(chat);
		expect(alerts).toHaveBeenCalledTimes(1);
		expect(all).toHaveBeenCalledTimes(2);
	});

	it('stops delivering after unsubscribe', () => {
		const bus = createEventBus();
		const handler = vi.fn();
		const off = bus.subscribe('chat.message', handler);
		off();
		bus.emit(chat);
		expect(handler).not.toHaveBeenCalled();
	});
});
