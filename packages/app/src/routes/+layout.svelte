<script lang="ts">
	import '$lib/shared/styles/reset.css';
	import '$lib/shared/styles/tokens.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
{@render children()}
