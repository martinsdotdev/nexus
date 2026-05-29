import { describe, expect, test } from 'vitest';
import { resolveThemeStyle } from './resolve-theme-style';
import type { CustomTheme } from '$lib/shared/crdt/workspace-view';

const custom = (tokens: Record<string, string>, base = 'cyber'): CustomTheme => ({
	id: 'theme-x',
	name: 'X',
	base,
	tokens
});

describe('resolveThemeStyle', () => {
	test('a built-in id maps to data-theme with no inline vars', () => {
		const r = resolveThemeStyle('cozy', []);
		expect(r.dataTheme).toBe('cozy');
		expect(r.inlineVars).toBe('');
	});

	test('an accent override on a built-in inlines only --accent', () => {
		const r = resolveThemeStyle('cozy', [], 'oklch(70% 0.2 30)');
		expect(r.dataTheme).toBe('cozy');
		expect(r.inlineVars).toContain('--accent: oklch(70% 0.2 30)');
	});

	test('an empty accent override is ignored', () => {
		expect(resolveThemeStyle('cozy', [], '').inlineVars).toBe('');
	});

	test('a custom theme uses its base as data-theme and inlines its tokens', () => {
		const r = resolveThemeStyle('theme-x', [custom({ primary: 'red', background: 'blue' })]);
		expect(r.dataTheme).toBe('cyber');
		expect(r.inlineVars).toContain('--primary: red;');
		expect(r.inlineVars).toContain('--background: blue;');
	});

	test('a link token resolves to its target value', () => {
		const r = resolveThemeStyle('theme-x', [custom({ primary: 'red', ring: 'link:primary' })]);
		expect(r.inlineVars).toContain('--ring: red;');
	});

	test('a link cycle resolves safely without looping or emitting the cyclic tokens', () => {
		const r = resolveThemeStyle('theme-x', [
			custom({ primary: 'link:accent', accent: 'link:primary' })
		]);
		expect(r.inlineVars).not.toContain('--primary:');
		expect(r.inlineVars).not.toContain('--accent:');
	});

	test('a dangling link is skipped (the base CSS covers it)', () => {
		const r = resolveThemeStyle('theme-x', [custom({ ring: 'link:nonexistent' })]);
		expect(r.inlineVars).not.toContain('--ring:');
	});

	test('an accent override wins over a custom theme accent token', () => {
		const r = resolveThemeStyle('theme-x', [custom({ accent: 'green' })], 'purple');
		expect(r.inlineVars.lastIndexOf('--accent: purple')).toBeGreaterThan(
			r.inlineVars.indexOf('--accent: green')
		);
	});

	test('an unknown theme id falls back to the default built-in', () => {
		const r = resolveThemeStyle('theme-missing', []);
		expect(r.dataTheme).toBe('cozy');
	});
});
