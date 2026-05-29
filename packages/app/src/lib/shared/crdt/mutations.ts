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

export function setSceneTheme(doc: LoroDoc, sceneId: string, themeId: string): void {
	nodeById(doc, sceneId)?.data.set('themeId', themeId);
}

export function setSceneOverride(doc: LoroDoc, sceneId: string, key: string, value: string): void {
	nodeById(doc, sceneId)?.data.set(key, value);
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
