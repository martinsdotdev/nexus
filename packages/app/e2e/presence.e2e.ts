import { test, expect, type Browser, type Page } from '@playwright/test';
import { PostgreSqlContainer, type StartedPostgreSqlContainer } from '@testcontainers/postgresql';
import { spawn, execFileSync, type ChildProcess } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

// Hermetic two-browser presence e2e (ADR-0009): two signed-in members of one workspace,
// and one sees the other's live cursor. Spins up its own ephemeral Postgres + cloud-mode
// relay (separate from the shared local-mode webServer), like the auth e2e. Needs Docker.

const PORT = 4274;
const BASE = `http://127.0.0.1:${PORT}`;
const appDir = process.cwd();
const repoRoot = path.resolve(appDir, '..', '..');
const buildDir = path.join(appDir, 'build');
const binary = path.join(
	repoRoot,
	'target',
	'debug',
	`nexus-server${process.platform === 'win32' ? '.exe' : ''}`
);

let pg: StartedPostgreSqlContainer;
let relay: ChildProcess;
let dataDir: string;
let sinkPath: string;
let relayLog = '';

async function waitForServer(timeoutMs = 60_000) {
	const deadline = Date.now() + timeoutMs;
	while (Date.now() < deadline) {
		try {
			const res = await fetch(`${BASE}/login`);
			if (res.ok) return;
		} catch {
			// not listening yet
		}
		await new Promise((r) => setTimeout(r, 250));
	}
	throw new Error(`relay did not become ready:\n${relayLog}`);
}

// The latest code the relay wrote for `email` to the File sink (ANSI-tolerant).
async function readCode(email: string, timeoutMs = 15_000): Promise<string> {
	const deadline = Date.now() + timeoutMs;
	while (Date.now() < deadline) {
		if (existsSync(sinkPath)) {
			const lines = readFileSync(sinkPath, 'utf8').trim().split('\n').reverse();
			for (const line of lines) {
				const [to, code] = line.split('\t');
				if (to === email && code) return code;
			}
		}
		await new Promise((r) => setTimeout(r, 100));
	}
	throw new Error(`no code written to the sink for ${email}`);
}

// Sign a fresh email-code user in through the login UI (a real browser, so it sends the
// Secure cookies over loopback that an API client would drop). Returns their page, now on
// the workspace picker.
async function signIn(browser: Browser, email: string): Promise<Page> {
	const context = await browser.newContext({ baseURL: BASE });
	const page = await context.newPage();
	await page.goto(`${BASE}/login`);
	await page.getByRole('textbox', { name: 'Email' }).fill(email);
	await page.getByRole('button', { name: 'Send code' }).click();
	const code = await readCode(email);
	await page.getByRole('textbox', { name: 'Verification code' }).fill(code);
	await page.getByRole('button', { name: 'Sign in' }).click();
	await page.waitForURL(/\/workspaces/);
	return page;
}

test.beforeAll(async () => {
	test.setTimeout(240_000);
	execFileSync('cargo', ['build', '-p', 'nexus-server'], { cwd: repoRoot, stdio: 'inherit' });

	pg = await new PostgreSqlContainer('postgres:16-alpine').start();
	dataDir = mkdtempSync(path.join(tmpdir(), 'nexus-presence-e2e-'));
	sinkPath = path.join(dataDir, 'codes.log');

	relay = spawn(
		binary,
		[
			'serve',
			'--host',
			'127.0.0.1',
			'--port',
			String(PORT),
			'--static-dir',
			buildDir,
			'--data-dir',
			dataDir,
			'--database-url',
			pg.getConnectionUri(),
			'--email-sink',
			sinkPath
		],
		{ cwd: repoRoot, stdio: ['ignore', 'pipe', 'pipe'] }
	);
	relay.stdout?.on('data', (d) => (relayLog += String(d)));
	relay.stderr?.on('data', (d) => (relayLog += String(d)));
	await waitForServer();
});

test.afterAll(async () => {
	relay?.kill();
	await pg?.stop();
	if (dataDir) rmSync(dataDir, { recursive: true, force: true });
});

test('a member sees another collaborator’s live cursor', async ({ browser }) => {
	// Two signed-in users; Bob signs in first so his account exists to be invited.
	const bobPage = await signIn(browser, 'bob@example.com');
	const alicePage = await signIn(browser, 'alice@example.com');

	// Alice creates a workspace and invites Bob. Same-origin fetches from her page carry
	// her session cookie and the Sec-Fetch-Site header the CSRF guard checks.
	const id = await alicePage.evaluate(async () => {
		const res = await fetch('/workspaces', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			credentials: 'include',
			body: JSON.stringify({ name: 'Team' })
		});
		return ((await res.json()) as { id: string }).id;
	});
	const inviteStatus = await alicePage.evaluate(async (workspaceId) => {
		const res = await fetch(`/workspaces/${workspaceId}/members`, {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			credentials: 'include',
			body: JSON.stringify({ email: 'bob@example.com' })
		});
		return res.status;
	}, id);
	expect(inviteStatus).toBe(204);

	// Both open the same workspace.
	await alicePage.goto(`${BASE}/edit?workspace=${id}`);
	await bobPage.goto(`${BASE}/edit?workspace=${id}`);
	await expect(alicePage.locator('.canvas-stage')).toBeVisible();
	await expect(bobPage.locator('.canvas-stage')).toBeVisible();

	// Alice moves her cursor across the canvas, which publishes her presence.
	const stage = alicePage.locator('.canvas-stage');
	await stage.hover({ position: { x: 300, y: 200 } });
	await stage.hover({ position: { x: 360, y: 260 } });
	await stage.hover({ position: { x: 420, y: 300 } });

	// Bob sees Alice's cursor and her roster chip (her display handle is the email local
	// part, "alice").
	await expect(bobPage.getByTestId('remote-cursors').locator('.cursor')).toHaveCount(1, {
		timeout: 15_000
	});
	// The pointer must actually be drawn on-screen, not merely present in the DOM: a
	// count-only check would pass even if a regression collapsed the graphic to zero width
	// or positioned it off-canvas. Assert the arrow is visible and inside the viewport.
	const aliceArrow = bobPage.getByTestId('remote-cursors').locator('.arrow');
	await expect(aliceArrow).toBeVisible();
	await expect(aliceArrow).toBeInViewport();
	await expect(bobPage.getByText('alice')).toBeVisible();
	await expect(bobPage.getByTestId('roster')).toBeVisible();

	await alicePage.context().close();
	await bobPage.context().close();
});
