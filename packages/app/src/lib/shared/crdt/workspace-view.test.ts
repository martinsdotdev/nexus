import { describe, expect, it } from 'vitest';
import { LoroDoc } from 'loro-crdt';
import { readWorkspace } from './workspace-view';

// A fixture mirroring nexus-core's curated default. (Real clients import the
// relay's snapshot rather than constructing this; here it is just a test input.)
function buildFixture(): LoroDoc {
	const doc = new LoroDoc();
	const tree = doc.getTree('tree');

	const layout = tree.createNode();
	layout.data.set('type', 'layout');
	layout.data.set('name', 'Main');
	layout.data.set('status', 'active');

	let firstSceneId = '';
	for (const kind of ['live', 'starting_soon', 'brb', 'ending']) {
		const scene = layout.createNode();
		scene.data.set('type', 'scene');
		scene.data.set('kind', kind);
		scene.data.set('name', kind);
		if (firstSceneId === '') firstSceneId = String(scene.id);
	}
	layout.data.set('activeSceneId', firstSceneId);

	const workspace = doc.getMap('workspace');
	workspace.set('activeLayoutId', String(layout.id));
	workspace.set('schemaVersion', 1);

	doc.commit();
	return doc;
}

describe('readWorkspace', () => {
	it('projects the curated default structure', () => {
		const view = readWorkspace(buildFixture());

		expect(view.scenes.map((scene) => scene.kind)).toEqual([
			'live',
			'starting_soon',
			'brb',
			'ending'
		]);
		expect(view.scenes[0].name).toBe('live');
		expect(view.activeSceneId).toBe(view.scenes[0].id);
		expect(view.activeLayoutId).not.toBe('');
	});
});
