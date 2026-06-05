// Ephemeral collaborator presence (ADR-0005 layer 3): cursors, selections, and identity
// shared between editors over Loro's EphemeralStore. This is NOT part of the persistent
// document, the relay forwards presence frames opaquely and never stores them. Conflict
// resolution is last-write-wins per key with a timeout, so a departed peer's entry
// expires on its own. Presence is keyed by a per-connection id; the signed-in account
// identity rides in the value, and the roster de-dupes by it (two tabs = one person).

import { EphemeralStore, type Value } from 'loro-crdt';

export interface PeerIdentity {
	/** The signed-in account id; presence is de-duped by this. */
	id: string;
	/** A human display handle for the cursor label and roster. */
	name: string;
}

export interface PeerPresence {
	user: PeerIdentity;
	/** Pointer position in virtual canvas coordinates, when on the canvas. */
	cursor?: { x: number; y: number };
	/** Ids of the widgets this peer has selected. */
	selection: string[];
}

export interface Presence {
	setCursor(x: number, y: number): void;
	setSelection(ids: string[]): void;
	/** Merge an inbound presence frame from another peer. */
	apply(bytes: Uint8Array): void;
	/** Every other peer's presence, de-duped by account id. */
	remotePeers(): PeerPresence[];
	/** Notify on any presence change (local or remote); returns an unsubscribe. */
	subscribe(listener: () => void): () => void;
	destroy(): void;
}

/**
 * Create a presence channel for one editor. `send` ships our own presence frames to the
 * relay (which forwards them to other peers); inbound frames arrive via `apply`.
 */
export function createPresence(
	identity: PeerIdentity,
	send: (bytes: Uint8Array) => void,
	timeoutMs = 30_000
): Presence {
	// `EphemeralStore` stores opaque Loro `Value`s; `PeerPresence` is a known JSON shape,
	// so we cast at the get/set boundary (the runtime values are valid `Value`s).
	const store = new EphemeralStore(timeoutMs);
	// A unique key per tab/connection. Two tabs of one account are distinct entries,
	// merged only for display (by account id) in `remotePeers`.
	const localKey = `peer-${crypto.randomUUID()}`;
	let local: PeerPresence = { user: identity, selection: [] };

	const offLocalUpdates = store.subscribeLocalUpdates((bytes) => send(bytes));
	const publish = () => store.set(localKey, { ...local } as unknown as Value);

	// Announce our identity on the next tick (so a caller can finish wiring `send`
	// first), then re-announce on every change.
	queueMicrotask(publish);

	return {
		setCursor(x, y) {
			local = { ...local, cursor: { x, y } };
			publish();
		},
		setSelection(ids) {
			local = { ...local, selection: [...ids] };
			publish();
		},
		apply(bytes) {
			store.apply(bytes);
		},
		remotePeers() {
			const seen = new Set<string>();
			const peers: PeerPresence[] = [];
			for (const [key, raw] of Object.entries(store.getAllStates())) {
				const value = raw as unknown as PeerPresence | undefined;
				if (key === localKey || !value?.user || seen.has(value.user.id)) continue;
				seen.add(value.user.id);
				peers.push(value);
			}
			return peers;
		},
		subscribe(listener) {
			return store.subscribe(() => listener());
		},
		destroy() {
			offLocalUpdates();
			store.destroy();
		}
	};
}
