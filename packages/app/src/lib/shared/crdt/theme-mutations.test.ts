import { describe, expect, test } from 'vitest';
import { LoroDoc } from 'loro-crdt';
import { createTheme, deleteTheme, exportTheme, renameTheme, setThemeToken } from './mutations';
import { readWorkspace } from './workspace-view';

const customThemes = (doc: LoroDoc) => readWorkspace(doc).customThemes;

describe('theme registry mutations', () => {
	test('createTheme registers a theme readable via the read model', () => {
		const doc = new LoroDoc();
		createTheme(doc, 'theme-1', 'My Cozy', 'cozy', { primary: 'red', accent: 'blue' });
		doc.commit();
		expect(customThemes(doc)).toEqual([
			{ id: 'theme-1', name: 'My Cozy', base: 'cozy', tokens: { primary: 'red', accent: 'blue' } }
		]);
	});

	test('setThemeToken and renameTheme round-trip', () => {
		const doc = new LoroDoc();
		createTheme(doc, 'theme-1', 'X', 'cyber', { primary: 'red' });
		setThemeToken(doc, 'theme-1', 'accent', 'green');
		renameTheme(doc, 'theme-1', 'Renamed');
		doc.commit();
		const theme = customThemes(doc)[0];
		expect(theme.name).toBe('Renamed');
		expect(theme.tokens.accent).toBe('green');
	});

	test('two themes with the same name keep distinct ids (no collision)', () => {
		const doc = new LoroDoc();
		createTheme(doc, 'theme-a', 'Mine', 'cozy', {});
		createTheme(doc, 'theme-b', 'Mine', 'cozy', {});
		doc.commit();
		expect(customThemes(doc).map((theme) => theme.id)).toEqual(['theme-a', 'theme-b']);
	});

	test('deleteTheme removes it', () => {
		const doc = new LoroDoc();
		createTheme(doc, 'theme-1', 'X', 'cozy', {});
		doc.commit();
		deleteTheme(doc, 'theme-1');
		doc.commit();
		expect(customThemes(doc)).toHaveLength(0);
	});

	test('exportTheme returns the plain shape, or null when missing', () => {
		const doc = new LoroDoc();
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
