// The editor's local-first workspace client: a Loro replica wired to the relay,
// exposed to the UI as reactive Svelte state. The local doc IS the store, a
// version counter (bumped on every doc change, local or remote) drives the
// `$derived` reads, so the UI re-renders whether an edit was made here or
// arrived from another collaborator. Replaces TanStack Query for document state
// (ADR-0005). Must be created client-side only (it touches WASM + WebSocket).

import { LoroDoc, UndoManager } from 'loro-crdt';
import { connectSync, type SyncConnection } from './sync-bridge';
import { readWorkspace, type SceneView, type WorkspaceView } from './workspace-view';
import * as mutate from './mutations';
import type { WidgetGeometry } from './mutations';

const TREE = 'tree';
const WORKSPACE = 'workspace';

export interface WorkspaceClient {
	readonly scenes: SceneView[];
	readonly activeSceneId: string;
	readonly workspace: WorkspaceView;
	readonly canUndo: boolean;
	readonly canRedo: boolean;
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
	undo(): void;
	redo(): void;
	dispose(): void;
}

export function createWorkspaceClient(url: string): WorkspaceClient {
	const doc = new LoroDoc();
	let version = $state(0);

	const unsubscribe = doc.subscribe(() => {
		version += 1;
	});
	const sync: SyncConnection = connectSync(doc, url);

	// Local undo over THIS peer's edits only; remote merges + relay repairs are a
	// different peer and never land on the stack. A 300ms merge window coalesces
	// a burst (e.g. inspector typing) into one step; a drag commits once on
	// pointer-up so it is already a single step.
	const undo = new UndoManager(doc, { mergeInterval: 300 });

	function activeLayout() {
		const tree = doc.getTree(TREE);
		const roots = tree.roots();
		const activeLayoutId = String(doc.getMap(WORKSPACE).get('activeLayoutId') ?? '');
		return roots.find((node) => String(node.id) === activeLayoutId) ?? roots[0];
	}

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
		activate(sceneId: string) {
			const layout = activeLayout();
			if (!layout) return;
			layout.data.set('activeSceneId', sceneId);
			doc.commit();
		},
		setWidgetGeometry(id, geom) {
			mutate.setWidgetGeometry(doc, id, geom);
			doc.commit();
		},
		setWidgetZ(id, z) {
			mutate.setWidgetZ(doc, id, z);
			doc.commit();
		},
		setWidgetVisible(id, visible) {
			mutate.setWidgetVisible(doc, id, visible);
			doc.commit();
		},
		setWidgetProp(id, key, value) {
			mutate.setWidgetProp(doc, id, key, value);
			doc.commit();
		},
		createWidget(sceneId, widgetType, geom, props) {
			const id = mutate.createWidget(doc, sceneId, widgetType, geom, props);
			doc.commit();
			return id;
		},
		deleteWidget(id) {
			mutate.deleteWidget(doc, id);
			doc.commit();
		},
		moveWidgetToScene(id, sceneId) {
			mutate.moveWidgetToScene(doc, id, sceneId);
			doc.commit();
		},
		setSceneTheme(sceneId, themeId) {
			mutate.setSceneTheme(doc, sceneId, themeId);
			doc.commit();
		},
		setSceneOverride(sceneId, key, value) {
			mutate.setSceneOverride(doc, sceneId, key, value);
			doc.commit();
		},
		undo() {
			undo.undo();
		},
		redo() {
			undo.redo();
		},
		dispose() {
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
