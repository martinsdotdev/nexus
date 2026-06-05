// The editor's local-first workspace client: a Loro replica wired to the relay,
// exposed to the UI as reactive Svelte state. The local doc IS the store, a
// version counter (bumped on every doc change, local or remote) drives the
// `$derived` reads, so the UI re-renders whether an edit was made here or
// arrived from another collaborator. Replaces TanStack Query for document state
// (ADR-0005). Must be created client-side only (it touches WASM + WebSocket).

import { LoroDoc, UndoManager } from 'loro-crdt';
import { connectSync, type SyncConnection } from './sync-bridge';
import {
	readWorkspace,
	type SceneView,
	type WorkspaceView,
	type ThemeTokens
} from './workspace-view';
import * as mutate from './mutations';
import type { WidgetGeometry } from './mutations';
import { createPresence, type PeerIdentity, type PeerPresence, type Presence } from './presence';

const TREE = 'tree';
const WORKSPACE = 'workspace';

export interface WorkspaceClient {
	readonly scenes: SceneView[];
	readonly activeSceneId: string;
	readonly workspace: WorkspaceView;
	readonly canUndo: boolean;
	readonly canRedo: boolean;
	/** Other editors currently present (cursors, selections), de-duped by account. */
	readonly remotePeers: PeerPresence[];
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
	createTheme(name: string, base: string, tokens: ThemeTokens): string;
	renameTheme(id: string, name: string): void;
	setThemeToken(id: string, token: string, value: string): void;
	deleteTheme(id: string): void;
	exportTheme(id: string): { name: string; base: string; tokens: ThemeTokens } | null;
	undo(): void;
	redo(): void;
	dispose(): void;
}

export function createWorkspaceClient(
	url: string,
	options: { identity?: PeerIdentity } = {}
): WorkspaceClient {
	const doc = new LoroDoc();
	let version = $state(0);

	const unsubscribe = doc.subscribe(() => {
		version += 1;
	});

	// Ephemeral presence (cursors, selections, identity), wired to the relay's presence
	// channel. Disabled when there is no identity (local mode has no signed-in account).
	let presenceVersion = $state(0);
	let presence: Presence | undefined;
	const sync: SyncConnection = connectSync(doc, url, {
		onPresence: (bytes) => presence?.apply(bytes)
	});
	let presenceUnsub = () => {};
	if (options.identity) {
		presence = createPresence(options.identity, (bytes) => sync.sendPresence(bytes));
		presenceUnsub = presence.subscribe(() => {
			presenceVersion += 1;
		});
	}

	// Local undo over THIS peer's edits only; remote merges + relay repairs are a
	// different peer and never land on the stack. mergeInterval 0 keeps each commit
	// its own undo step: a drag/add/delete commits exactly once, and discrete
	// actions never merge (a time window would fold a scene activation into a later
	// add, so one undo would revert both). Inspector typing is per-keystroke but
	// predictable; debounced commits are a later refinement.
	const undo = new UndoManager(doc, { mergeInterval: 0 });

	function activeLayout() {
		const tree = doc.getTree(TREE);
		const roots = tree.roots();
		const activeLayoutId = String(doc.getMap(WORKSPACE).get('activeLayoutId') ?? '');
		return roots.find((node) => String(node.id) === activeLayoutId) ?? roots[0];
	}

	// Run a mutation then commit (one undoable step) and return its result. Every
	// write goes through this, so "forgot to commit" is structurally impossible.
	const tx = <R>(run: () => R): R => {
		const result = run();
		doc.commit();
		return result;
	};

	return {
		get scenes(): SceneView[] {
			void version;
			return readWorkspace(doc).scenes;
		},
		get activeSceneId(): string {
			void version;
			return readWorkspace(doc).activeSceneId;
		},
		get workspace(): WorkspaceView {
			void version;
			return readWorkspace(doc);
		},
		get canUndo(): boolean {
			void version;
			return undo.canUndo();
		},
		get canRedo(): boolean {
			void version;
			return undo.canRedo();
		},
		get remotePeers(): PeerPresence[] {
			void presenceVersion;
			return presence?.remotePeers() ?? [];
		},
		setCursor(x, y) {
			presence?.setCursor(x, y);
		},
		setSelection(ids) {
			presence?.setSelection(ids);
		},
		activate(sceneId: string) {
			tx(() => {
				const layout = activeLayout();
				if (layout) layout.data.set('activeSceneId', sceneId);
			});
		},
		setWidgetGeometry(id, geom) {
			tx(() => mutate.setWidgetGeometry(doc, id, geom));
		},
		setWidgetZ(id, z) {
			tx(() => mutate.setWidgetZ(doc, id, z));
		},
		setWidgetVisible(id, visible) {
			tx(() => mutate.setWidgetVisible(doc, id, visible));
		},
		setWidgetProp(id, key, value) {
			tx(() => mutate.setWidgetProp(doc, id, key, value));
		},
		createWidget(sceneId, widgetType, geom, props) {
			return tx(() => mutate.createWidget(doc, sceneId, widgetType, geom, props));
		},
		deleteWidget(id) {
			tx(() => mutate.deleteWidget(doc, id));
		},
		moveWidgetToScene(id, sceneId) {
			tx(() => mutate.moveWidgetToScene(doc, id, sceneId));
		},
		setSceneTheme(sceneId, themeId) {
			tx(() => mutate.setSceneTheme(doc, sceneId, themeId));
		},
		setSceneOverride(sceneId, key, value) {
			tx(() => mutate.setSceneOverride(doc, sceneId, key, value));
		},
		createTheme(name, base, tokens) {
			// Mint a unique id (never the name) so concurrent creates never collide.
			const id = `theme-${crypto.randomUUID()}`;
			tx(() => mutate.createTheme(doc, id, name, base, tokens));
			return id;
		},
		renameTheme(id, name) {
			tx(() => mutate.renameTheme(doc, id, name));
		},
		setThemeToken(id, token, value) {
			tx(() => mutate.setThemeToken(doc, id, token, value));
		},
		deleteTheme(id) {
			tx(() => mutate.deleteTheme(doc, id));
		},
		exportTheme(id) {
			return mutate.exportTheme(doc, id);
		},
		undo() {
			undo.undo();
		},
		redo() {
			undo.redo();
		},
		dispose() {
			presenceUnsub();
			presence?.destroy();
			unsubscribe();
			sync.close();
			undo.free();
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
