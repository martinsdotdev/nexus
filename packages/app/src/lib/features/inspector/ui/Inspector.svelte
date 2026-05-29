<script lang="ts">
	// The inspector: a two-way bound editor for the selected widget (geometry, z,
	// visibility, and type-specific props) or, when nothing is selected, the active
	// scene's theme + overrides. Presentational: every change is reported through a
	// callback the page wires to the Loro client, so edits preview live on the
	// canvas/overlay and coalesce into undo steps. Editor chrome (editor tokens).
	import type { Theme, SceneView, WidgetView } from '$lib/shared/crdt/workspace-view';
	import { propSchemaFor } from '$lib/entities/widget';

	interface Props {
		widget: WidgetView | null;
		scene: SceneView | null;
		themes: Theme[];
		onSetGeometry: (
			id: string,
			geom: { x?: number; y?: number; w?: number; h?: number; z?: number }
		) => void;
		onSetProp: (id: string, key: string, value: unknown) => void;
		onSetVisible: (id: string, visible: boolean) => void;
		onSetSceneTheme: (sceneId: string, themeId: string) => void;
		onSetSceneOverride: (sceneId: string, key: string, value: string) => void;
		onCustomizeTheme?: () => void;
	}
	let {
		widget,
		scene,
		themes,
		onSetGeometry,
		onSetProp,
		onSetVisible,
		onSetSceneTheme,
		onSetSceneOverride,
		onCustomizeTheme
	}: Props = $props();

	const fields = $derived(widget ? propSchemaFor(widget.widgetType) : []);
	// Number fields commit only a finite parse, so typing '-', '1.', or clearing
	// the field is not clobbered by an immediate 0-write and a valid intermediate
	// is never forced back mid-keystroke.
	function onNumberInput(raw: string, write: (value: number) => void) {
		if (raw === '') return;
		const value = Number(raw);
		if (Number.isFinite(value)) write(value);
	}
	const DENSITIES = ['normal', 'compact', 'spacious'];
</script>

{#if widget}
	{@const w = widget}
	<div class="inspector">
		<h2 class="title">{w.widgetType}</h2>

		<fieldset class="group">
			<legend>Position &amp; size</legend>
			<div class="grid">
				<label
					>X<input
						type="number"
						value={w.x}
						oninput={(e) =>
							onNumberInput(e.currentTarget.value, (v) => onSetGeometry(w.id, { x: v }))}
					/></label
				>
				<label
					>Y<input
						type="number"
						value={w.y}
						oninput={(e) =>
							onNumberInput(e.currentTarget.value, (v) => onSetGeometry(w.id, { y: v }))}
					/></label
				>
				<label
					>W<input
						type="number"
						value={w.w}
						oninput={(e) =>
							onNumberInput(e.currentTarget.value, (v) => onSetGeometry(w.id, { w: v }))}
					/></label
				>
				<label
					>H<input
						type="number"
						value={w.h}
						oninput={(e) =>
							onNumberInput(e.currentTarget.value, (v) => onSetGeometry(w.id, { h: v }))}
					/></label
				>
				<label
					>Z<input
						type="number"
						value={w.z}
						oninput={(e) =>
							onNumberInput(e.currentTarget.value, (v) => onSetGeometry(w.id, { z: v }))}
					/></label
				>
			</div>
		</fieldset>

		<label class="row">
			<input
				type="checkbox"
				checked={w.visible}
				onchange={(e) => onSetVisible(w.id, e.currentTarget.checked)}
			/>
			Visible
		</label>

		{#if fields.length}
			<fieldset class="group">
				<legend>Properties</legend>
				{#each fields as field (field.key)}
					<label class="field">
						<span>{field.label}</span>
						{#if field.type === 'select'}
							<select
								value={String(w.props[field.key] ?? '')}
								onchange={(e) => onSetProp(w.id, field.key, e.currentTarget.value)}
							>
								{#each field.options ?? [] as option (option)}
									<option value={option}>{option}</option>
								{/each}
							</select>
						{:else if field.type === 'number'}
							<input
								type="number"
								value={Number(w.props[field.key] ?? 0)}
								oninput={(e) =>
									onNumberInput(e.currentTarget.value, (v) => onSetProp(w.id, field.key, v))}
							/>
						{:else}
							<input
								type="text"
								value={String(w.props[field.key] ?? '')}
								oninput={(e) => onSetProp(w.id, field.key, e.currentTarget.value)}
							/>
						{/if}
					</label>
				{/each}
			</fieldset>
		{/if}
	</div>
{:else if scene}
	{@const s = scene}
	<div class="inspector">
		<h2 class="title">Scene</h2>
		<label class="field">
			<span>Theme</span>
			<select value={s.themeId} onchange={(e) => onSetSceneTheme(s.id, e.currentTarget.value)}>
				{#each themes as theme (theme.id)}
					<option value={theme.id}>{theme.name}</option>
				{/each}
			</select>
		</label>
		<label class="field">
			<span>Accent</span>
			<input
				type="text"
				placeholder="theme default"
				value={s.overridesAccent}
				oninput={(e) => onSetSceneOverride(s.id, 'overridesAccent', e.currentTarget.value)}
			/>
		</label>
		<label class="field">
			<span>Density</span>
			<select
				value={s.overridesDensity || 'normal'}
				onchange={(e) => onSetSceneOverride(s.id, 'overridesDensity', e.currentTarget.value)}
			>
				{#each DENSITIES as density (density)}
					<option value={density}>{density}</option>
				{/each}
			</select>
		</label>
		<button class="customize" onclick={() => onCustomizeTheme?.()}>Customize theme</button>
	</div>
{:else}
	<p class="empty">Select a widget to edit its properties.</p>
{/if}

<style>
	.inspector {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.title {
		margin: 0;
		font-size: var(--text-sm);
		font-weight: 600;
		text-transform: capitalize;
		color: var(--foreground);
	}

	.group {
		border: var(--stroke-thin) solid var(--border-subtle);
		border-radius: var(--radius-md);
		padding: var(--space-2) var(--space-3) var(--space-3);
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	legend {
		padding: 0 var(--space-1);
		font-size: var(--text-xs);
		color: var(--muted-foreground);
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-2);
	}

	label {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--text-xs);
		color: var(--muted-foreground);
	}

	.field {
		flex-direction: column;
		align-items: stretch;
		gap: var(--space-1);
	}

	.row {
		font-size: var(--text-sm);
		color: var(--foreground);
	}

	input[type='number'],
	input[type='text'],
	select {
		width: 100%;
		min-width: 0;
		padding: var(--space-1) var(--space-2);
		font: inherit;
		font-size: var(--text-sm);
		color: var(--foreground);
		background: var(--input);
		border: var(--stroke-thin) solid var(--border);
		border-radius: var(--radius-md);
	}

	input:focus-visible,
	select:focus-visible {
		outline: none;
		box-shadow: var(--focus-ring);
	}

	.empty {
		margin: 0;
		font-size: var(--text-sm);
		color: var(--muted-foreground);
	}

	.customize {
		align-self: flex-start;
		font: inherit;
		font-size: var(--text-xs);
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-md);
		border: var(--stroke-thin) solid var(--border);
		background: var(--secondary);
		color: var(--secondary-foreground);
		cursor: pointer;
	}
</style>
