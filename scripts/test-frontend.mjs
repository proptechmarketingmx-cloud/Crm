import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { runCommand } from './run-command-with-timeout.mjs';
import { verifyFrontendTooling } from './verify-frontend-tooling.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const appDir = path.join(repoRoot, 'apps', 'desktop');
const binDir = path.join(repoRoot, 'node_modules', '.bin');
const timeoutMs = Number(process.env.FRONTEND_TEST_TIMEOUT_MS ?? 30_000);
const env = {
  ...process.env,
  PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ''}`
};

try {
  const { binPaths } = verifyFrontendTooling();
  await runCommand(process.execPath, [binPaths.vitest, 'run', 'src/lib', '--passWithNoTests'], {
    cwd: appDir,
    env,
    timeoutMs,
    label: 'vitest lib tests'
  });
} catch (error) {
  console.error(`Frontend tests failed: ${error.message}`);
  process.exit(1);
}
