// Wire framing for the /sync WebSocket, mirroring the Rust relay's tags
// (crates/nexus-server/src/protocol.rs): a 1-byte tag followed by the payload.
//   0x01 snapshot-request (no payload)
//   0x02 snapshot (full Loro snapshot bytes)
//   0x03 update   (incremental Loro update bytes)
//   0x04 presence (opaque EphemeralStore bytes; the relay forwards, never merges)
// An unknown tag decodes to null, so a new tag is forward compatible.

export type Frame =
	| { kind: 'snapshot-request' }
	| { kind: 'snapshot'; payload: Uint8Array }
	| { kind: 'update'; payload: Uint8Array }
	| { kind: 'presence'; payload: Uint8Array };

const TAG = { 'snapshot-request': 0x01, snapshot: 0x02, update: 0x03, presence: 0x04 } as const;

function tagged(tag: number, payload: Uint8Array): Uint8Array<ArrayBuffer> {
	const out = new Uint8Array(payload.length + 1);
	out[0] = tag;
	out.set(payload, 1);
	return out;
}

export function encodeFrame(frame: Frame): Uint8Array<ArrayBuffer> {
	switch (frame.kind) {
		case 'snapshot-request':
			return new Uint8Array([TAG['snapshot-request']]);
		case 'snapshot':
			return tagged(TAG.snapshot, frame.payload);
		case 'update':
			return tagged(TAG.update, frame.payload);
		case 'presence':
			return tagged(TAG.presence, frame.payload);
	}
}

export function decodeFrame(bytes: Uint8Array): Frame | null {
	if (bytes.length === 0) return null;
	const payload = bytes.slice(1);
	switch (bytes[0]) {
		case TAG['snapshot-request']:
			return payload.length === 0 ? { kind: 'snapshot-request' } : null;
		case TAG.snapshot:
			return { kind: 'snapshot', payload };
		case TAG.update:
			return { kind: 'update', payload };
		case TAG.presence:
			return { kind: 'presence', payload };
		default:
			return null;
	}
}
