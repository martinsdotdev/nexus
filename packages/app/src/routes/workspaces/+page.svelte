<script lang="ts">
	// The workspace picker (cloud mode): lists the workspaces the signed-in user belongs
	// to and opens one in the editor (/edit?workspace=<id>), or creates a new one. A 401
	// means the session lapsed, so we send the user back to sign in.
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { m } from '$lib/paraglide/messages';
	import { createForm } from '@tanstack/svelte-form';
	import Field from '$lib/shared/ui/Field.svelte';
	import Toaster from '$lib/shared/ui/Toaster.svelte';
	import { toast } from '$lib/shared/ui/toast';
	import { workspaceNameError } from '$lib/shared/lib/validators';

	interface Workspace {
		id: string;
		name: string;
		role: string;
	}

	let workspaces = $state<Workspace[] | null>(null);
	let error = $state<string | null>(null);

	$effect(() => {
		void load();
	});

	async function load() {
		try {
			const res = await fetch('/api/workspaces', { credentials: 'include' });
			if (res.status === 401) {
				goto(resolve('/login'));
				return;
			}
			if (!res.ok) {
				error = m['workspaces.error_load']();
				return;
			}
			workspaces = (await res.json()) as Workspace[];
		} catch {
			error = m['workspaces.error_network']();
		}
	}

	function openWorkspace(id: string) {
		// resolve() handles the base path; the workspace rides as a query the rule cannot
		// model, so the navigation is sound but the lint must be waived here.
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		goto(`${resolve('/edit')}?workspace=${id}`);
	}

	const nameForm = createForm(() => ({
		defaultValues: { name: '' },
		onSubmit: async ({ value }) => {
			error = null;
			try {
				const res = await fetch('/api/workspaces', {
					method: 'POST',
					headers: { 'content-type': 'application/json' },
					credentials: 'include',
					body: JSON.stringify({ name: value.name.trim() })
				});
				if (res.ok) {
					const { id } = (await res.json()) as { id: string };
					toast.success(m['workspaces.created']());
					openWorkspace(id);
				} else {
					toast.error(m['workspaces.error_load']());
				}
			} catch {
				toast.error(m['workspaces.error_network']());
			}
		}
	}));
</script>

<svelte:head><title>{m['workspaces.title']()}</title></svelte:head>

<main class="picker">
	<section class="card">
		<header class="head">
			<span class="mark">{m['app.name']()}</span>
			<h1 class="title">{m['workspaces.title']()}</h1>
		</header>

		{#if error}<p class="error" role="alert">{error}</p>{/if}

		{#if workspaces === null}
			<p class="muted" role="status" aria-live="polite">{m['workspaces.loading']()}</p>
		{:else if workspaces.length === 0}
			<p class="muted">{m['workspaces.empty']()}</p>
		{:else}
			<ul class="list">
				{#each workspaces as ws (ws.id)}
					<li>
						<button class="row" onclick={() => openWorkspace(ws.id)}>
							<span class="name">{ws.name}</span>
							<span class="role">{ws.role}</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}

		<form
			class="create-form"
			onsubmit={(event) => {
				event.preventDefault();
				nameForm.handleSubmit();
			}}
		>
			<nameForm.Field
				name="name"
				validators={{
					onBlur: ({ value }) => workspaceNameError(value),
					onSubmit: ({ value }) => workspaceNameError(value)
				}}
			>
				{#snippet children(field)}
					<Field
						label={m['workspaces.name_label']()}
						placeholder={m['workspaces.name_placeholder']()}
						value={field.state.value}
						oninput={(v) => field.handleChange(v)}
						onblur={() => field.handleBlur()}
						error={field.state.meta.errors[0]}
						required
					/>
				{/snippet}
			</nameForm.Field>
			<nameForm.Subscribe selector={(state) => state.isSubmitting}>
				{#snippet children(submitting)}
					<button class="create" type="submit" disabled={submitting}>
						{submitting ? m['workspaces.creating']() : m['workspaces.create']()}
					</button>
				{/snippet}
			</nameForm.Subscribe>
		</form>
	</section>
</main>

<Toaster />

<style>
	.picker {
		min-height: 100dvh;
		display: grid;
		place-items: center;
		padding: var(--space-5);
		background: var(--background);
		color: var(--foreground);
		font-family: var(--font-sans);
	}

	.card {
		width: 100%;
		max-width: 420px;
		padding: var(--space-6) var(--space-5);
		background: var(--card);
		border: var(--stroke-thin) solid var(--border-subtle);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-popover);
	}

	.head {
		margin-bottom: var(--space-5);
	}

	.mark {
		font-weight: 700;
		font-size: var(--text-sm);
		letter-spacing: 0.02em;
		color: var(--muted-foreground);
	}

	.title {
		margin: var(--space-2) 0 0;
		font-size: var(--text-xl);
		font-weight: 600;
		line-height: var(--leading-tight);
	}

	.muted {
		color: var(--muted-foreground);
		font-size: var(--text-sm);
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		margin: 0 0 var(--space-3);
	}

	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--space-3);
		border-radius: var(--radius-md);
		background: var(--muted);
		color: var(--foreground);
		font-size: var(--text-base);
		border: var(--stroke-thin) solid transparent;
		transition: border-color var(--dur-fast) var(--ease-out);
	}

	.row:hover {
		border-color: var(--border);
	}

	.role {
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		text-transform: capitalize;
	}

	.create-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.create {
		width: 100%;
		height: 38px;
		border-radius: var(--radius-md);
		background: var(--primary);
		color: var(--primary-foreground);
		font-size: var(--text-sm);
		font-weight: 600;
	}

	.create:disabled {
		opacity: 0.6;
		cursor: default;
	}

	.error {
		color: var(--destructive);
		font-size: var(--text-sm);
		margin: 0 0 var(--space-3);
	}
</style>
