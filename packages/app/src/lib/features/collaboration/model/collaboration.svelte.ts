// The collaboration store (cloud mode): the workspace's members + roles and the actions that
// mutate them (invite by email, change a role, remove a member, mint the read-only watch link).
// It owns the orchestration that used to live inline in the editor page — load + cache members,
// derive the caller's own role and whether the workspace is actively shared. Construct it per
// workspace once the caller is known; the Share popover and the page read its outputs. Created
// client-side only (it touches `fetch` + `location`).

import {
	listMembers,
	inviteMember,
	setMemberRole,
	removeMember,
	mintOverlayToken,
	type Member,
	type Role
} from '../api/share';

export interface Collaboration {
	/** Everyone in the workspace, with their role. */
	readonly members: Member[];
	/** The caller's own role (viewer until members load). */
	readonly myRole: Role;
	/** Actively shared: others are present, or a watch link has been minted. */
	readonly shareLive: boolean;
	/** The minted read-only watch-link URL, or null until one is created. */
	readonly watchLink: string | null;
	/** (Re)load the member list; swallows failures, leaving the list empty. */
	load(): Promise<void>;
	/** Invite an email at a role; resolves to an error message, or null on success. */
	invite(email: string, role: Role): Promise<string | null>;
	setRole(userId: string, role: Role): Promise<void>;
	remove(userId: string): Promise<void>;
	createWatchLink(): Promise<void>;
}

export function createCollaboration(workspaceId: string, selfId: string): Collaboration {
	let members = $state<Member[]>([]);
	let watchLink = $state<string | null>(null);

	const myRole = $derived<Role>(
		members.find((member) => member.user_id === selfId)?.role ?? 'viewer'
	);
	const shareLive = $derived(members.length > 1 || watchLink !== null);

	async function load() {
		try {
			members = await listMembers(workspaceId);
		} catch {
			members = [];
		}
	}

	return {
		get members() {
			return members;
		},
		get myRole() {
			return myRole;
		},
		get shareLive() {
			return shareLive;
		},
		get watchLink() {
			return watchLink;
		},
		load,
		async invite(email, role) {
			const res = await inviteMember(workspaceId, email, role);
			if (res.status === 404) return 'No Nexus account uses that email yet.';
			if (!res.ok) return 'Could not send the invite.';
			await load();
			return null;
		},
		async setRole(userId, role) {
			await setMemberRole(workspaceId, userId, role);
			await load();
		},
		async remove(userId) {
			await removeMember(workspaceId, userId);
			await load();
		},
		async createWatchLink() {
			try {
				const token = await mintOverlayToken(workspaceId);
				watchLink = `${location.origin}/overlay?token=${token}`;
			} catch {
				// Leave the link unset; the panel keeps offering to create one.
			}
		}
	};
}
