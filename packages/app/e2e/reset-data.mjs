// Ensure each e2e run starts from a fresh relay default (the eight seeded
// widgets + per-scene themes built by nexus-core's default_doc). Without this,
// the relay loads the snapshot persisted by a previous run, which can predate
// the current default and make assertions about seeded content flaky.
import { rmSync } from 'node:fs';

rmSync(new URL('../.e2e-data', import.meta.url), { recursive: true, force: true });
