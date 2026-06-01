<script lang="ts">
	// The Zed-style theme builder: assign the active scene's theme, edit any
	// theme's full token set in place (grouped), link a token to another (cycle-
	// guarded), duplicate a theme into an editable fork, and import/export. Every
	// theme is registry data now (ADR-0007), so built-ins are editable too; they
	// are `protected` (Delete hidden) so the default fallback always exists. The
	// editor canvas is the live preview. Presentational: every change is a callback
	// the page wires to the Loro client. Editor chrome (editor tokens).
	import type { Theme, SceneView, ThemeTokens } from '$lib/shared/crdt/workspace-view';
	import {
		DEFAULT_THEME_ID,
		LINK_PREFIX,
		THEME_TOKENS,
		TOKEN_GROUPS,
		isColorToken,
		resolveTokenValue,
		wouldCycle,
		oklchToHex,
		colorToOklch
	} from '$lib/entities/theme';
	import { Select, Collapsible, ColorField } from '$lib/shared/ui';

	interface Props {
		scene: SceneView | null;
		themes: Theme[];
		onCreateTheme: (name: string, base: string, tokens: ThemeTokens) => string;
		onRenameTheme: (id: string, name: string) => void;
		onSetThemeToken: (id: string, token: string, value: string) => void;
		onDeleteTheme: (id: string) => void;
		onExportTheme: (id: string) => { name: string; base: string; tokens: ThemeTokens } | null;
		onSetSceneTheme: (sceneId: string, themeId: string) => void;
		onClose: () => void;
	}
	let {
		scene,
		themes,
		onCreateTheme,
		onRenameTheme,
		onSetThemeToken,
		onDeleteTheme,
		onExportTheme,
		onSetSceneTheme,
		onClose
	}: Props = $props();

	const activeThemeId = $derived(scene?.themeId ?? '');
	const activeTheme = $derived(themes.find((theme) => theme.id === activeThemeId) ?? null);

	let importText = $state('');

	const linkTarget = (tokens: ThemeTokens, token: string): string => {
		const value = tokens[token];
		return value?.startsWith(LINK_PREFIX) ? value.slice(LINK_PREFIX.length) : '';
	};

	// Duplicate the active theme into a new, editable, deletable copy and assign it.
	// The copy derives from a protected built-in (if the source is one, inherit it;
	// otherwise carry the source's built-in base) so it keeps a resolvable floor.
	function duplicateTheme() {
		if (!scene || !activeTheme) return;
		const base = activeTheme.protected ? activeTheme.id : activeTheme.base;
		const id = onCreateTheme(`${activeTheme.name} copy`, base, { ...activeTheme.tokens });
		onSetSceneTheme(scene.id, id);
	}

	function changeLink(theme: Theme, token: string, target: string) {
		if (target === '') {
			// Unlink: freeze the token at its currently-resolved literal. If the chain
			// is dangling/cyclic (resolves to ''), leave the link for resolveThemeStyle
			// to fall back on the default theme rather than writing an empty token.
			const resolved = resolveTokenValue(theme.tokens, token);
			if (resolved) onSetThemeToken(theme.id, token, resolved);
		} else if (!wouldCycle(theme.tokens, token, target)) {
			onSetThemeToken(theme.id, token, `${LINK_PREFIX}${target}`);
		}
	}

	function exportActive(theme: Theme) {
		const data = onExportTheme(theme.id);
		if (!data) return;
		const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
		const url = URL.createObjectURL(blob);
		const anchor = document.createElement('a');
		anchor.href = url;
		anchor.download = `${data.name || 'theme'}.json`;
		anchor.click();
		URL.revokeObjectURL(url);
	}

	function deleteActive(theme: Theme) {
		if (!scene) return;
		const fallback = themes.some((t) => t.id === theme.base) ? theme.base : DEFAULT_THEME_ID;
		onDeleteTheme(theme.id);
		onSetSceneTheme(scene.id, fallback);
	}

	function importTheme() {
		if (!scene) return;
		try {
			const parsed = JSON.parse(importText);
			const base = themes.some((t) => t.id === String(parsed.base)) ? String(parsed.base) : '';
			const tokens =
				parsed.tokens && typeof parsed.tokens === 'object'
					? (parsed.tokens as ThemeTokens)
					: ({} as ThemeTokens);
			const id = onCreateTheme(String(parsed.name ?? 'Imported'), base, tokens);
			onSetSceneTheme(scene.id, id);
			importText = '';
		} catch {
			// Ignore invalid JSON; the textarea keeps its content for correction.
		}
	}
