import { describe, expect, test } from 'vitest';
import { LoroDoc, UndoManager } from 'loro-crdt';
import {
	activateLayout,
	archiveLayout,
	createLayout,
	createWidget,
	deleteWidget,
	duplicateLayout,
	moveWidgetToScene,
	setSceneOverride,
	setSceneTheme,
	setWidgetGeometry,
	setWidgetProp,
	setWidgetVisible,
	setWidgetZ
} from './mutations';
import { readWorkspace } from './workspace-view';

// One layout, two empty scenes (live + brb). Widgets are added by the tests via
// createWidget (the mutation under test). Returns the scene id strings.
function buildFixture() {
	const doc = new LoroDoc();
	const tree = doc.getTree('tree');
	const layout = tree.createNode();
	layout.data.set('type', 'layout');
	layout.data.set('name', 'Main');
	layout.data.set('status', 'active');
	const scenes: Record<string, string> = {};
	for (const [kind, theme] of [
		['live', 'cozy'],
		['brb', 'editorial']
	]) {
		const scene = layout.createNode();
		scene.data.set('type', 'scene');
		scene.data.set('kind', kind);
		scene.data.set('name', kind);
		scene.data.set('themeId', theme);
		scenes[kind] = String(scene.id);
	}
	layout.data.set('activeSceneId', scenes.live);
	doc.getMap('workspace').set('activeLayoutId', String(layout.id));
	doc.commit();
	return { doc, scenes };
}

const sceneByKind = (doc: LoroDoc, kind: string) =>
	readWorkspace(doc).scenes.find((scene) => scene.kind === kind)!;

describe('layouts', () => {
	test('createLayout adds an active layout with a starter scene', () => {
		const { doc } = buildFixture();
		const id = createLayout(doc, 'Vertical');
		doc.commit();
		const layout = readWorkspace(doc).layouts.find((l) => l.id === id)!;
		expect(layout.name).toBe('Vertical');
		expect(layout.status).toBe('active');
		expect(layout.scenes).toHaveLength(1);
		expect(layout.activeSceneId).toBe(layout.scenes[0].id);
	});

	test('activateLayout points the workspace at a layout', () => {
		const { doc } = buildFixture();
		const id = createLayout(doc, 'Vertical');
		activateLayout(doc, id);
		doc.commit();
		expect(readWorkspace(doc).activeLayoutId).toBe(id);
	});

	test('duplicateLayout deep-copies scenes and widgets with fresh ids', () => {
		const { doc, scenes } = buildFixture();
		createWidget(doc, scenes.live, 'goal-bar', { x: 1, y: 2, w: 3, h: 4, z: 5 }, { label: 'G' });
		doc.commit();
		const sourceId = readWorkspace(doc).layouts[0].id;
		const copyId = duplicateLayout(doc, sourceId, 'Main copy')!;
		doc.commit();
		const copy = readWorkspace(doc).layouts.find((l) => l.id === copyId)!;
		expect(copy.name).toBe('Main copy');
		expect(copy.scenes).toHaveLength(2);
		const liveCopy = copy.scenes.find((s) => s.kind === 'live')!;
		expect(liveCopy.widgets).toHaveLength(1);
		expect(liveCopy.widgets[0].props.label).toBe('G');
		expect(copy.id).not.toBe(sourceId);
		expect(liveCopy.id).not.toBe(scenes.live);
		expect(copy.activeSceneId).toBe(liveCopy.id);
	});

	test('archiveLayout marks the layout archived', () => {
		const { doc } = buildFixture();
		const id = createLayout(doc, 'Temp');
		archiveLayout(doc, id);
		doc.commit();
		expect(readWorkspace(doc).layouts.find((l) => l.id === id)!.status).toBe('archived');
	});
});

const GEOM = { x: 10, y: 20, w: 100, h: 50, z: 3 };

