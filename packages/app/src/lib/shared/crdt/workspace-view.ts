// Projects the collaborative Loro document into a plain read model for the UI,
// the TS analog of nexus-core's `read_workspace`. The skeleton surfaces the
// active layout's scenes and which one is active; SceneStrip renders from this.

import type { LoroDoc } from 'loro-crdt';

export interface SceneView {
	id: string;
	kind: string;
	name: string;
}

export interface WorkspaceView {
	activeLayoutId: string;
	activeSceneId: string;
	scenes: SceneView[];
}

const TREE = 'tree';
const WORKSPACE = 'workspace';

export function readWorkspace(doc: LoroDoc): WorkspaceView {
	const tree = doc.getTree(TREE);
	const workspace = doc.getMap(WORKSPACE);
	const activeLayoutId = String(workspace.get('activeLayoutId') ?? '');

	const roots = tree.roots();
	const layout = roots.find((node) => String(node.id) === activeLayoutId) ?? roots[0];
	if (!layout) {
		return { activeLayoutId, activeSceneId: '', scenes: [] };
	}

	const scenes: SceneView[] = (layout.children() ?? []).map((node) => ({
		id: String(node.id),
		kind: String(node.data.get('kind') ?? ''),
		name: String(node.data.get('name') ?? '')
	}));

	return {
		activeLayoutId,
		activeSceneId: String(layout.data.get('activeSceneId') ?? ''),
		scenes
	};
}
