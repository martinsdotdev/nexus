import { describe, expect, it } from 'vitest';
import { createPresence, type Presence } from './presence';

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

// Two presences wired so each ships its local updates straight into the other (standing
// in for the relay's opaque forwarding).
function connected(): { a: Presence; b: Presence } {
	const a = createPresence({ id: 'alice', name: 'Alice' }, (bytes) => b.apply(bytes));
	const b = createPresence({ id: 'bob', name: 'Bob' }, (bytes) => a.apply(bytes));
	return { a, b };
}

describe('presence', () => {
	it('lists a remote peer by identity and excludes self', async () => {
		const { a, b } = connected();
		await tick();

		expect(b.remotePeers().map((p) => p.user)).toEqual([{ id: 'alice', name: 'Alice' }]);
		expect(a.remotePeers().map((p) => p.user.id)).toEqual(['bob']);

		a.destroy();
		b.destroy();
	});

	it('shares cursor and selection', async () => {
		const { a, b } = connected();
		await tick();

		a.setCursor(120, 45);
		a.setSelection(['widget-1']);
		await tick();

		const alice = b.remotePeers().find((p) => p.user.id === 'alice');
		expect(alice?.cursor).toEqual({ x: 120, y: 45 });
		expect(alice?.selection).toEqual(['widget-1']);

		a.destroy();
		b.destroy();
	});

	it('notifies subscribers on remote changes but not on local ones', async () => {
		const { a, b } = connected();
		await tick();

		let notifications = 0;
		const off = b.subscribe(() => {
			notifications += 1;
		});

		// b's own cursor/selection never appears in its remotePeers() (self is excluded),
		// so a local change must not notify: a no-op re-render at best, and at worst it
		// re-enters whatever effect drove the change (Svelte effect_update_depth_exceeded).
		b.setCursor(5, 5);
		b.setSelection(['w1']);
		await tick();
		expect(notifications).toBe(0);

		// A remote peer's change does alter remotePeers(), so it must notify.
		a.setCursor(9, 9);
		await tick();
		expect(notifications).toBeGreaterThan(0);

		off();
		a.destroy();
		b.destroy();
	});

	it('de-dupes two tabs of one account into a single peer', async () => {
		const b = createPresence({ id: 'bob', name: 'Bob' }, () => {});
		const tab1 = createPresence({ id: 'alice', name: 'Alice' }, (bytes) => b.apply(bytes));
		const tab2 = createPresence({ id: 'alice', name: 'Alice' }, (bytes) => b.apply(bytes));
		await tick();

		expect(b.remotePeers().filter((p) => p.user.id === 'alice')).toHaveLength(1);

		b.destroy();
		tab1.destroy();
		tab2.destroy();
	});
});
