import { test, expect } from '@playwright/test';
import { PostgreSqlContainer, type StartedPostgreSqlContainer } from '@testcontainers/postgresql';
import { spawn, execFileSync, type ChildProcess } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

// Hermetic auth e2e (ADR-0010): this spec runs its OWN ephemeral Postgres and a
// cloud-mode relay, separate from the shared local-mode webServer the other specs use,
// because email-code login only exists in cloud mode. Needs Docker, like the Rust
// integration tests. The relay writes each issued code to a File sink we read to complete
// the sign-in without a real mailbox (no code ever crosses an HTTP surface).

const PORT = 4273;
const BASE = `http://127.0.0.1:${PORT}`;
// Playwright runs from the config dir (packages/app); the cargo workspace is two up.
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

// Poll the relay until it serves the login page (or time out with whatever it logged).
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

// Read the latest code the relay wrote for `email` to the File sink.
async function waitForCode(email: string, timeoutMs = 15_000): Promise<string> {
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
	throw new Error(`no code was written to the sink for ${email}`);
}

test.beforeAll(async () => {
	test.setTimeout(240_000); // a cold `cargo build` plus the Postgres pull can be slow

	// Make sure the relay binary exists (fast when already compiled).
	execFileSync('cargo', ['build', '-p', 'nexus-server'], { cwd: repoRoot, stdio: 'inherit' });

	pg = await new PostgreSqlContainer('postgres:16-alpine').start();
	dataDir = mkdtempSync(path.join(tmpdir(), 'nexus-e2e-'));
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

test('a streamer signs in with an email code and opens their workspace', async ({ page }) => {
	const email = 'streamer@example.com';

	await page.goto(`${BASE}/login`);

	// Step 1: request a code.
	await page.getByRole('textbox', { name: 'Email' }).fill(email);
	await page.getByRole('button', { name: 'Send code' }).click();

	// The relay mailed it to the File sink; read it back.
	const code = await waitForCode(email);
	expect(code).toMatch(/^[A-Z2-9]{8}$/);

	// Step 2: submit the code; the session lands us on the workspace picker.
	await page.getByRole('textbox', { name: 'Verification code' }).fill(code);
	await page.getByRole('button', { name: 'Sign in' }).click();

	// This first sign-in claims the bootstrapped workspace, so the picker lists it.
	await expect(page).toHaveURL(/\/workspaces$/);
	const workspace = page.getByRole('button', { name: /My Overlays/ });
	await expect(workspace).toBeVisible();

	// Opening it routes the editor to that workspace.
	await workspace.click();
	await expect(page).toHaveURL(/\/edit\?workspace=/);

	// The session cookie is set, and it authenticates an API call from the editor origin.
	const cookies = await page.context().cookies();
	expect(cookies.some((c) => c.name === 'nexus_session')).toBe(true);

	const meStatus = await page.evaluate(async () => {
		const res = await fetch('/auth/me', { credentials: 'include' });
		return res.status;
	});
	expect(meStatus).toBe(200);
});
