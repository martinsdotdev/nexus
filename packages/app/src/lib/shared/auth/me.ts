// The signed-in account, fetched from `/auth/me`. Used to stamp presence and identify the
// caller for collaboration. In local mode `/auth/me` is not a route, so this resolves to no
// identity (presence + sharing stay off). Client-only (it calls `fetch`).
import type { PeerIdentity } from '$lib/shared/crdt/presence';

export async function fetchSelf(): Promise<PeerIdentity | undefined> {
	try {
		const res = await fetch('/auth/me', { credentials: 'include' });
		if (!res.ok) return undefined;
		const me = (await res.json()) as { user_id: string; display: string };
		return { id: me.user_id, name: me.display };
	} catch {
		return undefined;
	}
}