</script>

<div class="theme-builder">
	<header class="head">
		<h2>Theme</h2>
		<button class="ghost" onclick={onClose}>Done</button>
	</header>

	{#if scene}
		{@const s = scene}
		<label class="field">
			<span>Scene theme</span>
			<Select
				ariaLabel="Scene theme"
				value={activeThemeId}
				options={themes.map((theme) => ({ value: theme.id, label: theme.name }))}
				onChange={(value) => onSetSceneTheme(s.id, value)}
			/>
		</label>

		{#if activeTheme}
			{@const ac = activeTheme}
			<label class="field">
				<span>Name</span>
				<input
					type="text"
					value={ac.name}
					oninput={(e) => onRenameTheme(ac.id, e.currentTarget.value)}
				/>
			</label>
			<div class="actions">
				<button class="ghost" onclick={duplicateTheme}>Duplicate</button>
				<button class="ghost" onclick={() => exportActive(ac)}>Export</button>
				{#if !ac.protected}
					<button class="ghost danger" onclick={() => deleteActive(ac)}>Delete</button>
				{/if}
			</div>
			{#if ac.protected}
				<p class="hint">
					Built-in theme: edits apply in place. Duplicate to fork a deletable copy.
				</p>
			{/if}

			{#each TOKEN_GROUPS as group (group.label)}
				<Collapsible title={group.label} open>
					{#each group.tokens as token (token)}
						{@const linked = linkTarget(ac.tokens, token)}
						<div class="token-row">
							<span class="token-label">{token}</span>
							{#if linked}
								<span class="linked" title="Linked to {linked}">&rarr; {linked}</span>
							{:else if isColorToken(token)}
								<ColorField
									ariaLabel={token}
									value={oklchToHex(ac.tokens[token] ?? '')}
									onChange={(color) => onSetThemeToken(ac.id, token, colorToOklch(color))}
								/>
							{:else}
								<input
									class="token-input"
									type="text"
									aria-label={token}
									value={ac.tokens[token] ?? ''}
									oninput={(e) => onSetThemeToken(ac.id, token, e.currentTarget.value)}
								/>
							{/if}
							{#if !ac.protected}
								<!-- Built-ins hold literals only (the resolver's floor), so no link control. -->
								<Select
									compact
									ariaLabel="Link {token}"
									value={linked}
									options={[
										{ value: '', label: '(literal)' },
										...THEME_TOKENS.filter((other) => other !== token).map((other) => ({
											value: other,
											label: other
										}))
									]}
									onChange={(value) => changeLink(ac, token, value)}
								/>
							{/if}
						</div>
					{/each}
				</Collapsible>
			{/each}

			<Collapsible title="Import">
				<textarea bind:value={importText} rows="4" placeholder="Paste theme JSON"></textarea>
				<button class="ghost" onclick={importTheme}>Import as new theme</button>
			</Collapsible>
		{:else}
			<p class="hint">This scene's theme is not in the registry.</p>
		{/if}
	{:else}
		<p class="hint">No active scene.</p>
	{/if}
</div>

<style>
	.theme-builder {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	h2 {
		margin: 0;
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--foreground);
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		font-size: var(--text-xs);
		color: var(--muted-foreground);
	}

	.actions {
		display: flex;
		gap: var(--space-2);
	}

	button {
		font: inherit;
		font-size: var(--text-xs);
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-md);
		border: var(--stroke-thin) solid var(--border);
		background: var(--secondary);
		color: var(--secondary-foreground);
		cursor: pointer;
	}

	button.danger {
		color: var(--destructive);
	}

	.token-row {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		padding: 2px 0;
	}

	.token-label {
		flex: 1;
		min-width: 0;
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.token-input {
		width: 9ch;
		flex: none;
	}

	.linked {
		flex: none;
		font-size: var(--text-xs);
		color: var(--primary);
		font-family: var(--font-mono);
	}

	input[type='text'],
	textarea {
		font: inherit;
		font-size: var(--text-xs);
		color: var(--foreground);
		background: var(--input);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-sm);
		padding: 2px var(--space-1);
	}

	textarea {
		width: 100%;
		resize: vertical;
		font-family: var(--font-mono);
	}

	input:focus-visible,
	textarea:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	.hint {
		margin: 0;
		font-size: var(--text-sm);
		color: var(--muted-foreground);
	}
</style>
