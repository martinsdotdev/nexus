<script module lang="ts">
	export interface RosterPerson {
		id: string;
		name: string;
		/** This is the local user (gets a color ring). */
		you?: boolean;
		/** The workspace owner (gets a crown pip). */
		host?: boolean;
		/** Present but inactive (dimmed). */
		idle?: boolean;
	}
</script>

<script lang="ts">
	// A who's-online avatar stack in the titlebar: one monogram chip per person in the
	// workspace, you first, each in that peer's curated color (matching their cursor and
	// selection outline). Your own chip carries a color ring; the workspace owner carries a
	// crown pip; idle peers dim. Beyond `max` chips it collapses to a "+N" badge. Clicking
	// opens Share (when an open handler is supplied). Hidden when nobody is here.
	import { Crown } from 'lucide-svelte';
	import { peerColor } from '../lib/peer-color';

	interface Props {
		people: RosterPerson[];
		/** How many chips to show before collapsing the rest into "+N". */
		max?: number;
		/** Open the Share/collaborators panel (makes the stack a button). */
		onOpen?: () => void;
	}
	let { people, max = 4, onOpen }: Props = $props();

	const shown = $derived(people.slice(0, max));
	const overflow = $derived(Math.max(0, people.length - max));

	// Monogram: two letters from a single name, or first+last initial of a full name.
	function initials(name: string): string {
		const parts = name.trim().split(/\s+/).filter(Boolean);
		if (parts.length === 0) return '?';
		if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
		return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
	}
</script>

{#snippet chips()}
	{#each shown as person (person.id)}
		<span
			class="avatar"
			class:you={person.you}
			class:idle={person.idle}
			style="--peer: {peerColor(person.id)};"
			title={person.name + (person.host ? ' · host' : '')}
		>
			{initials(person.name)}
			{#if person.host}
				<span class="pip"><Crown size={8} /></span>
			{/if}
		</span>
	{/each}
	{#if overflow > 0}
		<span class="avatar overflow">+{overflow}</span>
	{/if}
{/snippet}

{#if people.length > 0}
	{#if onOpen}
		<button
			class="stack"
			data-testid="roster"
			onclick={onOpen}
			aria-label="{people.length} people in this workspace"
		>
			{@render chips()}
		</button>
	{:else}
		<span class="stack" data-testid="roster">{@render chips()}</span>
	{/if}
{/if}

<style>
	.stack {
		display: inline-flex;
		align-items: center;
		padding-left: 4px;
	}
	button.stack {
		cursor: pointer;
	}

	.avatar {
		--peer: var(--mp-teal);
		position: relative;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
		border-radius: 50%;
		flex: none;
		font-size: 11px;
		font-weight: 700;
		color: oklch(18% 0.01 264);
		background: var(--peer);
		box-shadow: 0 0 0 2px var(--titlebar);
		margin-left: -7px;
		transition: transform var(--dur-fast) var(--ease-out);
	}
	.avatar:first-child {
		margin-left: 0;
	}
	.stack:hover .avatar {
		transform: translateY(-1px);
	}

	/* Your own chip: a second ring in your color. */
	.avatar.you {
		box-shadow:
			0 0 0 2px var(--titlebar),
			0 0 0 3.5px var(--peer);
	}
	.avatar.idle {
		opacity: 0.5;
	}

	/* Host crown, tucked into the chip's bottom-right. */
	.pip {
		position: absolute;
		right: -2px;
		bottom: -2px;
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background: var(--titlebar);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--warning);
	}

	.overflow {
		background: var(--secondary);
		color: var(--secondary-foreground);
		font-weight: 600;
	}
</style>
