// Pure write helpers over the workspace Loro document: the imperative shell's
// vocabulary for editor operations. Each operates on a passed LoroDoc and does
// NOT commit, the reactive client wraps `mutation + doc.commit()` so one
// interaction becomes one undoable step. No reactivity, no I/O: headless-
// testable. Widget default props are supplied by the caller (an upper FSD layer
// that may read entities/widget), keeping this shared module entity-free.

import { LoroMap } from 'loro-crdt';
import type { LoroDoc, LoroTreeNode, TreeID } from 'loro-crdt';
import type { ThemeTokens } from './workspace-view';

const TREE = 'tree';
const THEMES = 'themes';
const WORKSPACE = 'workspace';

function nodeById(doc: LoroDoc, id: string): LoroTreeNode | undefined {
	return doc.getTree(TREE).getNodeByID(id as TreeID);
}

export interface WidgetGeometry {
	x: number;
	y: number;
	w: number;
	h: number;
	z: number;
}

const GEOMETRY_KEYS = ['x', 'y', 'w', 'h', 'z'] as const;

/** Set any subset of a widget's geometry (used by drag/resize and the inspector). */
export function setWidgetGeometry(doc: LoroDoc, id: string, geom: Partial<WidgetGeometry>): void {
	const node = nodeById(doc, id);
	if (!node) return;
	for (const key of GEOMETRY_KEYS) {
		const value = geom[key];
		if (value !== undefined) node.data.set(key, value);
	}
}

export function setWidgetZ(doc: LoroDoc, id: string, z: number): void {
	nodeById(doc, id)?.data.set('z', z);
}

export function setWidgetVisible(doc: LoroDoc, id: string, visible: boolean): void {
	nodeById(doc, id)?.data.set('visible', visible);
}

export function setWidgetProp(doc: LoroDoc, id: string, key: string, value: unknown): void {
	nodeById(doc, id)?.data.set(key, value as never);
}

/** Create a widget node under a scene. Loro TreeIDs are conflict-free, so a
 *  client may create nodes safely (trap T1 concerns only the curated default). */
export function createWidget(
	doc: LoroDoc,
	sceneId: string,
	widgetType: string,
	geom: WidgetGeometry,
	props: Record<string, unknown>
): string | undefined {
	const scene = nodeById(doc, sceneId);
	if (!scene) return undefined;
	const node = scene.createNode();
	node.data.set('type', 'widget');
	node.data.set('widgetType', widgetType);
	node.data.set('x', geom.x);
	node.data.set('y', geom.y);
	node.data.set('w', geom.w);
	node.data.set('h', geom.h);
	node.data.set('z', geom.z);
	node.data.set('visible', true);
	for (const [key, value] of Object.entries(props)) node.data.set(key, value as never);
	return String(node.id);
}

export function deleteWidget(doc: LoroDoc, id: string): void {
	doc.getTree(TREE).delete(id as TreeID);
}

export function moveWidgetToScene(doc: LoroDoc, id: string, sceneId: string): void {
	doc.getTree(TREE).move(id as TreeID, sceneId as TreeID);
}

// --- Layouts (each is a tree root holding scenes -> widgets) ------------------
// A layout is a root node of the tree; the active one is named by the workspace
// map's `activeLayoutId`. Clients may create layouts freely: Loro TreeIDs are
// conflict-free (trap T1 concerns only the relay-seeded curated default).

/** Copy a node's flat meta (type, geometry, props, ...) onto another node. */
function copyMeta(from: LoroTreeNode, to: LoroTreeNode): void {
	for (const [key, value] of Object.entries(from.data.toJSON() as Record<string, unknown>)) {
		to.data.set(key, value as never);
	}
}

/** Create a new layout with one starter scene; returns the new layout id. */
export function createLayout(doc: LoroDoc, name: string): string {
	const layout = doc.getTree(TREE).createNode();
	layout.data.set('type', 'layout');
	layout.data.set('name', name);
	layout.data.set('status', 'active');
	const scene = layout.createNode();
	scene.data.set('type', 'scene');
	scene.data.set('kind', 'live');
	scene.data.set('name', 'live');
	scene.data.set('themeId', 'cozy');
	layout.data.set('activeSceneId', String(scene.id));
	return String(layout.id);
}

