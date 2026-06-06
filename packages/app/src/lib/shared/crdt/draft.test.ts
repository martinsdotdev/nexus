import { expect, test } from 'vitest';
import { LoroDoc } from 'loro-crdt';
import { beginDraft, publishDraft } from './draft';

function live(text: string): LoroDoc {
	const doc = new LoroDoc();
	doc.getText('t').insert(0, text);
	doc.commit();
	return doc;
}

test('a draft edit stays private until publish, which applies it and returns the delta', () => {
	const doc = live('hello');
	const draft = beginDraft(doc);
	draft.doc.getText('t').insert(5, ' world');
	draft.doc.commit();

	expect(doc.getText('t').toString()).toBe('hello'); // private until published
	const updates = publishDraft(doc, draft);
	expect(doc.getText('t').toString()).toBe('hello world');
	expect(updates.byteLength).toBeGreaterThan(0);
});

test('discarding (dropping the fork) leaves the live doc untouched', () => {
	const doc = live('hello');
	const draft = beginDraft(doc);
	draft.doc.getText('t').insert(5, ' world');
	draft.doc.commit();
	// Discard = simply drop `draft`; the live doc never saw the edit.
	expect(doc.getText('t').toString()).toBe('hello');
});

test('publish merges concurrent live edits made after the fork (CRDT)', () => {
	const doc = live('hello');
	const draft = beginDraft(doc);
	doc.getText('t').insert(0, '>> '); // a collaborator edits live after the fork
	doc.commit();
	draft.doc.getText('t').insert(5, '!');
	draft.doc.commit();

	publishDraft(doc, draft);
	const result = doc.getText('t').toString();
	expect(result).toContain('>>');
	expect(result).toContain('!');
});
