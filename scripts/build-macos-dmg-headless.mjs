#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import {
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  symlinkSync
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const args = parseArgs(process.argv.slice(2));

if (args.has('help') || args.has('h')) {
  printHelp();
  process.exit(0);
}

if (process.platform !== 'darwin') {
  fail('Headless macOS DMG packaging is only supported on macOS because it requires hdiutil.');
}

const npmCommand = process.platform === 'win32' ? 'npm.cmd' : 'npm';
const tauriConfig = readJson(path.join(repoRoot, 'apps', 'desktop', 'src-tauri', 'tauri.conf.json'));
const appName = stringArg('app-name') ?? tauriConfig.productName ?? '900CRM';
const appVersion = stringArg('app-version') ?? tauriConfig.version ?? '0.0.0';
const arch = stringArg('arch') ?? macArchLabel(process.arch);
const skipBuild = args.has('skip-build') || process.env.HEADLESS_DMG_SKIP_BUILD === '1';
const bundleRoot = path.join(repoRoot, 'target', 'release', 'bundle');
const appBundlePath = resolveFromRepo(
  stringArg('app-bundle') ?? path.join('target', 'release', 'bundle', 'macos', `${appName}.app`)
);
const outputPath = resolveFromRepo(
  stringArg('out') ??
    path.join(
      'target',
      'release',
      'bundle',
      'dmg',
      `${sanitizeFileName(appName)}_${sanitizeFileName(appVersion)}_${sanitizeFileName(arch)}.headless.dmg`
    )
);
const stagingRoot = path.join(bundleRoot, 'dmg', '.headless-staging');
const stagingAppPath = path.join(stagingRoot, `${appName}.app`);

console.log('Building a local headless macOS DMG package for maintainer evidence.');
console.log('This helper does not sign, notarize, publish, tag, attach release assets, or run Finder/AppleScript customization.');

if (!skipBuild) {
  runCommand(npmCommand, ['--workspace', 'apps/desktop', 'run', 'tauri', '--', 'build', '--bundles', 'app', '--no-sign'], {
    label: 'Build macOS .app bundle'
  });
} else {
  console.log('\nSkipping Tauri app build because --skip-build or HEADLESS_DMG_SKIP_BUILD=1 was set.');
}

if (!existsSync(appBundlePath)) {
  fail(`Expected app bundle was not found: ${toDisplayPath(appBundlePath)}`);
}

mkdirSync(path.dirname(outputPath), { recursive: true });
rmSync(stagingRoot, { recursive: true, force: true });
mkdirSync(stagingRoot, { recursive: true });

try {
  runCommand('ditto', [appBundlePath, stagingAppPath], {
    label: 'Copy app bundle into headless DMG staging directory'
  });
  symlinkSync('/Applications', path.join(stagingRoot, 'Applications'));

  runCommand(
    'hdiutil',
    ['create', '-volname', appName, '-srcfolder', stagingRoot, '-ov', '-format', 'UDZO', '-fs', 'HFS+', outputPath],
    {
      label: 'Create compressed headless DMG'
    }
  );

  runCommand('hdiutil', ['verify', outputPath], {
    label: 'Verify headless DMG'
  });
} finally {
  rmSync(stagingRoot, { recursive: true, force: true });
}

const stats = statSync(outputPath);
const sha256 = await sha256File(outputPath);

console.log('\nHeadless macOS DMG created.');
console.log(`Output: ${toDisplayPath(outputPath)}`);
console.log(`Size: ${stats.size} bytes (${formatMiB(stats.size)} MiB)`);
console.log(`SHA-256: ${sha256}`);
console.log('\nBoundary: local unsigned and unnotarized macOS package evidence only; not workflow artifact proof, not Windows/Linux proof, not GitHub Release proof, and not release completion.');

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
  console.log(`Build a local headless macOS DMG from the Tauri app bundle.

Usage:
  node scripts/build-macos-dmg-headless.mjs [options]

Options:
  --skip-build             Reuse an existing target/release/bundle/macos/900CRM.app.
  --app-bundle <path>      App bundle to package. Defaults to target/release/bundle/macos/900CRM.app.
  --out <path>             Output DMG path. Defaults to target/release/bundle/dmg/900CRM_<version>_<arch>.headless.dmg.
  --app-name <value>       Volume/app name. Defaults to tauri.conf.json productName.
  --app-version <value>    Version label. Defaults to tauri.conf.json version.
  --arch <value>           Filename architecture label. Defaults to the local macOS architecture.
  --help, -h               Show this help.

This is maintainer-local package evidence only. The output is unsigned,
unnotarized, not generated by the release workflow, and not release completion.
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

function runCommand(command, commandArgs, { label }) {
  console.log(`\n${label}`);
  console.log(`$ ${[command, ...commandArgs.map(quoteArg)].join(' ')}`);

  const result = spawnSync(command, commandArgs, {
    cwd: repoRoot,
    env: process.env,
    stdio: 'inherit'
  });

  if (result.error) {
    fail(`Could not start ${label}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    if (result.signal) {
      fail(`${label} was terminated by signal ${result.signal}.`);
    }
    fail(`${label} exited with status ${result.status}.`);
  }
}

function quoteArg(value) {
  if (/^[A-Za-z0-9_./:=@+-]+$/.test(value)) {
    return value;
  }
  return JSON.stringify(value);
}

function macArchLabel(value) {
  if (value === 'arm64') {
    return 'aarch64';
  }
  if (value === 'x64') {
    return 'x64';
  }
  return value;
}

function sanitizeFileName(value) {
  return String(value).replace(/[^A-Za-z0-9._-]+/g, '-');
}

function formatMiB(bytes) {
  return (bytes / 1024 / 1024).toFixed(2);
}

async function sha256File(filePath) {
  const hash = createHash('sha256');
  await new Promise((resolve, reject) => {
    const stream = createReadStream(filePath);
    stream.on('error', reject);
    stream.on('data', (chunk) => hash.update(chunk));
    stream.on('end', resolve);
  });
  return hash.digest('hex');
}

function toDisplayPath(value) {
  const relative = path.relative(repoRoot, value);
  return relative.startsWith('..') ? value : relative.split(path.sep).join('/');
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
