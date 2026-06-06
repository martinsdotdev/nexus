<script lang="ts">
	// The email one-time-code sign-in (ADR-0010): a two-step machine (request a code, then
	// verify it), each step backed by TanStack Form with a pure validator (shared/lib) shown
	// inline through the Ark Field wrapper. Server outcomes surface as a form-level alert, and
	// a success toast confirms the code was sent. The route owns navigation; this reports
	// success through `onAuthenticated`, so it stays router-free and testable.
	import { createForm } from '@tanstack/svelte-form';
	import { m } from '$lib/paraglide/messages';
	import Field from '$lib/shared/ui/Field.svelte';
	import { toast } from '$lib/shared/ui/toast';
	import { emailError, loginCodeError } from '$lib/shared/lib/validators';

	interface Props {
		/** Called once a verified code has established the session. */
		onAuthenticated?: () => void;
	}
	let { onAuthenticated }: Props = $props();

	type Step = 'email' | 'code';
	let step = $state<Step>('email');
	let sentTo = $state('');
	let serverError = $state<string | null>(null);

	const emailForm = createForm(() => ({
		defaultValues: { email: '' },
		onSubmit: async ({ value }) => {
			serverError = null;
			try {
				const res = await fetch('/auth/email', {
					method: 'POST',
					headers: { 'content-type': 'application/json' },
					credentials: 'include',
					body: JSON.stringify({ email: value.email })
				});
				if (res.ok) {
					sentTo = value.email;
					step = 'code';
					toast.success(m['login.code_sent']({ email: value.email }));
				} else if (res.status === 400) {
					serverError = m['login.error_email']();
				} else {
					serverError = m['login.error_network']();
				}
			} catch {
				serverError = m['login.error_network']();
			}
		}
	}));

	const codeForm = createForm(() => ({
		defaultValues: { code: '' },
		onSubmit: async ({ value }) => {
			serverError = null;
			try {
				const res = await fetch('/auth/email/verify', {
					method: 'POST',
					headers: { 'content-type': 'application/json' },
					credentials: 'include',
					body: JSON.stringify({ code: value.code.trim().toUpperCase() })
				});
				if (res.ok) {
					onAuthenticated?.();
				} else if (res.status === 429) {
					serverError = m['login.error_rate_limited']();
				} else {
					serverError = m['login.error_code']();
				}
			} catch {
				serverError = m['login.error_network']();
			}
		}
	}));

	function restart() {
		step = 'email';
		serverError = null;
		codeForm.reset();
	}
</script>

{#if step === 'email'}
	<form
		class="form"
		onsubmit={(event) => {
			event.preventDefault();
			emailForm.handleSubmit();
		}}
	>
		<emailForm.Field
			name="email"
			validators={{
				onBlur: ({ value }) => emailError(value),
				onSubmit: ({ value }) => emailError(value)
			}}
		>
			{#snippet children(field)}
				<Field
					label={m['login.email_label']()}
					type="email"
					autocomplete="email"
					placeholder={m['login.email_placeholder']()}
					value={field.state.value}
					oninput={(v) => field.handleChange(v)}
					onblur={() => field.handleBlur()}
					error={field.state.meta.errors[0]}
					required
				/>
			{/snippet}
		</emailForm.Field>

		{#if serverError}<p class="error" role="alert">{serverError}</p>{/if}

		<emailForm.Subscribe selector={(state) => state.isSubmitting}>
			{#snippet children(submitting)}
				<button class="submit" type="submit" disabled={submitting} aria-busy={submitting}>
					{submitting ? m['login.sending']() : m['login.send_code']()}
				</button>
			{/snippet}
		</emailForm.Subscribe>
	</form>
{:else}
	<form
		class="form"
		onsubmit={(event) => {
			event.preventDefault();
			codeForm.handleSubmit();
		}}
	>
		<p class="hint">{m['login.code_hint']({ email: sentTo })}</p>

		<codeForm.Field
			name="code"
			validators={{
				onBlur: ({ value }) => loginCodeError(value),
				onSubmit: ({ value }) => loginCodeError(value)
			}}
		>
			{#snippet children(field)}
				<Field
					label={m['login.code_label']()}
					autocomplete="one-time-code"
					inputmode="text"
					maxlength={8}
					mono
					value={field.state.value}
					oninput={(v) => field.handleChange(v)}
					onblur={() => field.handleBlur()}
					error={field.state.meta.errors[0]}
					required
				/>
			{/snippet}
		</codeForm.Field>

		{#if serverError}<p class="error" role="alert">{serverError}</p>{/if}

		<codeForm.Subscribe selector={(state) => state.isSubmitting}>
			{#snippet children(submitting)}
				<button class="submit" type="submit" disabled={submitting} aria-busy={submitting}>
					{submitting ? m['login.verifying']() : m['login.verify']()}
				</button>
			{/snippet}
		</codeForm.Subscribe>

		<button class="link" type="button" onclick={restart}>
			{m['login.use_different_email']()}
		</button>
	</form>
{/if}

<style>
	.form {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.hint {
		font-size: var(--text-sm);
		color: var(--muted-foreground);
		line-height: var(--leading-base);
	}

	.submit {
		height: 38px;
		border-radius: var(--radius-md);
		background: var(--primary);
		color: var(--primary-foreground);
		font-size: var(--text-sm);
		font-weight: 600;
		transition: opacity var(--dur-fast) var(--ease-out);
	}

	.submit:not(:disabled):active {
		transform: scale(var(--press-scale));
	}

	.submit:disabled {
		opacity: 0.6;
		cursor: default;
	}

	.link {
		align-self: center;
		color: var(--muted-foreground);
		font-size: var(--text-sm);
	}

	@media (hover: hover) {
		.link:hover {
			color: var(--foreground);
		}
	}

	.error {
		color: var(--destructive);
		font-size: var(--text-sm);
		line-height: var(--leading-base);
	}
</style>
