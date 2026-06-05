import { afterEach, expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import LoginForm from './LoginForm.svelte';

afterEach(() => vi.unstubAllGlobals());

// Stub `fetch` with a status decided per URL, recording the request paths made.
function stubFetch(statusFor: (url: string) => number): string[] {
	const calls: string[] = [];
	vi.stubGlobal(
		'fetch',
		vi.fn(async (url: string) => {
			calls.push(url);
			return new Response(null, { status: statusFor(url) });
		})
	);
	return calls;
}

test('the email-code flow posts both steps and authenticates on success', async () => {
	const calls = stubFetch(() => 200);
	const onAuthenticated = vi.fn();
	render(LoginForm, { onAuthenticated });

	// Step 1: request a code.
	await page.getByRole('textbox', { name: 'Email' }).fill('me@example.com');
	await page.getByRole('button', { name: 'Send code' }).click();

	// Step 2: the code field appears; submit a code.
	await page.getByRole('textbox', { name: 'Verification code' }).fill('ABCD2345');
	await page.getByRole('button', { name: 'Sign in' }).click();

	await vi.waitFor(() => expect(onAuthenticated).toHaveBeenCalled());
	expect(calls).toEqual(['/auth/email', '/auth/email/verify']);
});

test('a rejected code shows an error and does not authenticate', async () => {
	stubFetch((url) => (url.endsWith('/verify') ? 401 : 200));
	const onAuthenticated = vi.fn();
	render(LoginForm, { onAuthenticated });

	await page.getByRole('textbox', { name: 'Email' }).fill('me@example.com');
	await page.getByRole('button', { name: 'Send code' }).click();
	await page.getByRole('textbox', { name: 'Verification code' }).fill('WRONGCOD');
	await page.getByRole('button', { name: 'Sign in' }).click();

	await expect.element(page.getByRole('alert')).toBeVisible();
	expect(onAuthenticated).not.toHaveBeenCalled();
});
