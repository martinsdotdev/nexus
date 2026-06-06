import { expect, test } from 'vitest';
import { emailError, loginCodeError, workspaceNameError } from './validators';

test('emailError flags malformed addresses and passes valid ones', () => {
	expect(emailError('not-an-email')).toBeTruthy();
	expect(emailError('a@b')).toBeTruthy();
	expect(emailError('')).toBeTruthy();
	expect(emailError('me@example.com')).toBeUndefined();
});

test('loginCodeError requires the eight-character code alphabet', () => {
	expect(loginCodeError('ABC')).toBeTruthy();
	expect(loginCodeError('ABCD23451')).toBeTruthy(); // nine characters
	expect(loginCodeError('ABCD0001')).toBeTruthy(); // 0 and 1 are not in the alphabet
	expect(loginCodeError('ABCD2345')).toBeUndefined();
	expect(loginCodeError('abcd2345')).toBeUndefined(); // folded to uppercase first
});

test('workspaceNameError requires a non-empty, reasonably short name', () => {
	expect(workspaceNameError('')).toBeTruthy();
	expect(workspaceNameError('   ')).toBeTruthy();
	expect(workspaceNameError('x'.repeat(61))).toBeTruthy();
	expect(workspaceNameError('My Overlays')).toBeUndefined();
});
