<script lang="ts">
	// The account settings page (cloud mode): manage where the account is signed in. Lists
	// the active sessions (device + last-used + IP), marks this device, and can revoke one
	// session or sign out every other device. Profile + delete-account land here next. A 401
	// means the session lapsed, so we send the user back to sign in.
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { m } from '$lib/paraglide/messages';
	import Toaster from '$lib/shared/ui/Toaster.svelte';
	import { toast } from '$lib/shared/ui/toast';

	interface Session {
		id: string;
		user_agent: string | null;
		ip: string | null;
		created_at: string;
		last_used_at: string;
		current: boolean;
	}

	let sessions = $state<Session[] | null>(null);
	let error = $state<string | null>(null);

	// Profile (display name) + the type-the-name confirm for deleting the account.
	let display = $state('');
	let nameInput = $state('');
	let saving = $state(false);
	let confirmInput = $state('');

	$effect(() => {
		void loadSessions();
		void loadMe();
	});

	async function loadMe() {
		try {
			const res = await fetch('/auth/me', { credentials: 'include' });
			if (res.ok) {
				const me = (await res.json()) as { display: string };
				display = me.display;
				nameInput = me.display;
			}
		} catch {
			// Non-fatal; the sessions load already surfaces connectivity errors.
		}
	}

	async function loadSessions() {
		try {
			const res = await fetch('/auth/sessions', { credentials: 'include' });
			if (res.status === 401) {
				goto(resolve('/login'));
				return;
			}
			if (!res.ok) {
				error = m['account.error']();
				return;
			}
			sessions = (await res.json()) as Session[];
		} catch {
			error = m['account.error']();
		}
	}

	function formatWhen(iso: string): string {
		const date = new Date(iso);
		return Number.isNaN(date.getTime()) ? iso : date.toLocaleString();
	}

	async function revoke(id: string) {
		try {
			const res = await fetch(`/auth/sessions/${id}`, {
				method: 'DELETE',
				credentials: 'include'
			});
			if (res.ok) {
				sessions = (sessions ?? []).filter((s) => s.id !== id);
				toast.success(m['account.session_revoked']());
			} else {
				toast.error(m['account.error']());
			}
		} catch {
			toast.error(m['account.error']());
		}
	}

	async function logoutOthers() {
		try {
			const res = await fetch('/auth/sessions/logout-others', {
				method: 'POST',
				credentials: 'include'
			});
			if (res.ok) {
				sessions = (sessions ?? []).filter((s) => s.current);
				toast.success(m['account.others_logged_out']());
			} else {
				toast.error(m['account.error']());
			}
		} catch {
			toast.error(m['account.error']());
		}
	}

	const hasOthers = $derived((sessions ?? []).some((s) => !s.current));

	async function saveProfile() {
		saving = true;
		try {
			const res = await fetch('/auth/profile', {
				method: 'PUT',
				headers: { 'content-type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({ display_name: nameInput.trim() })
			});
			if (res.ok) {
				await loadMe();
				toast.success(m['account.profile_saved']());
			} else {
				toast.error(m['account.error']());
			}
		} catch {
			toast.error(m['account.error']());
		} finally {
			saving = false;
		}
	}

	async function deleteAccount() {
		try {
			const res = await fetch('/auth/account', { method: 'DELETE', credentials: 'include' });
			if (res.ok) {
				goto(resolve('/login'));
			} else {
				toast.error(m['account.error']());
			}
		} catch {
			toast.error(m['account.error']());
		}
	}
</script>

<svelte:head><title>{m['account.title']()}</title></svelte:head>

<main class="account">
	<section class="card">
		<header class="head">
			<a class="back" href={resolve('/workspaces')}>{m['account.back']()}</a>
			<h1 class="title">{m['account.title']()}</h1>
		</header>

		{#if error}<p class="error" role="alert">{error}</p>{/if}

		<section class="block" aria-label={m['account.sessions_title']()}>
			<h2 class="block-title">{m['account.sessions_title']()}</h2>
			{#if sessions === null}
				<p class="muted" role="status" aria-live="polite">{m['account.loading']()}</p>
			{:else}
				<ul class="sessions">
					{#each sessions as session (session.id)}
						<li class="session">
							<div class="session-info">
								<span class="device">
									{session.user_agent ?? m['account.unknown_device']()}
									{#if session.current}<span class="badge">{m['account.this_device']()}</span>{/if}
								</span>
								<span class="meta">
									{formatWhen(session.last_used_at)}{#if session.ip}
										&middot; {session.ip}{/if}
								</span>
							</div>
							{#if !session.current}
								<button type="button" class="revoke" onclick={() => revoke(session.id)}>
									{m['account.revoke']()}
								</button>
							{/if}
						</li>
					{/each}
				</ul>
				{#if hasOthers}
					<button type="button" class="logout-others" onclick={logoutOthers}>
						{m['account.logout_others']()}
					</button>
				{/if}
			{/if}
		</section>

		<section class="block" aria-label={m['account.profile_title']()}>
			<h2 class="block-title">{m['account.profile_title']()}</h2>
			<label class="field">
				<span>{m['account.display_name_label']()}</span>
				<input bind:value={nameInput} placeholder={display} />
			</label>
			<button type="button" class="save" disabled={saving} onclick={saveProfile}>
				{saving ? m['account.saving']() : m['account.save']()}
			</button>
		</section>

		<section class="block danger-zone" aria-label={m['account.danger_title']()}>
			<h2 class="block-title danger">{m['account.danger_title']()}</h2>
			<p class="warning">{m['account.delete_warning']()}</p>
			<label class="field">
				<span>{m['account.delete_confirm_prompt']()} ({display})</span>
				<input
					aria-label={m['account.confirm_label']()}
					bind:value={confirmInput}
					placeholder={display}
				/>
			</label>
			<button
				type="button"
				class="delete-account"
				disabled={confirmInput.trim() !== display}
				onclick={deleteAccount}
			>
				{m['account.delete_account']()}
			</button>
		</section>
	</section>
</main>

<Toaster />

<style>
	.account {
		min-height: 100dvh;
		display: grid;
		place-items: start center;
		padding: var(--space-6) var(--space-5);
		background: var(--background);
		color: var(--foreground);
		font-family: var(--font-sans);
	}

	.card {
		width: 100%;
		max-width: 520px;
		padding: var(--space-6) var(--space-5);
		background: var(--card);
		border: var(--stroke-thin) solid var(--border-subtle);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-popover);
	}

	.head {
		margin-bottom: var(--space-5);
	}

	.back {
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		text-decoration: none;
	}

	.back:hover {
		color: var(--foreground);
	}

	.title {
		margin: var(--space-2) 0 0;
		font-size: var(--text-xl);
		font-weight: 600;
		line-height: var(--leading-tight);
	}

	.block {
		margin-top: var(--space-5);
	}

	.block-title {
		margin: 0 0 var(--space-3);
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--foreground);
	}

	.muted {
		color: var(--muted-foreground);
		font-size: var(--text-sm);
	}

	.sessions {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		margin: 0 0 var(--space-3);
	}

	.session {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
		padding: var(--space-3);
		border-radius: var(--radius-md);
		background: var(--muted);
	}

	.session-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.device {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--text-sm);
		color: var(--foreground);
	}

	.badge {
		font-size: var(--text-xs);
		font-weight: 600;
		color: var(--primary);
		background: var(--accent);
		padding: 1px var(--space-1);
		border-radius: var(--radius-sm);
	}

	.meta {
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.revoke {
		flex: none;
		font: inherit;
		font-size: var(--text-xs);
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-md);
		border: var(--stroke-thin) solid var(--border);
		background: var(--secondary);
		color: var(--secondary-foreground);
		cursor: pointer;
	}

	.logout-others {
		font: inherit;
		font-size: var(--text-sm);
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-md);
		border: var(--stroke-thin) solid var(--border);
		background: var(--secondary);
		color: var(--secondary-foreground);
		cursor: pointer;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		margin-bottom: var(--space-3);
	}

	.field input {
		padding: var(--space-2);
		border-radius: var(--radius-md);
		border: var(--stroke-thin) solid var(--border);
		background: var(--input);
		color: var(--foreground);
		font: inherit;
		font-size: var(--text-sm);
	}

	.field input:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	.save {
		font: inherit;
		font-size: var(--text-sm);
		font-weight: 600;
		padding: var(--space-2) var(--space-4);
		border-radius: var(--radius-md);
		background: var(--primary);
		color: var(--primary-foreground);
		cursor: pointer;
	}

	.save:disabled {
		opacity: 0.6;
		cursor: default;
	}

	.danger-zone {
		border: var(--stroke-thin) solid var(--destructive);
		border-radius: var(--radius-md);
		padding: var(--space-4);
	}

	.block-title.danger {
		color: var(--destructive);
	}

	.warning {
		margin: 0 0 var(--space-3);
		font-size: var(--text-sm);
		color: var(--muted-foreground);
	}

	.delete-account {
		font: inherit;
		font-size: var(--text-sm);
		font-weight: 600;
		padding: var(--space-2) var(--space-4);
		border-radius: var(--radius-md);
		border: var(--stroke-thin) solid var(--destructive);
		background: var(--destructive);
		color: var(--primary-foreground);
		cursor: pointer;
	}

	.delete-account:disabled {
		opacity: 0.5;
		cursor: default;
		background: transparent;
		color: var(--destructive);
	}

	.error {
		color: var(--destructive);
		font-size: var(--text-sm);
		margin: 0 0 var(--space-3);
	}
</style>
