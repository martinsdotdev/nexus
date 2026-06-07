// The editor's local-first workspace client: a Loro replica wired to the relay, exposed to
// the UI as reactive Svelte state. The local doc IS the store; a version counter (bumped on
// every doc change, local or remote) drives the `$derived` reads, so the UI re-renders
// whether an edit was made here or arrived from another collaborator. Replaces TanStack Query
// for document state (ADR-0005). Must be created client-side only (it touches WASM + WebSocket).
//
// Live/Draft (per-editor): in draft mode the client edits a private fork of the live doc (see
// ./draft); the live doc keeps syncing, so the overlay + peers show the last live state until
// Publish merges the fork back in. Reads, writes, undo, and presence all follow the active doc.

import { LoroDoc, UndoManager } from 'loro-crdt';
import { connectSync, type SyncConnection, type ConnectionState } from './sync-bridge';
import {
	readWorkspace,
	type SceneView,
	type WorkspaceView,
	type ThemeTokens
} from './workspace-view';
import * as mutate from './mutations';
import type { WidgetGeometry } from './mutations';
import { createPresence, type PeerIdentity, type PeerPresence, type Presence } from './presence';
import { beginDraft, publishDraft, type Draft } from './draft';

const TREE = 'tree';
const WORKSPACE = 'workspace';

/** 'live' (edits broadcast immediately) or 'draft' (edits are private until published). */
export type EditorMode = 'live' | 'draft';

export interface WorkspaceClient {
	readonly scenes: SceneView[];
	readonly activeSceneId: string;
	readonly workspace: WorkspaceView;
	readonly canUndo: boolean;
	readonly canRedo: boolean;
	/** Other editors currently present (cursors, selections), de-duped by account. */
	readonly remotePeers: PeerPresence[];
	/** The live link to the relay (for the sync-status pill). */
	readonly connection: ConnectionState;
	/** Whether this editor is editing live or a private draft. */
	readonly mode: EditorMode;
	/** Share this editor's pointer position (virtual canvas coordinates). */
	setCursor(x: number, y: number): void;
	/** Share this editor's current widget selection. */
	setSelection(ids: string[]): void;
	activate(sceneId: string): void;
	setWidgetGeometry(id: string, geom: Partial<WidgetGeometry>): void;
	setWidgetZ(id: string, z: number): void;
	setWidgetVisible(id: string, visible: boolean): void;
	setWidgetProp(id: string, key: string, value: unknown): void;
	createWidget(
		sceneId: string,
		widgetType: string,
		geom: WidgetGeometry,
		props: Record<string, unknown>
	): string | undefined;
	deleteWidget(id: string): void;
	moveWidgetToScene(id: string, sceneId: string): void;
	setSceneTheme(sceneId: string, themeId: string): void;
	setSceneOverride(sceneId: string, key: string, value: string): void;
	/** Add a freeform scene to the active layout; returns its id. */
	createScene(name: string): string | undefined;
	renameScene(sceneId: string, name: string): void;
	/** Delete a scene (refused for a layout's only scene; re-points active first). */
	deleteScene(sceneId: string): void;
	duplicateScene(sceneId: string, name: string): string | undefined;
	/** Move a scene to a new index among the active layout's scenes. */
	reorderScene(sceneId: string, index: number): void;
	createTheme(name: string, base: string, tokens: ThemeTokens): string;
	renameTheme(id: string, name: string): void;
	setThemeToken(id: string, token: string, value: string): void;
	deleteTheme(id: string): void;
	exportTheme(id: string): { name: string; base: string; tokens: ThemeTokens } | null;
	createLayout(name: string): string;
	renameLayout(id: string, name: string): void;
	duplicateLayout(id: string, name: string): string | undefined;
	archiveLayout(id: string): void;
	activateLayout(id: string): void;
	/** Enter draft mode: fork the live doc; edits stay private until publish. */
	enterDraft(): void;
	/** Merge the draft into live (broadcasts to the relay + overlay) and return to live. */
	publish(): void;
	/** Drop the draft and return to live, discarding its edits. */
	discard(): void;
	undo(): void;
	redo(): void;
	dispose(): void;
}

