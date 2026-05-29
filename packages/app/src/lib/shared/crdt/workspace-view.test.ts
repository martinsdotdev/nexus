import { describe, expect, it } from 'vitest';
import { LoroDoc } from 'loro-crdt';
import { readWorkspace } from './workspace-view';

// A fixture mirroring nexus-core's curated default (one layout, themed scenes,
// widgets on the live scene). Real clients import the relay's snapshot.
function buildFixture(): LoroDoc {
	const doc = new LoroDoc();
	const tree = doc.getTree('tree');

	const layout = tree.createNode();
	layout.data.set('type', 'layout');
	layout.data.set('name', 'Main');
	layout.data.set('status', 'active');

	let liveSceneId = '';
	for (const [kind, theme] of [
		['live', 'cozy'],
		['brb', 'editorial']
	]) {
		const scene = layout.createNode();
		scene.data.set('type', 'scene');
		scene.data.set('kind', kind);
		scene.data.set('name', kind);
		scene.data.set('themeId', theme);

		if (kind === 'live') {
			liveSceneId = String(scene.id);
			const seed = (widgetType: string, x: number, extra: Record<string, unknown> = {}) => {
				const w = scene.createNode();
				w.data.set('type', 'widget');
				w.data.set('widgetType', widgetType);
				w.data.set('x', x);
				w.data.set('y', 900);
				w.data.set('w', 560);
				w.data.set('h', 96);
				w.data.set('z', 2);
				w.data.set('visible', true);
				for (const [k, v] of Object.entries(extra)) w.data.set(k, v as never);
			};
			seed('stream-info', 40, { title: 'My Stream' });
			seed('goal-bar', 620);
		}
	}
	layout.data.set('activeSceneId', liveSceneId);

	const workspace = doc.getMap('workspace');
	workspace.set('activeLayoutId', String(layout.id));
	doc.commit();
	return doc;
}

describe('readWorkspace', () => {
	it('projects scenes, themes, and widgets of the active layout', () => {
		const view = readWorkspace(buildFixture());

		expect(view.scenes.map((scene) => scene.kind)).toEqual(['live', 'brb']);
		const live = view.scenes[0];
		expect(live.themeId).toBe('cozy');
		expect(view.activeSceneId).toBe(live.id);

		expect(live.widgets.map((widget) => widget.widgetType)).toEqual(['stream-info', 'goal-bar']);
		const streamInfo = live.widgets[0];
		expect(streamInfo.x).toBe(40);
		expect(streamInfo.visible).toBe(true);
		expect(streamInfo.props.title).toBe('My Stream');
		// Structural keys are stripped from props.
		expect(streamInfo.props.type).toBeUndefined();
		expect(streamInfo.props.widgetType).toBeUndefined();
	});

	it('exposes every layout for ?layout= selection', () => {
		const view = readWorkspace(buildFixture());
		expect(view.layouts).toHaveLength(1);
		expect(view.layouts[0].id).toBe(view.activeLayoutId);
		expect(view.layouts[0].scenes.map((scene) => scene.kind)).toEqual(['live', 'brb']);
	});
});
