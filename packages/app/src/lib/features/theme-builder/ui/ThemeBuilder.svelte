<script lang="ts">
	// The Zed-style theme builder: assign the active scene's theme, duplicate a
	// built-in into an editable custom theme, edit the full token set (grouped),
	// link a token to another (cycle-guarded), and import/export. The editor
	// canvas is the live preview. Presentational: every change is a callback the
	// page wires to the Loro client. Editor chrome (editor tokens).
	import type { CustomTheme, SceneView, ThemeTokens } from '$lib/shared/crdt/workspace-view';
	import {
		BUILTIN_THEME_IDS,
		LINK_PREFIX,
		TOKEN_GROUPS,
		isBuiltinTheme,
		isColorToken,
		readBuiltinTokens,
		resolveTokenValue,
		wouldCycle
	} from '$lib/entities/theme';

	interface Props {
		scene: SceneView | null;
		customThemes: CustomTheme[];
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
		customThemes,
		onCreateTheme,
		onRenameTheme,
		onSetThemeToken,
		onDeleteTheme,
		onExportTheme,
		onSetSceneTheme,
		onClose
	}: Props = $props();

	const activeThemeId = $derived(scene?.themeId ?? '');
	const activeCustom = $derived(customThemes.find((theme) => theme.id === activeThemeId) ?? null);
	const allTokens = TOKEN_GROUPS.flatMap((group) => group.tokens);

	let importText = $state('');

	const linkTarget = (tokens: ThemeTokens, token: string): string => {
		const value = tokens[token];
		return value?.startsWith(LINK_PREFIX) ? value.slice(LINK_PREFIX.length) : '';
	};

	function duplicateToCustomize() {
		if (!scene) return;
		const base = isBuiltinTheme(activeThemeId) ? activeThemeId : 'cozy';
		const id = onCreateTheme(`${base} custom`, base, readBuiltinTokens(base));
		onSetSceneTheme(scene.id, id);
	}

	function changeLink(theme: CustomTheme, token: string, target: string) {
		if (target === '') {
			// Unlink: freeze the token at its currently-resolved literal.
			onSetThemeToken(theme.id, token, resolveTokenValue(theme.tokens, token));
		} else if (!wouldCycle(theme.tokens, token, target)) {
			onSetThemeToken(theme.id, token, `${LINK_PREFIX}${target}`);
		}
	}

	function exportActive(theme: CustomTheme) {
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

	function deleteActive(theme: CustomTheme) {
		if (!scene) return;
		const fallback = isBuiltinTheme(theme.base) ? theme.base : 'cozy';
		onDeleteTheme(theme.id);
		onSetSceneTheme(scene.id, fallback);
	}

	function importTheme() {
		if (!scene) return;
		try {
			const parsed = JSON.parse(importText);
			const base = isBuiltinTheme(String(parsed.base)) ? String(parsed.base) : 'cozy';
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
			<select value={activeThemeId} onchange={(e) => onSetSceneTheme(s.id, e.currentTarget.value)}>
				{#each BUILTIN_THEME_IDS as id (id)}
					<option value={id}>{id}</option>
				{/each}
				{#each customThemes as theme (theme.id)}
					<option value={theme.id}>{theme.name}</option>
				{/each}
			</select>
		</label>

		{#if activeCustom}
			{@const ac = activeCustom}
			<label class="field">
				<span>Name</span>
				<input
					type="text"
					value={ac.name}
					oninput={(e) => onRenameTheme(ac.id, e.currentTarget.value)}
				/>
			</label>
			<div class="actions">
				<button class="ghost" onclick={() => exportActive(ac)}>Export</button>
				<button class="ghost danger" onclick={() => deleteActive(ac)}>Delete</button>
			</div>

			{#each TOKEN_GROUPS as group (group.label)}
				<details open>
					<summary>{group.label}</summary>
					{#each group.tokens as token (token)}
						{@const linked = linkTarget(ac.tokens, token)}
						<div class="token-row">
							<span class="token-label">{token}</span>
							{#if linked}
								<span class="linked" title="Linked to {linked}">&rarr; {linked}</span>
							{:else}
								{#if isColorToken(token)}
									<span class="swatch" style="background: {ac.tokens[token] || 'transparent'};"
									></span>
								{/if}
								<input
									class="token-input"
									type="text"
									aria-label={token}
									value={ac.tokens[token] ?? ''}
									oninput={(e) => onSetThemeToken(ac.id, token, e.currentTarget.value)}
								/>
							{/if}
							<select
								class="link-select"
								value={linked}
								aria-label="Link {token}"
								onchange={(e) => changeLink(ac, token, e.currentTarget.value)}
							>
								<option value="">(literal)</option>
								{#each allTokens.filter((other) => other !== token) as other (other)}
									<option value={other}>{other}</option>
								{/each}
							</select>
						</div>
					{/each}
				</details>
			{/each}

			<details>
				<summary>Import</summary>
				<textarea bind:value={importText} rows="4" placeholder="Paste theme JSON"></textarea>
				<button class="ghost" onclick={importTheme}>Import as new theme</button>
			</details>
		{:else}
			<p class="hint">This scene uses a built-in theme. Duplicate it to edit its tokens.</p>
			<button class="primary" onclick={duplicateToCustomize}>Duplicate to customize</button>
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

	button.primary {
		background: var(--primary);
		color: var(--primary-foreground);
		border-color: transparent;
	}

	button.danger {
		color: var(--destructive);
	}

	summary {
		font-size: var(--text-xs);
		color: var(--muted-foreground);
		cursor: pointer;
		padding: var(--space-1) 0;
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

	.swatch {
		width: 16px;
		height: 16px;
		flex: none;
		border-radius: var(--radius-sm);
		border: var(--stroke-thin) solid var(--border);
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

	.link-select {
		width: 2.4ch;
		flex: none;
	}

	input[type='text'],
	select,
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
	select:focus-visible,
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
