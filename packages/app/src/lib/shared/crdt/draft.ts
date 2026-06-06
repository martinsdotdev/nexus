// Per-editor Live/Draft via a Loro fork. Entering draft forks the live document into a
// private copy the editor edits; the live doc keeps syncing, so collaborators and the
// overlay keep showing the last live state. Publish merges the fork's changes back into
// live and returns those bytes so the caller can re-broadcast them (import() does not fire
// local-update subscribers, so the relay would otherwise never see them). Discard just drops
// the fork. The draft never leaves this editor until it is published (ADR-0005: Loro is the
// document). Pure: given the docs, no reactivity or I/O.
import type { LoroDoc, VersionVector } from 'loro-crdt';

export interface Draft {
	/** The forked document the editor writes to while in draft mode. */
	doc: LoroDoc;
	/** The live version at the fork, so publish exports only the draft's own changes. */
	from: VersionVector;
}

/** Fork the live document into a private draft branch. */
export function beginDraft(live: LoroDoc): Draft {
	return { doc: live.fork(), from: live.version() };
}

/** Merge the draft's changes into the live doc; returns the update bytes to re-broadcast. */
export function publishDraft(live: LoroDoc, draft: Draft): Uint8Array {
	draft.doc.commit();
	const updates = draft.doc.export({ mode: 'update', from: draft.from });
	live.import(updates);
	return updates;
}
