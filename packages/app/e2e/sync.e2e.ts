import { expect, test } from '@playwright/test';

// The walking skeleton's headline proof: a scene activation in one editor tab
// flows through the relay and reaches another tab. State-robust (it activates
// whichever scene is not currently active), so it does not depend on the relay
// starting from a fresh workspace.
test('a scene activation propagates from one tab to another', async ({ browser }) => {
	const alice = await browser.newPage();
	const bob = await browser.newPage();
	await alice.goto('/edit');
	await bob.goto('/edit');

	const aliceCards = alice.locator('.scene-card');
	const bobCards = bob.locator('.scene-card');
	await expect(aliceCards).toHaveCount(4);
	await expect(bobCards).toHaveCount(4);

	// Find the scene currently active in Alice's tab, then target a different one.
	let activeIndex = 0;
	for (let i = 0; i < 4; i++) {
		if ((await aliceCards.nth(i).getAttribute('aria-checked')) === 'true') {
			activeIndex = i;
			break;
		}
	}
	const target = (activeIndex + 1) % 4;

	// Alice activates the target scene; it must light up in Bob's tab via the relay.
	await aliceCards.nth(target).click();
	await expect(bobCards.nth(target)).toHaveAttribute('aria-checked', 'true');
	await expect(aliceCards.nth(target)).toHaveAttribute('aria-checked', 'true');
});
