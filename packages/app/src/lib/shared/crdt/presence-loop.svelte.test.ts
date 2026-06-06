// Regression: the editor broadcasts its selection from an $effect (edit/+page.svelte), which
// calls client.setSelection -> presence publish. If publishing notifies presence subscribers
// on local changes, the version-counter bump reads+writes state inside that effect and Svelte
// throws effect_update_depth_exceeded, crashing the editor the moment presence is enabled
// (the two-browser presence e2e first surfaced this). Presence must stay quiet on local writes.
import { test, expect } from 'vitest';
import { flushSync } from 'svelte';
import { createWorkspaceClient } from './client.svelte';

test('broadcasting selection from an effect does not loop', () => {
	const cleanup = $effect.root(() => {
		const client = createWorkspaceClient('ws://127.0.0.1:9/sync', {
			identity: { id: 'u1', name: 'alice' }
		});
		let selectedId = $state<string | null>(null);
		$effect(() => {
			client.setSelection(selectedId ? [selectedId] : []);
		});
		// Drive a change through the effect, like clicking a widget then clearing.
		selectedId = 'widget-1';
		flushSync();
		selectedId = null;
		flushSync();
	});
	flushSync();
	cleanup();
	expect(true).toBe(true);
});
