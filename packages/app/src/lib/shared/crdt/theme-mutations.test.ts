import { describe, expect, test } from 'vitest';
import { LoroDoc, LoroMap } from 'loro-crdt';
import { createTheme, deleteTheme, exportTheme, renameTheme, setThemeToken } from './mutations';
import { readWorkspace } from './workspace-view';

const themes = (doc: LoroDoc) => readWorkspace(doc).themes;

// Mirror the relay seed: a protected built-in (literal tokens, no base).
function seedBuiltin(doc: LoroDoc, id: string, name = id) {
	const entry = doc.getMap('themes').setContainer(id, new LoroMap());
	entry.set('name', name);
	entry.set('base', '');
	entry.set('protected', true);
	entry.setContainer('tokens', new LoroMap());
}

describe('theme registry mutations', () => {
	test('createTheme registers an unprotected theme deriving from a built-in', () => {
		const doc = new LoroDoc();
		seedBuiltin(doc, 'cozy', 'Cozy');
		createTheme(doc, 'theme-1', 'My Cozy', 'cozy', { primary: 'red', accent: 'blue' });
		doc.commit();
		expect(themes(doc).find((t) => t.id === 'theme-1')).toEqual({
			id: 'theme-1',
			name: 'My Cozy',
			base: 'cozy',
			protected: false,
			tokens: { primary: 'red', accent: 'blue' }
		});
	});

	test('createTheme drops a base that is not a protected built-in', () => {
		const doc = new LoroDoc();
		createTheme(doc, 'theme-1', 'X', 'not-a-builtin', {});
		doc.commit();
		expect(themes(doc)[0].base).toBe('');
	});

	test('setThemeToken and renameTheme round-trip', () => {
		const doc = new LoroDoc();
		createTheme(doc, 'theme-1', 'X', '', { primary: 'red' });
		setThemeToken(doc, 'theme-1', 'accent', 'green');
		renameTheme(doc, 'theme-1', 'Renamed');
		doc.commit();
		const theme = themes(doc)[0];
		expect(theme.name).toBe('Renamed');
		expect(theme.tokens.accent).toBe('green');
	});

	test('setThemeToken refuses a link on a protected theme but accepts literals', () => {
		const doc = new LoroDoc();
		seedBuiltin(doc, 'cozy');
		setThemeToken(doc, 'cozy', 'primary', 'link:accent'); // refused
		setThemeToken(doc, 'cozy', 'background', 'red'); // accepted
		doc.commit();
		const cozy = themes(doc).find((t) => t.id === 'cozy')!;
		expect(cozy.tokens.primary).toBeUndefined();
		expect(cozy.tokens.background).toBe('red');
	});

	test('setThemeToken accepts a link on a non-protected theme', () => {
		const doc = new LoroDoc();
		createTheme(doc, 'theme-1', 'X', '', { primary: 'red' });
		setThemeToken(doc, 'theme-1', 'ring', 'link:primary');
		doc.commit();
		expect(themes(doc)[0].tokens.ring).toBe('link:primary');
	});

	test('two themes with the same name keep distinct ids (no collision)', () => {
		const doc = new LoroDoc();
		createTheme(doc, 'theme-a', 'Mine', '', {});
		createTheme(doc, 'theme-b', 'Mine', '', {});
		doc.commit();
		expect(themes(doc).map((theme) => theme.id)).toEqual(['theme-a', 'theme-b']);
	});

	test('deleteTheme removes an unprotected theme', () => {
		const doc = new LoroDoc();
		createTheme(doc, 'theme-1', 'X', '', {});
		doc.commit();
		deleteTheme(doc, 'theme-1');
		doc.commit();
		expect(themes(doc)).toHaveLength(0);
	});

	test('deleteTheme refuses to remove a protected (built-in) theme', () => {
		const doc = new LoroDoc();
		seedBuiltin(doc, 'cozy');
		doc.commit();
		deleteTheme(doc, 'cozy');
		doc.commit();
		expect(themes(doc).map((theme) => theme.id)).toEqual(['cozy']);
	});

	test('exportTheme returns the plain shape (no protected flag), or null when missing', () => {
		const doc = new LoroDoc();
		seedBuiltin(doc, 'editorial');
		createTheme(doc, 'theme-1', 'X', 'editorial', { primary: 'red' });
		doc.commit();
		expect(exportTheme(doc, 'theme-1')).toEqual({
			name: 'X',
			base: 'editorial',
			tokens: { primary: 'red' }
		});
		expect(exportTheme(doc, 'missing')).toBeNull();
	});
});
