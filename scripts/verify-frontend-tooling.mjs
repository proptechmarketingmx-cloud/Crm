import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const packageSpecs = [
  '@sveltejs/kit',
  '@sveltejs/vite-plugin-svelte',
  'svelte',
  'svelte-check',
  'vite',
  'vitest'
];

const binSpecs = ['svelte-kit', 'svelte-check', 'vite', 'vitest'];

function assertInsideRepo(label, resolvedPath) {
  const relative = path.relative(repoRoot, resolvedPath);
  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error(`${label} resolves outside this repository: ${resolvedPath}`);
  }
}

export function verifyFrontendTooling() {
  const resolved = [];
  const binPaths = {};

  for (const spec of packageSpecs) {
    const packagePath = path.join(repoRoot, 'node_modules', ...spec.split('/'), 'package.json');
    if (!fs.existsSync(packagePath)) {
      throw new Error(`${spec} is not installed at ${packagePath}`);
    }
    assertInsideRepo(spec, packagePath);
    resolved.push(`${spec} -> ${path.relative(repoRoot, packagePath)}`);
  }

  for (const bin of binSpecs) {
    const binPath = path.join(repoRoot, 'node_modules', '.bin', bin);
    if (!fs.existsSync(binPath)) {
      throw new Error(`${bin} is not installed at ${binPath}`);
    }
    assertInsideRepo(bin, binPath);
    const realBinPath = fs.realpathSync(binPath);
    assertInsideRepo(`${bin} target`, realBinPath);
    binPaths[bin] = realBinPath;
    resolved.push(`${bin} -> ${path.relative(repoRoot, binPath)} -> ${path.relative(repoRoot, realBinPath)}`);
  }

  return { repoRoot, resolved, binPaths };
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const { resolved } = verifyFrontendTooling();
  console.log('Frontend tooling resolves inside this repository:');
  for (const entry of resolved) {
    console.log(`- ${entry}`);
  }
}
