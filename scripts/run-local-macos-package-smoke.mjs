#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createReadStream, copyFileSync, existsSync, lstatSync, mkdirSync, readFileSync, rmSync, statSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const args = parseArgs(process.argv.slice(2));

if (args.has('help') || args.has('h')) {
  printHelp();
  process.exit(0);
}

if (process.platform !== 'darwin') {
  fail('Local macOS package smoke is only supported on macOS because it requires hdiutil.');
}

const tauriConfig = readJson(path.join(repoRoot, 'apps', 'desktop', 'src-tauri', 'tauri.conf.json'));
const appName = stringArg('app-name') ?? tauriConfig.productName ?? '900CRM';
const appVersion = stringArg('app-version') ?? tauriConfig.version ?? '0.0.0';
const arch = stringArg('arch') ?? macArchLabel(process.arch);
const skipBuild = args.has('skip-build') || process.env.LOCAL_MACOS_PACKAGE_SMOKE_SKIP_BUILD === '1';
const skipMount = args.has('skip-mount') || process.env.LOCAL_MACOS_PACKAGE_SMOKE_SKIP_MOUNT === '1';
const dmgPath = resolveFromRepo(
  stringArg('dmg') ??
    path.join(
      'target',
      'release',
      'bundle',
      'dmg',
      `${sanitizeFileName(appName)}_${sanitizeFileName(appVersion)}_${sanitizeFileName(arch)}.headless.dmg`
    )
);
const localSmokeRoot = path.join(repoRoot, 'dist', 'local-macos-package-smoke');
const releaseDownloadRoot = path.join(localSmokeRoot, 'release-download');
const packageRoot = path.join(releaseDownloadRoot, `900crm-${sanitizeFileName(appVersion)}-macos-packages`);
const packageDmgRoot = path.join(packageRoot, 'dmg');
const metadataRoot = path.join(releaseDownloadRoot, `900crm-${sanitizeFileName(appVersion)}-macos-release-metadata`);
const copiedDmgPath = path.join(packageDmgRoot, path.basename(dmgPath));
const mountPoint = path.join(tmpdir(), `900crm-local-macos-package-smoke-${process.pid}`);

console.log('Running local macOS package smoke for maintainer evidence.');
console.log('Boundary: local unsigned and unnotarized macOS package evidence only.');
console.log('This does not prove workflow artifacts, Windows/Linux packages, signing, notarization, publishing, or release completion.');

const headlessArgs = ['scripts/build-macos-dmg-headless.mjs', '--out', toDisplayPath(dmgPath), '--app-name', appName, '--app-version', appVersion, '--arch', arch];
if (skipBuild) {
  headlessArgs.push('--skip-build');
}
runCommand(process.execPath, headlessArgs, {
  label: 'Build and verify local headless macOS DMG'
});

if (!existsSync(dmgPath)) {
  fail(`Expected DMG was not found: ${toDisplayPath(dmgPath)}`);
}

runCommand('hdiutil', ['verify', dmgPath], {
  label: 'Verify local macOS DMG image'
});

rmSync(localSmokeRoot, { recursive: true, force: true });
mkdirSync(packageDmgRoot, { recursive: true });
copyFileSync(dmgPath, copiedDmgPath);

const releaseRef = gitValue(['rev-parse', '--abbrev-ref', 'HEAD']);
const releaseSha = gitValue(['rev-parse', 'HEAD']);
runCommand(
  process.execPath,
  [
    'scripts/generate-release-manifest.mjs',
    '--artifact-dir',
    toDisplayPath(packageRoot),
    '--out-dir',
    toDisplayPath(metadataRoot),
    '--platform',
    'macos',
    '--release-version',
    appVersion,
    '--release-title',
    '900CRM local macOS package smoke',
    '--release-ref',
    releaseRef,
    '--release-sha',
    releaseSha
  ],
  {
    label: 'Generate local macOS package metadata'
  }
);

runCommand(
  process.execPath,
  [
    'scripts/verify-release-artifacts.mjs',
    '--artifact-root',
    toDisplayPath(releaseDownloadRoot),
    '--release-version',
    appVersion,
    '--platforms',
    'macos'
  ],
  {
    label: 'Verify local macOS package metadata and checksums'
  }
);

