import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { runCommand } from './run-command-with-timeout.mjs';
import { verifyFrontendTooling } from './verify-frontend-tooling.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const appDir = path.join(repoRoot, 'apps', 'desktop');
const binDir = path.join(repoRoot, 'node_modules', '.bin');
const timeoutMs = Number(process.env.FRONTEND_CHECK_TIMEOUT_MS ?? 30_000);
const env = {
  ...process.env,
  PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ''}`
};

try {
  const { resolved, binPaths } = verifyFrontendTooling();
  console.log('Frontend tooling resolution:');
  for (const entry of resolved) {
    console.log(`- ${entry}`);
  }

  await runCommand(process.execPath, [binPaths['svelte-kit'], 'sync'], {
    cwd: appDir,
    env,
    timeoutMs,
    label: 'svelte-kit sync'
  });

  await runCommand(process.execPath, [binPaths['svelte-check'], '--tsconfig', './tsconfig.json'], {
    cwd: appDir,
    env,
    timeoutMs,
    label: 'svelte-check'
  });
} catch (error) {
  console.error(`Frontend check failed: ${error.message}`);
  process.exit(1);
}
