import { afterEach, expect, test, vi } from 'vitest';
import { fetchSelf } from './me';

afterEach(() => vi.unstubAllGlobals());

test('maps /auth/me to an identity on success', async () => {
	vi.stubGlobal(
		'fetch',
		async () =>
			new Response(JSON.stringify({ user_id: 'u1', display: 'mara' }), {
				status: 200,
				headers: { 'content-type': 'application/json' }
			})
	);
	expect(await fetchSelf()).toEqual({ id: 'u1', name: 'mara' });
});

test('returns undefined when signed out / in local mode (non-ok)', async () => {
	vi.stubGlobal('fetch', async () => new Response(null, { status: 401 }));
	expect(await fetchSelf()).toBeUndefined();
});

test('returns undefined when the request throws', async () => {
	vi.stubGlobal('fetch', async () => {
		throw new Error('network');
	});
	expect(await fetchSelf()).toBeUndefined();
});
