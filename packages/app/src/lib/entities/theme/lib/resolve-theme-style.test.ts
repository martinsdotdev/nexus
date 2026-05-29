import { describe, expect, test } from 'vitest';
import { resolveThemeStyle } from './resolve-theme-style';
import type { Theme } from '$lib/shared/crdt/workspace-view';

const theme = (id: string, tokens: Record<string, string>, base = ''): Theme => ({
	id,
	name: id,
	base,
	protected: false,
	tokens
});

// A stand-in for the seeded default theme (the resolver's safety floor).
const cozy = theme('cozy', {
	primary: 'cozy-primary',
	background: 'cozy-bg',
	accent: 'cozy-accent'
});

describe('resolveThemeStyle', () => {
	test("inlines the selected theme's tokens as CSS variables", () => {
		const r = resolveThemeStyle('t', [theme('t', { primary: 'red', background: 'blue' })]);
		expect(r.inlineVars).toContain('--primary: red;');
		expect(r.inlineVars).toContain('--background: blue;');
	});

	test('an accent override replaces the theme accent and is emitted exactly once', () => {
		const r = resolveThemeStyle('t', [theme('t', { accent: 'green' })], 'purple');
		expect(r.inlineVars).toContain('--accent: purple;');
		expect(r.inlineVars).not.toContain('green');
		expect(r.inlineVars.match(/--accent:/g)?.length).toBe(1);
	});

	test('an empty accent override is ignored', () => {
		const r = resolveThemeStyle('t', [theme('t', { accent: 'green' })], '');
		expect(r.inlineVars).not.toContain('purple');
		expect(r.inlineVars).toContain('--accent: green;');
	});

	test('a link token resolves to its target value', () => {
		const r = resolveThemeStyle('t', [theme('t', { primary: 'red', ring: 'link:primary' })]);
		expect(r.inlineVars).toContain('--ring: red;');
	});

	test('a link cycle is dropped without looping (no default floor present)', () => {
		const r = resolveThemeStyle('t', [
			theme('t', { primary: 'link:accent', accent: 'link:primary' })
		]);
		expect(r.inlineVars).not.toContain('--primary:');
		expect(r.inlineVars).not.toContain('--accent:');
	});

	test('an unset token inherits from the theme base', () => {
		const base = theme('cyber', { primary: 'cyber-primary', background: 'cyber-bg' });
		const derived = theme('t', { primary: 'override' }, 'cyber');
		const r = resolveThemeStyle('t', [base, derived]);
		expect(r.inlineVars).toContain('--primary: override;'); // own value wins
		expect(r.inlineVars).toContain('--background: cyber-bg;'); // inherited from base
	});

	test('an unset token falls back to the default theme when no base covers it', () => {
		const derived = theme('t', { primary: 'override' }); // no base
		const r = resolveThemeStyle('t', [cozy, derived]);
		expect(r.inlineVars).toContain('--primary: override;');
		expect(r.inlineVars).toContain('--background: cozy-bg;'); // default floor
	});

	test('a dangling link falls back to the default theme literal', () => {
		const derived = theme('t', { background: 'link:nonexistent' });
		const r = resolveThemeStyle('t', [cozy, derived]);
		expect(r.inlineVars).toContain('--background: cozy-bg;');
	});

	test('an unknown theme id falls back to the default theme', () => {
		const r = resolveThemeStyle('missing', [cozy]);
		expect(r.inlineVars).toContain('--background: cozy-bg;');
		expect(r.inlineVars).toContain('--primary: cozy-primary;');
	});

	test('an empty registry yields no inline vars (transient pre-sync state)', () => {
		expect(resolveThemeStyle('cozy', []).inlineVars).toBe('');
	});
});
