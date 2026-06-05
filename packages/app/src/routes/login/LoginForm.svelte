<script lang="ts">
	// The email one-time-code sign-in form (ADR-0010), a two-step state machine: enter an
	// email to request a code, then enter the code to establish a session. It talks to the
	// cloud-mode relay over plain HTTP (same-origin, so the browser sends the session
	// cookie and the `Sec-Fetch-Site` header the CSRF guard checks). The route owns
	// navigation; this component reports success through `onAuthenticated` so it can be
	// tested without a router.
	import { m } from '$lib/paraglide/messages';

	interface Props {
		/** Called once a verified code has established the session. */
		onAuthenticated?: () => void;
	}
	let { onAuthenticated }: Props = $props();

	type Step = 'email' | 'code';
	let step = $state<Step>('email');
	let email = $state('');
	let code = $state('');
	let pending = $state(false);
	let error = $state<string | null>(null);

	async function requestCode(event: SubmitEvent) {
		event.preventDefault();
		error = null;
		pending = true;
		try {
			const res = await fetch('/auth/email', {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({ email })
			});
			if (res.ok) {
				code = '';
				step = 'code';
			} else if (res.status === 400) {
				error = m['login.error_email']();
			} else {
				error = m['login.error_network']();
			}
		} catch {
			error = m['login.error_network']();
		} finally {
			pending = false;
		}
	}

	async function verifyCode(event: SubmitEvent) {
		event.preventDefault();
		error = null;
		pending = true;
		try {
			const res = await fetch('/auth/email/verify', {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({ code: code.trim().toUpperCase() })
			});
			if (res.ok) {
				onAuthenticated?.();
			} else if (res.status === 429) {
				error = m['login.error_rate_limited']();
			} else {
				error = m['login.error_code']();
			}
		} catch {
			error = m['login.error_network']();
		} finally {
			pending = false;
		}
	}

	function restart() {
		step = 'email';
		code = '';
		error = null;
	}
</script>

{#if step === 'email'}
	<form class="form" onsubmit={requestCode}>
		<label class="field">
			<span class="label">{m['login.email_label']()}</span>
			<input
				class="input"
				type="email"
				name="email"
				autocomplete="email"
				autocapitalize="off"
				spellcheck="false"
				placeholder={m['login.email_placeholder']()}
				required
				bind:value={email}
			/>
		</label>
		{#if error}<p class="error" role="alert">{error}</p>{/if}
		<button class="submit" type="submit" disabled={pending} aria-busy={pending}>
			{pending ? m['login.sending']() : m['login.send_code']()}
		</button>
	</form>
{:else}
	<form class="form" onsubmit={verifyCode}>
		<p class="hint">{m['login.code_hint']({ email })}</p>
		<label class="field">
			<span class="label">{m['login.code_label']()}</span>
			<input
				class="input code"
				type="text"
				name="code"
				autocomplete="one-time-code"
				autocapitalize="characters"
				spellcheck="false"
				maxlength="8"
				required
				bind:value={code}
			/>
		</label>
		{#if error}<p class="error" role="alert">{error}</p>{/if}
		<button class="submit" type="submit" disabled={pending} aria-busy={pending}>
			{pending ? m['login.verifying']() : m['login.verify']()}
		</button>
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

	.field {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
	}

	.label {
		font-size: var(--text-sm);
		color: var(--muted-foreground);
	}

	.hint {
		font-size: var(--text-sm);
		color: var(--muted-foreground);
		line-height: var(--leading-base);
	}

	.input {
		height: 36px;
		padding: 0 var(--space-3);
		background: var(--input);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-md);
		font-size: var(--text-base);
		transition: border-color var(--dur-fast) var(--ease-out);
	}

	.input::placeholder {
		color: var(--muted-foreground);
	}

	/* The global :focus-visible rule supplies the ring; the field just tints its border. */
	.input:focus-visible {
		border-color: var(--ring);
	}

	.code {
		font-family: var(--font-mono);
		letter-spacing: 0.28em;
		text-transform: uppercase;
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