export function createWorkspaceClient(
	url: string,
	options: { identity?: PeerIdentity } = {}
): WorkspaceClient {
	const liveDoc = new LoroDoc();
	let version = $state(0);

	const liveUnsub = liveDoc.subscribe(() => {
		version += 1;
	});

	// Draft state: a private fork of the live doc. While drafting, reads + writes target the
	// fork; the live doc keeps syncing (peers + overlay see live). `mode` is client-only.
	let mode = $state<EditorMode>('live');
	let draft: Draft | undefined;
	let draftUndo: UndoManager | undefined;
	let draftUnsub = () => {};
	const active = () => draft?.doc ?? liveDoc;

	// Ephemeral presence (cursors, selections, identity), wired to the relay's presence
	// channel on the LIVE doc. Disabled when there is no identity (local mode has no account).
	let presenceVersion = $state(0);
	let presence: Presence | undefined;
	let connectionState = $state<ConnectionState>('syncing');
	const sync: SyncConnection = connectSync(liveDoc, url, {
		onPresence: (bytes) => presence?.apply(bytes),
		onState: (state) => {
			connectionState = state;
		}
	});
	let presenceUnsub = () => {};
	if (options.identity) {
		presence = createPresence(options.identity, (bytes) => sync.sendPresence(bytes));
		presenceUnsub = presence.subscribe(() => {
			presenceVersion += 1;
		});
	}

	// Local undo over THIS peer's edits only; remote merges + relay repairs are a different
	// peer and never land on the stack. mergeInterval 0 keeps each commit its own undo step.
	// A draft gets its own UndoManager so undo within a draft does not touch live history.
	const liveUndo = new UndoManager(liveDoc, { mergeInterval: 0 });
	const undoMgr = () => draftUndo ?? liveUndo;

	function activeLayout(doc: LoroDoc) {
		const tree = doc.getTree(TREE);
		const roots = tree.roots();
		const activeLayoutId = String(doc.getMap(WORKSPACE).get('activeLayoutId') ?? '');
		return roots.find((node) => String(node.id) === activeLayoutId) ?? roots[0];
	}

	// Run a mutation against the active doc then commit (one undoable step) and return its
	// result. Every write goes through this, so "forgot to commit" is structurally impossible.
	const tx = <R>(run: (doc: LoroDoc) => R): R => {
		const doc = active();
		const result = run(doc);
		doc.commit();
		return result;
	};

	function exitDraft() {
		draftUnsub();
		draftUnsub = () => {};
		draftUndo?.free();
		draftUndo = undefined;
		draft = undefined;
		mode = 'live';
		version += 1; // re-read the live doc
	}

	return {
		get scenes(): SceneView[] {
			void version;
			return readWorkspace(active()).scenes;
		},
		get activeSceneId(): string {
			void version;
			return readWorkspace(active()).activeSceneId;
		},
		get workspace(): WorkspaceView {
			void version;
			return readWorkspace(active());
		},
		get canUndo(): boolean {
			void version;
			return undoMgr().canUndo();
		},
		get canRedo(): boolean {
			void version;
			return undoMgr().canRedo();
		},
		get remotePeers(): PeerPresence[] {
			void presenceVersion;
			return presence?.remotePeers() ?? [];
		},
		get connection(): ConnectionState {
			return connectionState;
		},
		get mode(): EditorMode {
			return mode;
		},
		setCursor(x, y) {
			// A draft is private, so its cursor (on a divergent canvas) is not broadcast.
			if (mode === 'live') presence?.setCursor(x, y);
		},
		setSelection(ids) {
			if (mode === 'live') presence?.setSelection(ids);
		},
		activate(sceneId: string) {
			tx((doc) => {
				const layout = activeLayout(doc);
				if (layout) layout.data.set('activeSceneId', sceneId);
			});
		},
		setWidgetGeometry(id, geom) {
			tx((doc) => mutate.setWidgetGeometry(doc, id, geom));
		},
		setWidgetZ(id, z) {
			tx((doc) => mutate.setWidgetZ(doc, id, z));
		},
		setWidgetVisible(id, visible) {
			tx((doc) => mutate.setWidgetVisible(doc, id, visible));
		},
		setWidgetProp(id, key, value) {
			tx((doc) => mutate.setWidgetProp(doc, id, key, value));
		},
		createWidget(sceneId, widgetType, geom, props) {
			return tx((doc) => mutate.createWidget(doc, sceneId, widgetType, geom, props));
		},
		deleteWidget(id) {
			tx((doc) => mutate.deleteWidget(doc, id));
		},
		moveWidgetToScene(id, sceneId) {
			tx((doc) => mutate.moveWidgetToScene(doc, id, sceneId));
		},
		setSceneTheme(sceneId, themeId) {
			tx((doc) => mutate.setSceneTheme(doc, sceneId, themeId));
		},
		setSceneOverride(sceneId, key, value) {
			tx((doc) => mutate.setSceneOverride(doc, sceneId, key, value));
		},
		createScene(name) {
			return tx((doc) => {
				const layout = activeLayout(doc);
				return layout ? mutate.createScene(doc, String(layout.id), name) : undefined;
			});
		},
		renameScene(sceneId, name) {
			tx((doc) => mutate.renameScene(doc, sceneId, name));
		},
		deleteScene(sceneId) {
			tx((doc) => mutate.deleteScene(doc, sceneId));
		},
		duplicateScene(sceneId, name) {
			return tx((doc) => mutate.duplicateScene(doc, sceneId, name));
		},
		reorderScene(sceneId, index) {
			tx((doc) => mutate.reorderScene(doc, sceneId, index));
		},
		createTheme(name, base, tokens) {
			// Mint a unique id (never the name) so concurrent creates never collide.
			const id = `theme-${crypto.randomUUID()}`;
			tx((doc) => mutate.createTheme(doc, id, name, base, tokens));
			return id;
		},
		renameTheme(id, name) {
			tx((doc) => mutate.renameTheme(doc, id, name));
		},
		setThemeToken(id, token, value) {
			tx((doc) => mutate.setThemeToken(doc, id, token, value));
		},
		deleteTheme(id) {
			tx((doc) => mutate.deleteTheme(doc, id));
		},
		exportTheme(id) {
			return mutate.exportTheme(active(), id);
		},
		createLayout(name) {
			return tx((doc) => mutate.createLayout(doc, name));
		},
		renameLayout(id, name) {
			tx((doc) => mutate.renameLayout(doc, id, name));
		},
		duplicateLayout(id, name) {
			return tx((doc) => mutate.duplicateLayout(doc, id, name));
		},
		archiveLayout(id) {
			tx((doc) => mutate.archiveLayout(doc, id));
		},
		activateLayout(id) {
			tx((doc) => mutate.activateLayout(doc, id));
		},
		enterDraft() {
			if (mode === 'draft') return;
			draft = beginDraft(liveDoc);
			draftUndo = new UndoManager(draft.doc, { mergeInterval: 0 });
			draftUnsub = draft.doc.subscribe(() => {
				version += 1;
			});
			mode = 'draft';
			version += 1;
		},
		publish() {
			if (!draft) return;
			// import() does not fire the live doc's local-update subscriber, so re-broadcast
			// the published delta to the relay explicitly (it merges + forwards to peers).
			const updates = publishDraft(liveDoc, draft);
			sync.sendUpdate(updates);
			exitDraft();
		},
		discard() {
			if (!draft) return;
			exitDraft();
		},
		undo() {
			undoMgr().undo();
		},
		redo() {
			undoMgr().redo();
		},
		dispose() {
			draftUnsub();
			draftUndo?.free();
			presenceUnsub();
			presence?.destroy();
			liveUnsub();
			sync.close();
			liveUndo.free();
		}
	};
}

export interface ReadOnlyClient {
	readonly workspace: WorkspaceView;
	dispose(): void;
}

/** A read-only replica for the overlay: imports relay state, never writes. */
export function createReadOnlyClient(url: string): ReadOnlyClient {
	const doc = new LoroDoc();
	let version = $state(0);

	const unsubscribe = doc.subscribe(() => {
		version += 1;
	});
	const sync: SyncConnection = connectSync(doc, url, { readonly: true });

	return {
		get workspace(): WorkspaceView {
			void version;
			return readWorkspace(doc);
		},
		dispose() {
			unsubscribe();
			sync.close();
		}
	};
}