/** Deep-copy a layout (its scenes + their widgets) into a new one; returns its id. */
export function duplicateLayout(doc: LoroDoc, id: string, name: string): string | undefined {
	const source = nodeById(doc, id);
	if (!source) return undefined;
	const layout = doc.getTree(TREE).createNode();
	layout.data.set('type', 'layout');
	layout.data.set('name', name);
	layout.data.set('status', 'active');
	const sourceActive = String(source.data.get('activeSceneId') ?? '');
	let activeSceneId = '';
	for (const sceneNode of source.children() ?? []) {
		if (sceneNode.data.get('type') !== 'scene') continue;
		const scene = layout.createNode();
		copyMeta(sceneNode, scene);
		for (const widgetNode of sceneNode.children() ?? []) {
			if (widgetNode.data.get('type') === 'widget') copyMeta(widgetNode, scene.createNode());
		}
		if (String(sceneNode.id) === sourceActive) activeSceneId = String(scene.id);
	}
	layout.data.set('activeSceneId', activeSceneId || String((layout.children() ?? [])[0]?.id ?? ''));
	return String(layout.id);
}

export function renameLayout(doc: LoroDoc, id: string, name: string): void {
	nodeById(doc, id)?.data.set('name', name);
}

/** Archive a layout: hide it from the switcher and blank its active scene (the relay's
 *  validator requires an archived layout to hold none, so do it here to match). */
export function archiveLayout(doc: LoroDoc, id: string): void {
	const node = nodeById(doc, id);
	if (!node) return;
	node.data.set('status', 'archived');
	node.data.set('activeSceneId', '');
}

/** Point the workspace at a layout (the editor + overlay project the active one). */
export function activateLayout(doc: LoroDoc, id: string): void {
	doc.getMap(WORKSPACE).set('activeLayoutId', id);
}

/** Delete a layout (and its scenes + widgets). Refuses to remove the workspace's only
 *  layout, and re-points `activeLayoutId` to a surviving layout (an active one when
 *  possible) first if the deleted layout was active, so the pointer never dangles. */
export function deleteLayout(doc: LoroDoc, id: string): void {
	const tree = doc.getTree(TREE);
	const survivors = tree.roots().filter((node) => String(node.id) !== id);
	if (survivors.length === 0) return; // a workspace keeps at least one layout
	const workspace = doc.getMap(WORKSPACE);
	if (String(workspace.get('activeLayoutId')) === id) {
		const next = survivors.find((node) => node.data.get('status') !== 'archived') ?? survivors[0];
		workspace.set('activeLayoutId', String(next.id));
	}
	tree.delete(id as TreeID);
}

export function setSceneTheme(doc: LoroDoc, sceneId: string, themeId: string): void {
	nodeById(doc, sceneId)?.data.set('themeId', themeId);
}

export function setSceneOverride(doc: LoroDoc, sceneId: string, key: string, value: string): void {
	nodeById(doc, sceneId)?.data.set(key, value);
}

/** Create a freeform scene under a layout; returns its id. A `custom`-kind scene
 *  with the default theme, appended after the layout's existing scenes. */
export function createScene(doc: LoroDoc, layoutId: string, name: string): string | undefined {
	const layout = nodeById(doc, layoutId);
	if (!layout) return undefined;
	const scene = layout.createNode();
	scene.data.set('type', 'scene');
	scene.data.set('kind', 'custom');
	scene.data.set('name', name);
	scene.data.set('themeId', 'cozy');
	return String(scene.id);
}

export function renameScene(doc: LoroDoc, sceneId: string, name: string): void {
	nodeById(doc, sceneId)?.data.set('name', name);
}

/** Delete a scene. Refuses to remove a layout's only scene (a layout always keeps
 *  one), and re-points the layout's `activeSceneId` to a surviving sibling first
 *  when the deleted scene was active, so the active pointer never dangles. */
export function deleteScene(doc: LoroDoc, sceneId: string): void {
	const tree = doc.getTree(TREE);
	const node = nodeById(doc, sceneId);
	const layout = node?.parent();
	if (!node || !layout) return;
	const siblings = (layout.children() ?? []).filter((child) => child.data.get('type') === 'scene');
	if (siblings.length <= 1) return; // a layout keeps at least one scene
	if (String(layout.data.get('activeSceneId')) === sceneId) {
		const survivor = siblings.find((scene) => String(scene.id) !== sceneId);
		if (survivor) layout.data.set('activeSceneId', String(survivor.id));
	}
	tree.delete(sceneId as TreeID);
}

