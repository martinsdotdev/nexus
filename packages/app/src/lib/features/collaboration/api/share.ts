// Cloud-mode share/collaboration API for the Share popover: list members, change roles,
// invite by email, remove a member, and mint a read-only watch link (overlay token). These
// are same-origin fetches from the editor, so the browser attaches the session cookie and
// the Sec-Fetch-Site header the relay's CSRF guard checks; we only set credentials + the
// JSON content-type. Reads parse + throw on failure; mutations return the raw Response so
// the caller can branch on the status (e.g. a 404 invite means "no account with that email").

export type Role = 'owner' | 'editor' | 'viewer';

export interface Member {
	user_id: string;
	/** Display handle (email local-part), as in /auth/me. */
	display: string;
	role: Role;
}

const JSON_HEADERS = { 'content-type': 'application/json' };

export async function listMembers(workspaceId: string): Promise<Member[]> {
	const res = await fetch(`/api/workspaces/${workspaceId}/members`, { credentials: 'include' });
	if (!res.ok) throw new Error(`list members failed: ${res.status}`);
	return (await res.json()) as Member[];
}

export function inviteMember(workspaceId: string, email: string, role: Role): Promise<Response> {
	return fetch(`/api/workspaces/${workspaceId}/members`, {
		method: 'POST',
		headers: JSON_HEADERS,
		credentials: 'include',
		body: JSON.stringify({ email, role })
	});
}

export function setMemberRole(workspaceId: string, userId: string, role: Role): Promise<Response> {
	return fetch(`/api/workspaces/${workspaceId}/members/${userId}`, {
		method: 'PUT',
		headers: JSON_HEADERS,
		credentials: 'include',
		body: JSON.stringify({ role })
	});
}

export function removeMember(workspaceId: string, userId: string): Promise<Response> {
	return fetch(`/api/workspaces/${workspaceId}/members/${userId}`, {
		method: 'DELETE',
		credentials: 'include'
	});
}

export async function mintOverlayToken(workspaceId: string): Promise<string> {
	const res = await fetch(`/api/workspaces/${workspaceId}/overlay-token`, {
		method: 'POST',
		headers: JSON_HEADERS,
		credentials: 'include'
	});
	if (!res.ok) throw new Error(`mint token failed: ${res.status}`);
	return ((await res.json()) as { token: string }).token;
}

/** A minted watch link's public metadata (the secret is shown only once, at mint). */
export interface OverlayTokenSummary {
	id: string;
	created_at: string;
	revoked: boolean;
}

export async function listOverlayTokens(workspaceId: string): Promise<OverlayTokenSummary[]> {
	const res = await fetch(`/api/workspaces/${workspaceId}/overlay-tokens`, {
		credentials: 'include'
	});
	if (!res.ok) throw new Error(`list tokens failed: ${res.status}`);
	return (await res.json()) as OverlayTokenSummary[];
}

export function revokeOverlayToken(workspaceId: string, tokenId: string): Promise<Response> {
	return fetch(`/api/workspaces/${workspaceId}/overlay-token/${tokenId}`, {
		method: 'DELETE',
		credentials: 'include'
	});
}
