// Bridges a local Loro replica to the relay's /sync WebSocket: local ops are shipped as
// Update frames, inbound Snapshot/Update frames are imported. On (re)connect the client
// flushes its accumulated ops so edits made while offline resync and merge (ADR-0005). A
// dropped socket reconnects on a capped backoff, and the connection state is reported so
// the editor can surface it (Synced / Syncing / Reconnecting / Working offline).

import type { LoroDoc } from 'loro-crdt';
import { decodeFrame, encodeFrame } from './protocol';

/** Where the live link to the relay stands, for the sync-status pill. */
export type ConnectionState = 'syncing' | 'synced' | 'reconnecting' | 'offline';

export interface SyncConnection {
	close(): void;
	/** Ship an opaque presence frame to the relay, which forwards it to other peers. */
	sendPresence(bytes: Uint8Array): void;
	/** Ship a document update frame (e.g. a published draft) to the relay. `doc.import()`
	 *  does not fire the local-update subscriber, so the caller re-broadcasts explicitly. */
	sendUpdate(bytes: Uint8Array): void;
}

interface SyncOptions {
	readonly?: boolean;
	onPresence?: (bytes: Uint8Array) => void;
	onState?: (state: ConnectionState) => void;
}

// Default to "online" unless the browser explicitly says otherwise (and in non-browser
// test envs where `navigator` may be absent).
const isOnline = () => typeof navigator === 'undefined' || navigator.onLine !== false;

export function connectSync(doc: LoroDoc, url: string, options: SyncOptions = {}): SyncConnection {
	let ws!: WebSocket;
	let disposed = false;
	let retries = 0;
	let reconnectTimer: ReturnType<typeof setTimeout> | undefined;

	const setState = (state: ConnectionState) => options.onState?.(state);

	function connect() {
		setState(retries === 0 ? 'syncing' : 'reconnecting');
		ws = new WebSocket(url);
		ws.binaryType = 'arraybuffer';

		ws.onopen = () => {
			retries = 0;
			setState('synced');
			// Flush all local ops so offline edits reach the relay on (re)connect.
			if (!options.readonly) {
				ws.send(encodeFrame({ kind: 'update', payload: doc.export({ mode: 'update' }) }));
			}
		};

		ws.onmessage = (event) => {
			const frame = decodeFrame(new Uint8Array(event.data as ArrayBuffer));
			if (!frame) return;
			if (frame.kind === 'snapshot' || frame.kind === 'update') {
				doc.import(frame.payload);
			} else if (frame.kind === 'presence') {
				options.onPresence?.(frame.payload);
			}
		};

		ws.onclose = () => {
			if (disposed) return;
			retries += 1;
			setState(isOnline() ? 'reconnecting' : 'offline');
			const delay = Math.min(8000, 500 * 2 ** Math.min(retries, 4));
			reconnectTimer = setTimeout(connect, delay);
		};
	}

	connect();

	// Ship every local op to the relay (no-op while the socket is not open; the on-open
	// flush above catches anything made in the meantime). The closure reads the live `ws`,
	// so it follows the socket across reconnects.
	let unsubscribe = () => {};
	if (!options.readonly) {
		unsubscribe = doc.subscribeLocalUpdates((bytes) => {
			if (ws.readyState === WebSocket.OPEN) {
				ws.send(encodeFrame({ kind: 'update', payload: bytes }));
			}
		});
	}

	return {
		close() {
			disposed = true;
			clearTimeout(reconnectTimer);
			unsubscribe();
			ws.close();
		},
		sendPresence(bytes) {
			if (ws.readyState === WebSocket.OPEN) {
				ws.send(encodeFrame({ kind: 'presence', payload: bytes }));
			}
		},
		sendUpdate(bytes) {
			if (ws.readyState === WebSocket.OPEN) {
				ws.send(encodeFrame({ kind: 'update', payload: bytes }));
			}
		}
	};
}
