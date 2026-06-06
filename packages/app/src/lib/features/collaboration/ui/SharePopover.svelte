<script lang="ts">
	// The Share / collaborators panel: a popover off the titlebar that lists who is in the
	// workspace and their roles, lets the owner invite by email, change roles, and remove
	// members, and mints a read-only "watch link" (overlay token) for OBS or viewers. Cloud
	// model: editors and viewers are real accounts that sign in by email; the watch link is
	// the only no-account way in, and it is read-only. The relay enforces every permission;
	// this panel only shows the controls the caller may use.
	import { Crown, ChevronDown, UserPlus, Link, Copy, Check, Shield, X } from 'lucide-svelte';
	import type { Member, Role } from '../api/share';
	import { peerColor } from '$lib/shared/lib/peer-color';
	import { toast } from '$lib/shared/ui/toast';
	import { emailError } from '$lib/shared/lib/validators';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		members: Member[];
		/** The caller's own account id (marks their row "you"). */
		selfId: string;
		/** The caller's role, which gates the management controls. */
		myRole: Role;
		/** The minted watch-link URL, or null until one is created. */
		watchLink: string | null;
		onClose: () => void;
		/** Invite an email at a role; resolves to an error message, or null on success. */
		onInvite: (email: string, role: Role) => Promise<string | null>;
		onSetRole: (userId: string, role: Role) => void;
		onRemove: (userId: string) => void;
		onCreateWatchLink: () => void;
	}
	let {
		members,
		selfId,
		myRole,
		watchLink,
		onClose,
		onInvite,
		onSetRole,
		onRemove,
		onCreateWatchLink
	}: Props = $props();

	const canManage = $derived(myRole === 'owner');
	const canShare = $derived(myRole === 'owner' || myRole === 'editor');

	let inviteEmail = $state('');
	let inviteRole = $state<Role>('editor');
	let inviteMsg = $state<string | null>(null);
	let inviting = $state(false);
	let copied = $state(false);
	let copyTimer: ReturnType<typeof setTimeout> | undefined;

	function initials(name: string): string {
		const parts = name.trim().split(/\s+/).filter(Boolean);
		if (parts.length === 0) return '?';
		if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
		return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
	}

	function roleLabel(role: Role): string {
		return role === 'owner' ? 'Host' : role === 'editor' ? 'Can edit' : 'View only';
	}

	async function invite(event?: SubmitEvent) {
		event?.preventDefault();
		if (inviting) return;
		const email = inviteEmail.trim();
		const invalid = emailError(email);
		if (invalid) {
			inviteMsg = invalid;
			return;
		}
		inviting = true;
		inviteMsg = null;
		const error = await onInvite(email, inviteRole);
		inviting = false;
		if (error) {
			inviteMsg = error;
		} else {
			inviteEmail = '';
			toast.success(m['share.invite_sent']({ email }));
		}
	}

	function copyLink() {
		if (!watchLink) return;
		navigator.clipboard?.writeText(watchLink);
		copied = true;
		clearTimeout(copyTimer);
		copyTimer = setTimeout(() => (copied = false), 1500);
	}

	$effect(() => {
		const onKey = (event: KeyboardEvent) => {
			if (event.key === 'Escape') onClose();
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="scrim" onclick={onClose}></div>
<div class="pop" role="dialog" aria-label="Share workspace" data-testid="share-popover">
	<header class="head">
		<span class="title">Share this workspace</span>
		<span class="count">{members.length} {members.length === 1 ? 'person' : 'people'}</span>
	</header>

	<div class="people">
		{#each members as m (m.user_id)}
			<div class="person">
				<span class="avatar" style="--peer: {peerColor(m.user_id)};">{initials(m.display)}</span>
				<span class="person-name">
					<span class="name-row">
						{m.display}{#if m.user_id === selfId}<span class="you-tag">you</span>{/if}
					</span>
					<span class="sub">{m.role === 'owner' ? 'Host' : roleLabel(m.role)}</span>
				</span>

				{#if m.role === 'owner'}
					<span class="role"><Crown size={12} color="var(--warning)" />Host</span>
				{:else if canManage && m.user_id !== selfId}
					<span class="role-select">
						<select
							aria-label="Role for {m.display}"
							value={m.role}
							onchange={(e) => onSetRole(m.user_id, e.currentTarget.value as Role)}
						>
							<option value="editor">Can edit</option>
							<option value="viewer">View only</option>
						</select>
						<span class="caret"><ChevronDown size={12} /></span>
					</span>
					<button
						class="icon-btn"
						aria-label="Remove {m.display}"
						title="Remove"
						onclick={() => onRemove(m.user_id)}
					>
						<X size={14} />
					</button>
				{:else}
					<span class="role">{roleLabel(m.role)}</span>
				{/if}
			</div>
		{/each}
	</div>

	{#if canManage}
		<div class="section">
			<form class="invite" onsubmit={invite}>
				<input
					class="invite-input"
					type="email"
					placeholder="Invite by email"
					aria-label="Invite by email"
					aria-invalid={inviteMsg ? 'true' : undefined}
					bind:value={inviteEmail}
				/>
				<div class="seg">
					<button
						type="button"
						class:on={inviteRole === 'editor'}
						onclick={() => (inviteRole = 'editor')}
					>
						Editor
					</button>
					<button
						type="button"
						class:on={inviteRole === 'viewer'}
						onclick={() => (inviteRole = 'viewer')}
					>
						Viewer
					</button>
				</div>
				<button class="invite-btn" type="submit" aria-label="Send invite" disabled={inviting}>
					<UserPlus size={14} />
				</button>
			</form>
			{#if inviteMsg}<p class="invite-msg" role="alert">{inviteMsg}</p>{/if}
		</div>
	{/if}

	{#if canShare}
		<div class="section">
			{#if watchLink}
				<div class="link-row">
					<Link size={14} color="var(--muted-foreground)" />
					<span class="link-code">{watchLink}</span>
					<button class="copy-btn" class:copied onclick={copyLink}>
						{#if copied}<Check size={12} />Copied{:else}<Copy size={12} />Copy{/if}
					</button>
				</div>
			{:else}
				<button class="link-create" onclick={onCreateWatchLink}>
					<Link size={14} />Create a watch link
				</button>
			{/if}
			<p class="sub link-sub">A read-only link for OBS or viewers, no account needed.</p>
		</div>
	{/if}

	<footer class="foot">
		<span class="note-ico"><Shield size={13} /></span>
		<span class="note"
			>Editors and viewers sign in with their email; the watch link is read-only.</span
		>
	</footer>
</div>

<style>
	.scrim {
		position: fixed;
		inset: 0;
		z-index: 90;
	}

	.pop {
		position: absolute;
		top: calc(var(--titlebar-height) + 6px);
		right: var(--space-3);
		z-index: 91;
		width: 320px;
		background: var(--popover);
		color: var(--popover-foreground);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-popover);
		overflow: hidden;
		animation: pop-in var(--dur-normal) var(--ease-out);
	}

	@keyframes pop-in {
		from {
			transform: translateY(-6px) scale(0.98);
		}
		to {
			transform: none;
		}
	}

	.head {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-3) var(--space-2);
	}
	.title {
		flex: 1;
		font-size: var(--text-sm);
		font-weight: 600;
	}
	.count {
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		font-variant-numeric: tabular-nums;
	}

	.people {
		display: flex;
		flex-direction: column;
		gap: 2px;
		max-height: 230px;
		overflow: auto;
		padding: var(--space-1);
	}
	.person {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-md);
	}
	.person:hover {
		background: var(--accent);
	}

	.avatar {
		flex: none;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
		border-radius: 50%;
		background: var(--peer);
		color: oklch(18% 0.01 264);
		font-size: 11px;
		font-weight: 700;
	}

	.person-name {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}
	.name-row {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		font-size: var(--text-sm);
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.you-tag {
		font-size: 10px;
		font-weight: 600;
		color: var(--muted-foreground);
		background: var(--muted);
		padding: 0 5px;
		border-radius: var(--radius-sm);
	}
	.sub {
		font-size: var(--text-xs);
		color: var(--muted-foreground);
	}

	.role {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		height: 22px;
		padding: 0 6px;
		border-radius: var(--radius-sm);
		font-size: var(--text-xs);
		font-weight: 600;
		color: var(--muted-foreground);
		background: var(--muted);
		border: 1px solid var(--border-subtle);
	}

	.role-select {
		position: relative;
		display: inline-flex;
	}
	.role-select select {
		appearance: none;
		height: 24px;
		padding: 0 20px 0 8px;
		border-radius: var(--radius-sm);
		background: var(--secondary);
		color: var(--secondary-foreground);
		border: 1px solid var(--border-subtle);
		font: inherit;
		font-size: var(--text-xs);
		font-weight: 600;
		cursor: pointer;
	}
	.role-select select:hover {
		border-color: var(--border);
	}
	.caret {
		position: absolute;
		right: 5px;
		top: 50%;
		transform: translateY(-50%);
		pointer-events: none;
		color: var(--muted-foreground);
		display: inline-flex;
	}

	.icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-sm);
		color: var(--muted-foreground);
		cursor: pointer;
	}
	.icon-btn:hover {
		color: var(--destructive);
		background: var(--muted);
	}

	.section {
		padding: var(--space-2) var(--space-3);
		border-top: 1px solid var(--border-subtle);
	}

	.invite {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	.invite-input {
		flex: 1;
		min-width: 0;
		height: 28px;
		padding: 0 var(--space-2);
		background: var(--input);
		border: 1px solid var(--border);
		border-radius: var(--radius-md);
		color: var(--foreground);
		font: inherit;
		font-size: var(--text-sm);
	}
	.invite-input:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring);
	}
	.seg {
		display: inline-flex;
		background: var(--background);
		border: 1px solid var(--border);
		border-radius: var(--radius-md);
		padding: 2px;
		flex: none;
	}
	.seg button {
		height: 22px;
		padding: 0 6px;
		border-radius: var(--radius-sm);
		font-size: var(--text-xs);
		font-weight: 600;
		color: var(--muted-foreground);
		cursor: pointer;
	}
	.seg button.on {
		background: var(--secondary);
		color: var(--foreground);
		box-shadow: var(--shadow-panel);
	}
	.invite-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		flex: none;
		border-radius: var(--radius-md);
		background: var(--primary);
		color: var(--primary-foreground);
		cursor: pointer;
	}
	.invite-btn:disabled {
		opacity: 0.6;
		cursor: default;
	}
	.invite-msg {
		margin: var(--space-2) 0 0;
		font-size: var(--text-xs);
		color: var(--destructive);
	}

	.link-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2);
		background: var(--background);
		border: 1px solid var(--border);
		border-radius: var(--radius-md);
	}
	.link-code {
		flex: 1;
		min-width: 0;
		font-family: var(--font-mono);
		font-size: var(--text-xs);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.copy-btn {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		height: 26px;
		padding: 0 var(--space-2);
		flex: none;
		border-radius: var(--radius-sm);
		background: var(--secondary);
		color: var(--secondary-foreground);
		font-size: var(--text-xs);
		font-weight: 600;
		border: 1px solid var(--border-subtle);
		cursor: pointer;
	}
	.copy-btn:hover {
		background: var(--accent);
	}
	.copy-btn.copied {
		color: var(--success);
	}
	.link-create {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		height: 28px;
		padding: 0 var(--space-3);
		border-radius: var(--radius-md);
		background: var(--secondary);
		color: var(--secondary-foreground);
		border: 1px solid var(--border-subtle);
		font-size: var(--text-sm);
		font-weight: 600;
		cursor: pointer;
	}
	.link-create:hover {
		background: var(--accent);
	}
	.link-sub {
		margin: var(--space-2) 0 0;
	}

	.foot {
		display: flex;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		border-top: 1px solid var(--border-subtle);
	}
	.note-ico {
		flex: none;
		margin-top: 1px;
		color: var(--muted-foreground);
	}
	.note {
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		line-height: var(--leading-base);
	}
</style>