describe('widget mutations', () => {
	test('createWidget adds a child widget with geometry and default props', () => {
		const { doc, scenes } = buildFixture();
		const id = createWidget(doc, scenes.live, 'stream-info', GEOM, {
			title: 'My Stream',
			game: 'Just Chatting'
		});
		doc.commit();

		const live = sceneByKind(doc, 'live');
		expect(live.widgets).toHaveLength(1);
		expect(id).toBe(live.widgets[0].id);
		expect(live.widgets[0].widgetType).toBe('stream-info');
		expect(live.widgets[0].x).toBe(10);
		expect(live.widgets[0].z).toBe(3);
		expect(live.widgets[0].props.title).toBe('My Stream');
	});

	test('setWidgetGeometry updates only the given fields', () => {
		const { doc, scenes } = buildFixture();
		const id = createWidget(doc, scenes.live, 'goal-bar', GEOM, {})!;
		doc.commit();
		setWidgetGeometry(doc, id, { x: 200, w: 640 });
		doc.commit();

		const w = sceneByKind(doc, 'live').widgets[0];
		expect(w.x).toBe(200);
		expect(w.w).toBe(640);
		expect(w.y).toBe(20); // unchanged
	});

	test('setWidgetProp, setWidgetZ, setWidgetVisible round-trip', () => {
		const { doc, scenes } = buildFixture();
		const id = createWidget(doc, scenes.live, 'goal-bar', GEOM, {})!;
		doc.commit();
		setWidgetProp(doc, id, 'label', 'Sub Goal');
		setWidgetZ(doc, id, 9);
		setWidgetVisible(doc, id, false);
		doc.commit();

		const w = sceneByKind(doc, 'live').widgets[0];
		expect(w.props.label).toBe('Sub Goal');
		expect(w.z).toBe(9);
		expect(w.visible).toBe(false);
	});

	test('deleteWidget removes the widget', () => {
		const { doc, scenes } = buildFixture();
		const id = createWidget(doc, scenes.live, 'goal-bar', GEOM, {})!;
		doc.commit();
		deleteWidget(doc, id);
		doc.commit();
		expect(sceneByKind(doc, 'live').widgets).toHaveLength(0);
	});

	test('moveWidgetToScene reparents the widget', () => {
		const { doc, scenes } = buildFixture();
		const id = createWidget(doc, scenes.live, 'goal-bar', GEOM, {})!;
		doc.commit();
		moveWidgetToScene(doc, id, scenes.brb);
		doc.commit();
		expect(sceneByKind(doc, 'live').widgets).toHaveLength(0);
		expect(sceneByKind(doc, 'brb').widgets).toHaveLength(1);
	});
});

describe('scene mutations', () => {
	test('setSceneTheme and setSceneOverride round-trip', () => {
		const { doc, scenes } = buildFixture();
		setSceneTheme(doc, scenes.live, 'cyber');
		setSceneOverride(doc, scenes.live, 'overridesAccent', 'oklch(70% 0.2 30)');
		doc.commit();

		const live = sceneByKind(doc, 'live');
		expect(live.themeId).toBe('cyber');
		expect(live.overridesAccent).toBe('oklch(70% 0.2 30)');
	});
});

describe('undo via UndoManager', () => {
	test('undo reverts a local mutation; redo reapplies it', () => {
		const { doc, scenes } = buildFixture();
		const undo = new UndoManager(doc, { mergeInterval: 0 });

		createWidget(doc, scenes.live, 'goal-bar', GEOM, {});
		doc.commit();
		expect(sceneByKind(doc, 'live').widgets).toHaveLength(1);
		expect(undo.canUndo()).toBe(true);

		undo.undo();
		expect(sceneByKind(doc, 'live').widgets).toHaveLength(0);

		undo.redo();
		expect(sceneByKind(doc, 'live').widgets).toHaveLength(1);
	});

	test('a remote peer change is NOT undone by the local UndoManager', () => {
		const { doc, scenes } = buildFixture();
		const undo = new UndoManager(doc, { mergeInterval: 0 });

		// Local edit on this peer: live -> cyber.
		setSceneTheme(doc, scenes.live, 'cyber');
		doc.commit();

		// A second replica (distinct peer) edits a different scene and we merge it.
		const remote = new LoroDoc();
		remote.import(doc.export({ mode: 'snapshot' }));
		setSceneTheme(remote, scenes.brb, 'sticker');
		remote.commit();
		doc.import(remote.export({ mode: 'update' }));

		// Undo reverts only the local edit; the remote edit survives.
		undo.undo();
		expect(sceneByKind(doc, 'live').themeId).toBe('cozy');
		expect(sceneByKind(doc, 'brb').themeId).toBe('sticker');
	});
});
