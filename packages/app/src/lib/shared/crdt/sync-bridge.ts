// Bridges a local Loro replica to the relay's /sync WebSocket: local ops are
// shipped as Update frames, inbound Snapshot/Update frames are imported. On
// (re)connect the client flushes its accumulated ops so edits made while
// offline resync and merge (ADR-0005). The relay validates/repairs/rebroadcasts.

import type { LoroDoc } from 'loro-crdt';
import { decodeFrame, encodeFrame } from './protocol';

export interface SyncConnection {
	close(): void;
	/** Ship an opaque presence frame to the relay, which forwards it to other peers. */
	sendPresence(bytes: Uint8Array): void;
}

export function connectSync(
	doc: LoroDoc,
	url: string,
	options: { readonly?: boolean; onPresence?: (bytes: Uint8Array) => void } = {}
): SyncConnection {
	const ws = new WebSocket(url);
	ws.binaryType = 'arraybuffer';

	// A read-only replica (the overlay) imports relay state but never sends.
	let unsubscribe = () => {};
	if (!options.readonly) {
		// Ship every local op to the relay (no-op while the socket is not open;
		// the on-open flush below catches anything made in the meantime).
		unsubscribe = doc.subscribeLocalUpdates((bytes) => {
			if (ws.readyState === WebSocket.OPEN) {
				ws.send(encodeFrame({ kind: 'update', payload: bytes }));
			}
		});
		ws.onopen = () => {
			// Flush all local ops so offline edits reach the relay on (re)connect.
			ws.send(encodeFrame({ kind: 'update', payload: doc.export({ mode: 'update' }) }));
		};
	}

	ws.onmessage = (event) => {
		const frame = decodeFrame(new Uint8Array(event.data as ArrayBuffer));
		if (!frame) return;
		if (frame.kind === 'snapshot' || frame.kind === 'update') {
			doc.import(frame.payload);
		} else if (frame.kind === 'presence') {
			options.onPresence?.(frame.payload);
		}
	};

	return {
		close() {
			unsubscribe();
			ws.close();
		},
		sendPresence(bytes) {
			if (ws.readyState === WebSocket.OPEN) {
				ws.send(encodeFrame({ kind: 'presence', payload: bytes }));
			}
		}
	};
}
