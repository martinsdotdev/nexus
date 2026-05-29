// The editor's local-first workspace client: a Loro replica wired to the relay,
// exposed to the UI as reactive Svelte state. The local doc IS the store, a
// version counter (bumped on every doc change, local or remote) drives the
// `$derived` reads, so the UI re-renders whether an edit was made here or
// arrived from another collaborator. Replaces TanStack Query for document state
// (ADR-0005). Must be created client-side only (it touches WASM + WebSocket).

import { LoroDoc } from 'loro-crdt';
import { connectSync, type SyncConnection } from './sync-bridge';
import { readWorkspace, type SceneView } from './workspace-view';

const TREE = 'tree';
const WORKSPACE = 'workspace';

export interface WorkspaceClient {
	readonly scenes: SceneView[];
	readonly activeSceneId: string;
	activate(sceneId: string): void;
	dispose(): void;
}

export function createWorkspaceClient(url: string): WorkspaceClient {
	const doc = new LoroDoc();
	let version = $state(0);

	const unsubscribe = doc.subscribe(() => {
		version += 1;
	});
	const sync: SyncConnection = connectSync(doc, url);

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
		activate(sceneId: string) {
			const layout = activeLayout();
			if (!layout) return;
			layout.data.set('activeSceneId', sceneId);
			doc.commit();
		},
		dispose() {
			unsubscribe();
			sync.close();
		}
	};
}