/** Deep-copy a scene (its meta + child widgets) into a new sibling; returns its id. */
export function duplicateScene(doc: LoroDoc, sceneId: string, name: string): string | undefined {
	const source = nodeById(doc, sceneId);
	const layout = source?.parent();
	if (!source || !layout) return undefined;
	const scene = layout.createNode();
	copyMeta(source, scene);
	scene.data.set('name', name);
	for (const widget of source.children() ?? []) {
		if (widget.data.get('type') === 'widget') copyMeta(widget, scene.createNode());
	}
	return String(scene.id);
}

/** Move a scene to a new index among its layout's scenes (strip order is tree order). */
export function reorderScene(doc: LoroDoc, sceneId: string, index: number): void {
	const tree = doc.getTree(TREE);
	const node = nodeById(doc, sceneId);
	const layout = node?.parent();
	if (!node || !layout) return;
	tree.move(sceneId as TreeID, layout.id, index);
}

// --- Theme registry (root "themes" map of nested theme maps) -----------------
// The relay seeds the built-ins here as `protected` (ADR-0007); user themes are
// added with caller-minted ids (unique, never names) so two themes named the
// same never collide on merge (trap T1). Token values are literals or
// `link:<token>`.

function themeContainer(doc: LoroDoc, id: string): LoroMap | undefined {
	const container = doc.getMap(THEMES).get(id);
	return container instanceof LoroMap ? container : undefined;
}

/** True when `id` names a protected (built-in) theme: its tokens are the
 *  link-free literal floor a derived theme may inherit from. */
function isProtected(doc: LoroDoc, id: string): boolean {
	return themeContainer(doc, id)?.get('protected') === true;
}

export function createTheme(
	doc: LoroDoc,
	id: string,
	name: string,
	base: string,
	tokens: ThemeTokens
): void {
	const theme = doc.getMap(THEMES).setContainer(id, new LoroMap());
	theme.set('name', name);
	// A theme may only derive from a protected built-in (whose literals are a
	// guaranteed-resolvable inheritance floor); any other base falls back to the
	// default theme at resolve time. This keeps base single-level and cycle-free.
	theme.set('base', isProtected(doc, base) ? base : '');
	// User themes are never protected; only the relay seeds protected built-ins.
	theme.set('protected', false);
	const tokenMap = theme.setContainer('tokens', new LoroMap());
	for (const [key, value] of Object.entries(tokens)) tokenMap.set(key, value);
}

export function renameTheme(doc: LoroDoc, id: string, name: string): void {
	themeContainer(doc, id)?.set('name', name);
}

/** Set one token's value (a literal or a `link:<token>` reference). A protected
 *  (built-in) theme accepts literals only, never a `link:` value, so it stays a
 *  fully-resolvable floor for the resolver and for any theme that derives from it. */
export function setThemeToken(doc: LoroDoc, id: string, token: string, value: string): void {
	const container = themeContainer(doc, id);
	if (!container) return;
	if (container.get('protected') === true && value.startsWith('link:')) return;
	const tokens = container.get('tokens');
	if (tokens instanceof LoroMap) tokens.set(token, value);
}

/** Delete a user theme. Protected (built-in) themes are kept so the default
 *  fallback always exists; the builder also hides Delete for them. */
export function deleteTheme(doc: LoroDoc, id: string): void {
	if (themeContainer(doc, id)?.get('protected') === true) return;
	doc.getMap(THEMES).delete(id);
}

/** Read a theme out to a plain JSON value (for export); null if it is gone. */
export function exportTheme(
	doc: LoroDoc,
	id: string
): { name: string; base: string; tokens: ThemeTokens } | null {
	const theme = themeContainer(doc, id);
	if (!theme) return null;
	const tokens = theme.get('tokens');
	return {
		name: String(theme.get('name') ?? ''),
		base: String(theme.get('base') ?? ''),
		tokens: tokens instanceof LoroMap ? (tokens.toJSON() as ThemeTokens) : {}
	};
}
