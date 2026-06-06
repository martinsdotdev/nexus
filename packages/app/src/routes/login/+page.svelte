<script lang="ts">
	// The sign-in page: a centered auth card around the email-code form. Standalone (no
	// editor chrome); the root layout supplies the global tokens and fonts. On a verified
	// code the form establishes the session and we navigate to the workspace picker.
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { m } from '$lib/paraglide/messages';
	import LoginForm from './LoginForm.svelte';
	import Toaster from '$lib/shared/ui/Toaster.svelte';
</script>

<svelte:head><title>{m['login.title']()}</title></svelte:head>

<main class="auth">
	<section class="card">
		<header class="head">
			<span class="mark">{m['app.name']()}</span>
			<h1 class="title">{m['login.title']()}</h1>
		</header>
		<LoginForm onAuthenticated={() => goto(resolve('/workspaces'))} />
	</section>
</main>

<Toaster />

<style>
	.auth {
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
		max-width: 360px;
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
</style>
