<script lang="ts">
	// The overlay must be transparent so OBS composites it over the game capture.
	// The root layout ships the dark editor tokens + class="dark" on <html>; this
	// nested layout neutralizes the page background for the /overlay route only.
	let { children } = $props();
</script>

{@render children()}

<style>
	:global(html),
	:global(body) {
		background: transparent !important;
	}
	:global(html.dark) {
		--background: transparent;
		color-scheme: normal;
	}
</style>
