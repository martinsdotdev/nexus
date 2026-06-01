import { expect, test, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ColorField from './ColorField.svelte';

// The wrapper is a generic color input (hex in, rgba out); the theme feature
// bridges oklch. Open the swatch trigger, type a hex, commit, expect an rgba.
test('opening the picker and committing a hex reports an rgba color', async () => {
	const onChange = vi.fn();
	render(ColorField, { value: '#ff0000', onChange, ariaLabel: 'primary' });

	await page.getByRole('button', { name: 'primary' }).click();
	const hex = page.getByRole('textbox', { name: 'Hex' });
	await hex.fill('00ff00');
	// Commit the channel input (Ark fires onValueChangeEnd on Enter / blur).
	const el = (await hex.element()) as HTMLInputElement;
	el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
	el.blur();

	await expect.poll(() => onChange.mock.calls.length).toBeGreaterThan(0);
	expect(onChange.mock.calls.at(-1)?.[0]).toMatch(/rgba?\(/);
});
