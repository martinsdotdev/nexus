import { defineConfig } from '@playwright/test';

export default defineConfig({
	webServer: {
		// Build the static UI, then run the relay serving it AND the /sync
		// WebSocket on one origin, so the e2e drives the real relay end to end.
		command:
			'pnpm build && cargo run -p nexus-server -- serve --port 4173 --static-dir build --data-dir .e2e-data',
		port: 4173,
		reuseExistingServer: !process.env.CI,
		timeout: 240_000
	},
	use: { baseURL: 'http://localhost:4173' },
	testMatch: '**/*.e2e.{ts,js}'
});
