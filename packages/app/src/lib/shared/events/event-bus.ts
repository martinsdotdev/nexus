// A tiny in-process pub/sub for overlay events. Subscribers match by exact kind
// ('alert.follow'), a one-level prefix wildcard ('alert.*'), or '*' for all.
// One bus per overlay/editor tab; not persisted, not synced (spec §9).

import type { OverlayEvent } from './events';

export type EventHandler = (event: OverlayEvent) => void;

export interface EventBus {
	emit(event: OverlayEvent): void;
	/** Subscribe to a kind, a `prefix.*` wildcard, or `*`. Returns an unsubscribe. */
	subscribe(pattern: string, handler: EventHandler): () => void;
}

function matches(pattern: string, kind: string): boolean {
	if (pattern === '*' || pattern === kind) return true;
	if (pattern.endsWith('.*')) return kind.startsWith(pattern.slice(0, -1));
	return false;
}

export function createEventBus(): EventBus {
	const subscribers = new Map<string, Set<EventHandler>>();
	return {
		emit(event) {
			for (const [pattern, handlers] of subscribers) {
				if (matches(pattern, event.kind)) {
					for (const handler of handlers) handler(event);
				}
			}
		},
		subscribe(pattern, handler) {
			let handlers = subscribers.get(pattern);
			if (!handlers) {
				handlers = new Set();
				subscribers.set(pattern, handlers);
			}
			handlers.add(handler);
			return () => {
				handlers.delete(handler);
			};
		}
	};
}
