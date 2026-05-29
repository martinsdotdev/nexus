// Projects the collaborative Loro document into a plain read model for the UI,
// the TS analog of nexus-core's `read_workspace`. The editor uses the active
// layout's scenes; the overlay uses `layouts` (to honor ?layout=) and each
// scene's `widgets` + `themeId` to render the composition.

import type { LoroDoc, LoroTreeNode } from 'loro-crdt';

export interface WidgetView {
	id: string;
	widgetType: string;
	x: number;
	y: number;
	w: number;
	h: number;
	z: number;
	visible: boolean;
	props: Record<string, unknown>;
}

export interface SceneView {
	id: string;
	kind: string;
	name: string;
	themeId: string;
	overridesAccent: string;
	overridesDensity: string;
	widgets: WidgetView[];
}

export interface LayoutView {
	id: string;
	name: string;
	activeSceneId: string;
	scenes: SceneView[];
}

export interface WorkspaceView {
	activeLayoutId: string;
	// Convenience accessors for the active layout (the editor reads these).
	activeSceneId: string;
	scenes: SceneView[];
	// Every layout, so the overlay can select one by ?layout=.
	layouts: LayoutView[];
}

const TREE = 'tree';
const WORKSPACE = 'workspace';
const STRUCTURAL = new Set(['type', 'widgetType', 'x', 'y', 'w', 'h', 'z', 'visible', 'locked']);

const str = (value: unknown): string => (value == null ? '' : String(value));
const num = (value: unknown): number => (typeof value === 'number' ? value : Number(value ?? 0));

function projectWidget(node: LoroTreeNode): WidgetView {
	const meta = node.data.toJSON() as Record<string, unknown>;
	const props: Record<string, unknown> = {};
	for (const [key, value] of Object.entries(meta)) {
		if (!STRUCTURAL.has(key)) props[key] = value;
	}
	return {
		id: String(node.id),
		widgetType: str(meta.widgetType),
		x: num(meta.x),
		y: num(meta.y),
		w: num(meta.w),
		h: num(meta.h),
		z: num(meta.z),
		visible: meta.visible !== false,
		props
	};
}

function projectScene(node: LoroTreeNode): SceneView {
	return {
		id: String(node.id),
		kind: str(node.data.get('kind')),
		name: str(node.data.get('name')),
		themeId: str(node.data.get('themeId')),
		overridesAccent: str(node.data.get('overridesAccent')),
		overridesDensity: str(node.data.get('overridesDensity')),
		widgets: (node.children() ?? [])
			.filter((child) => child.data.get('type') === 'widget')
			.map(projectWidget)
	};
}

function projectLayout(node: LoroTreeNode): LayoutView {
	return {
		id: String(node.id),
		name: str(node.data.get('name')),
		activeSceneId: str(node.data.get('activeSceneId')),
		scenes: (node.children() ?? [])
			.filter((child) => child.data.get('type') === 'scene')
			.map(projectScene)
	};
}

export function readWorkspace(doc: LoroDoc): WorkspaceView {
	const tree = doc.getTree(TREE);
	const workspace = doc.getMap(WORKSPACE);
	const activeLayoutId = String(workspace.get('activeLayoutId') ?? '');

	const layouts = tree.roots().map(projectLayout);
	const active = layouts.find((layout) => layout.id === activeLayoutId) ?? layouts[0];

	return {
		activeLayoutId,
		activeSceneId: active?.activeSceneId ?? '',
		scenes: active?.scenes ?? [],
		layouts
	};
}
