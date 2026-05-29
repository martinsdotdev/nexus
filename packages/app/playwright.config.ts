import { defineConfig } from '@playwright/test';

export default defineConfig({
	// The relay holds one shared workspace, so e2e tests cannot run concurrently
	// against it without racing on activeSceneId. Serialize them; each test sets
	// up the scene state it asserts on rather than assuming the relay default.
	fullyParallel: false,
	workers: 1,
	webServer: {
		// Build the static UI, then run the relay serving it AND the /sync
		// WebSocket on one origin, so the e2e drives the real relay end to end.
		command:
			'node e2e/reset-data.mjs && pnpm build && cargo run -p nexus-server -- serve --port 4173 --static-dir build --data-dir .e2e-data',
		port: 4173,
		reuseExistingServer: !process.env.CI,
		timeout: 240_000
	},
	use: { baseURL: 'http://localhost:4173' },
	testMatch: '**/*.e2e.{ts,js}'
});
