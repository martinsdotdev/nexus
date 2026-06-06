import { afterEach, describe, expect, it, vi } from 'vitest';
import { LoroDoc } from 'loro-crdt';
import { connectSync, type ConnectionState } from './sync-bridge';

// A controllable WebSocket stand-in: tests drive emitOpen()/close() manually.
class MockWS {
	static CONNECTING = 0;
	static OPEN = 1;
	static CLOSED = 3;
	static instances: MockWS[] = [];
	readyState = MockWS.CONNECTING;
	binaryType = '';
	onopen: (() => void) | null = null;
	onmessage: ((e: { data: ArrayBuffer }) => void) | null = null;
	onclose: (() => void) | null = null;
	sent: Uint8Array[] = [];
	constructor(public url: string) {
		MockWS.instances.push(this);
	}
	send(data: Uint8Array) {
		this.sent.push(data);
	}
	close() {
		this.readyState = MockWS.CLOSED;
		this.onclose?.();
	}
	emitOpen() {
		this.readyState = MockWS.OPEN;
		this.onopen?.();
	}
}

afterEach(() => {
	MockWS.instances = [];
	vi.restoreAllMocks();
	vi.useRealTimers();
});

function setup(onLine = true) {
	vi.stubGlobal('WebSocket', MockWS);
	vi.stubGlobal('navigator', { onLine });
	const states: ConnectionState[] = [];
	const conn = connectSync(new LoroDoc(), 'ws://relay/sync', { onState: (s) => states.push(s) });
	return { states, conn };
}

describe('sync-bridge connection state', () => {
	it('reports syncing then synced once the socket opens', () => {
		const { states, conn } = setup();
		expect(states[0]).toBe('syncing');
		MockWS.instances[0].emitOpen();
		expect(states).toContain('synced');
		conn.close();
	});

	it('reconnects on an unexpected close', () => {
		vi.useFakeTimers();
		const { states, conn } = setup();
		MockWS.instances[0].emitOpen();
		MockWS.instances[0].close();
		expect(states.at(-1)).toBe('reconnecting');
		// The backoff fires a fresh connection attempt.
		vi.runOnlyPendingTimers();
		expect(MockWS.instances).toHaveLength(2);
		conn.close();
	});

	it('shows working offline when the browser is offline', () => {
		vi.useFakeTimers();
		const { states, conn } = setup(false);
		MockWS.instances[0].emitOpen();
		MockWS.instances[0].close();
		expect(states.at(-1)).toBe('offline');
		conn.close();
	});

	it('does not reconnect after an intentional close', () => {
		vi.useFakeTimers();
		const { conn } = setup();
		MockWS.instances[0].emitOpen();
		conn.close();
		vi.runOnlyPendingTimers();
		expect(MockWS.instances).toHaveLength(1);
	});
});