if (!skipMount) {
  runMountedDmgSmoke();
} else {
  console.log('\nSkipping mounted-DMG layout smoke because --skip-mount or LOCAL_MACOS_PACKAGE_SMOKE_SKIP_MOUNT=1 was set.');
}

const stats = statSync(dmgPath);
const sha256 = await sha256File(dmgPath);

console.log('\nLocal macOS package smoke passed.');
console.log(`DMG: ${toDisplayPath(dmgPath)}`);
console.log(`DMG size: ${stats.size} bytes (${formatMiB(stats.size)} MiB)`);
console.log(`DMG SHA-256: ${sha256}`);
console.log(`Local artifact verification root: ${toDisplayPath(releaseDownloadRoot)}`);
console.log(`Copied package artifact: ${toDisplayPath(copiedDmgPath)}`);
console.log(`Metadata: ${toDisplayPath(metadataRoot)}`);
console.log(
  `Verifier: npm run release:artifacts:verify -- --artifact-root dist/local-macos-package-smoke/release-download --release-version ${appVersion} --platforms macos`
);
console.log('Boundary: local unsigned and unnotarized macOS package smoke only; not alpha release completion.');

function runMountedDmgSmoke() {
  rmSync(mountPoint, { recursive: true, force: true });
  mkdirSync(mountPoint, { recursive: true });

  let attached = false;
  try {
    runCommand('hdiutil', ['attach', '-nobrowse', '-readonly', '-mountpoint', mountPoint, dmgPath], {
      label: 'Mount local macOS DMG for layout smoke'
    });
    attached = true;

    const mountedAppPath = path.join(mountPoint, `${appName}.app`);
    if (!existsSync(mountedAppPath) || !lstatSync(mountedAppPath).isDirectory()) {
      fail(`Mounted DMG is missing ${appName}.app.`);
    }

    const applicationsPath = path.join(mountPoint, 'Applications');
    if (!existsSync(applicationsPath) || !lstatSync(applicationsPath).isSymbolicLink()) {
      fail('Mounted DMG is missing the Applications symlink.');
    }

    console.log(`Mounted-DMG layout smoke passed: ${appName}.app and Applications symlink are present.`);
  } finally {
    if (attached) {
      runCommand('hdiutil', ['detach', mountPoint], {
        label: 'Detach local macOS DMG smoke mount'
      });
    }
    rmSync(mountPoint, { recursive: true, force: true });
  }
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
  console.log(`Run a local macOS package smoke baseline.

Usage:
  node scripts/run-local-macos-package-smoke.mjs [options]

Options:
  --skip-build        Reuse an existing target/release/bundle/macos/900CRM.app.
  --skip-mount        Skip the mounted-DMG layout smoke.
  --dmg <path>        Expected/headless DMG path. Defaults to target/release/bundle/dmg/900CRM_<version>_<arch>.headless.dmg.
  --app-name <value>  App and volume name. Defaults to tauri.conf.json productName.
  --app-version <v>   Version label. Defaults to tauri.conf.json version.
  --arch <value>      Filename architecture label. Defaults to the local macOS architecture.
  --help, -h          Show this help.

The smoke builds a local unsigned/unnotarized headless DMG, verifies it with
hdiutil, generates local macOS release metadata/checksums/SBOM under ignored
dist/, verifies those generated files with the release artifact verifier, and
mounts the DMG to confirm the app bundle plus Applications symlink are present.
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

function gitValue(gitArgs) {
  try {
    const result = spawnSync('git', gitArgs, {
      cwd: repoRoot,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore']
    });
    if (result.status === 0) {
      return result.stdout.trim() || 'unknown';
    }
  } catch {
    // fall through
  }
  return 'unknown';
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
  return String(value).replace(/[^A-Za-z0-9._-]+/g, '-').replace(/^-+|-+$/g, '') || 'unknown';
}

function formatMiB(bytes) {
  return (bytes / 1024 / 1024).toFixed(2);
}

function sha256File(filePath) {
  const hash = createHash('sha256');
  return new Promise((resolve, reject) => {
    const stream = createReadStream(filePath);
    stream.on('error', reject);
    stream.on('data', (chunk) => hash.update(chunk));
    stream.on('end', () => resolve(hash.digest('hex')));
  });
}

function toDisplayPath(value) {
  const relative = path.relative(repoRoot, value);
  return relative.startsWith('..') ? value : relative.split(path.sep).join('/');
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
