#!/usr/bin/env node
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const args = parseArgs(process.argv.slice(2));

if (args.has('help') || args.has('h')) {
  printHelp();
  process.exit(0);
}

const rootPackage = readJson(path.join(repoRoot, 'package.json'));
const tauriConfig =
  readOptionalJson(path.join(repoRoot, 'apps', 'desktop', 'src-tauri', 'tauri.conf.json')) ?? {};
const appName = stringArg('app-name') ?? tauriConfig.productName ?? displayPackageName(rootPackage.name) ?? '900CRM';
const releaseVersion = stringArg('release-version') ?? tauriConfig.version ?? rootPackage.version ?? '0.0.0';
const releaseTitle = stringArg('release-title') ?? `${appName} ${releaseVersion}`;
const outFile = resolveFromRepo(stringArg('out') ?? 'release-notes.md');

mkdirSync(path.dirname(outFile), { recursive: true });
writeFileSync(outFile, releaseNotes(releaseTitle));

console.log(`Release notes written to ${toDisplayPath(outFile)}`);

function releaseNotes(title) {
  return `${title}

This release was packaged by the manual release workflow.

Scope:
- Windows, macOS, and Linux Tauri bundles from the workflow matrix.
- SHA-256 checksum files and release metadata generated per platform.
- SPDX-shaped dependency inventory generated from package-lock.json and Cargo.lock.

Boundaries:
- Artifacts are not signed.
- macOS artifacts are not notarized.
- No update channel, telemetry, crash reporting, or credential handling is configured.
`;
}

function parseArgs(argv) {
  const parsed = new Map();

  for (let index = 0; index < argv.length; index += 1) {
    const entry = argv[index];
    if (!entry.startsWith('--')) {
      fail(`Unexpected argument: ${entry}`);
    }

    const withoutPrefix = entry.slice(2);
    const equalsIndex = withoutPrefix.indexOf('=');
    if (equalsIndex !== -1) {
      parsed.set(withoutPrefix.slice(0, equalsIndex), withoutPrefix.slice(equalsIndex + 1));
      continue;
    }

    const next = argv[index + 1];
    if (next === undefined || next.startsWith('--')) {
      parsed.set(withoutPrefix, true);
      continue;
    }

    parsed.set(withoutPrefix, next);
    index += 1;
  }

  return parsed;
}

function printHelp() {
  console.log(`Generate 900CRM release notes.

Usage:
  node scripts/generate-release-notes.mjs [options]

Options:
  --release-version <value>  Release/package version label. Defaults to the Tauri app version.
  --release-title <value>    Human-readable release title. Defaults to "<app name> <release version>".
  --out <file>               Output file. Defaults to release-notes.md.
  --help, -h                 Show this help.

Examples:
  node scripts/generate-release-notes.mjs --help
  node scripts/generate-release-notes.mjs --release-version 0.0.0-sample --release-title "900CRM sample release" --out dist/release-sample/release-notes.md
  node scripts/generate-release-notes.mjs --release-version 1.0.0 --out release-notes.md
`);
}

function stringArg(name) {
  const value = args.get(name);
  if (value === undefined || value === true || value === '') {
    return undefined;
  }
  return String(value);
}

function resolveFromRepo(value) {
  return path.isAbsolute(value) ? value : path.join(repoRoot, value);
}

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, 'utf8'));
}

function readOptionalJson(filePath) {
  try {
    return readJson(filePath);
  } catch (error) {
    if (error.code === 'ENOENT') {
      return undefined;
    }
    throw error;
  }
}

function displayPackageName(value) {
  if (typeof value !== 'string' || value.length === 0) {
    return undefined;
  }
  if (value.toLowerCase() === '900crm' || value.toLowerCase() === '@900crm/desktop') {
    return '900CRM';
  }
  return value;
}

function toDisplayPath(value) {
  const relative = path.relative(repoRoot, value);
  return relative.startsWith('..') ? value : relative.split(path.sep).join('/');
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
