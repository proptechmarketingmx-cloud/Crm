#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { createReadStream, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const defaultArtifactDir = path.join(repoRoot, 'target', 'release', 'bundle');
const releaseArtifactSuffixes = [
  '.msi',
  '.exe',
  '.dmg',
  '.deb',
  '.AppImage',
  '.rpm',
  '.tar.gz',
  '.zip'
];
const ignoredReleaseArtifactPrefixes = ['rw.'];

const args = parseArgs(process.argv.slice(2));
if (args.has('help') || args.has('h')) {
  printHelp();
  process.exit(0);
}

const rootPackage = readJson(path.join(repoRoot, 'package.json'));
const tauriConfig =
  readOptionalJson(path.join(repoRoot, 'apps', 'desktop', 'src-tauri', 'tauri.conf.json')) ?? {};
const appName = stringArg('app-name') ?? tauriConfig.productName ?? rootPackage.name ?? '900CRM';
const appVersion =
  stringArg('app-version') ?? stringArg('release-version') ?? tauriConfig.version ?? rootPackage.version ?? '0.0.0';
const appIdentifier = stringArg('app-identifier') ?? tauriConfig.identifier ?? 'unknown';
const releaseVersion = stringArg('release-version') ?? appVersion;
const platform = stringArg('platform') ?? process.env.RUNNER_OS?.toLowerCase() ?? process.platform;
const releaseRef = stringArg('release-ref') ?? process.env.GITHUB_REF_NAME ?? gitValue(['rev-parse', '--abbrev-ref', 'HEAD']);
const releaseSha = stringArg('release-sha') ?? process.env.GITHUB_SHA ?? gitValue(['rev-parse', 'HEAD']);
const releaseTitle = stringArg('release-title') ?? `${appName} ${releaseVersion}`;
const repository = stringArg('repository') ?? repositoryUrl(rootPackage.repository);
const generatedAt = normalizeGeneratedAt(stringArg('generated-at') ?? process.env.SOURCE_DATE_EPOCH);
const outDir = resolveFromRepo(stringArg('out-dir') ?? path.join('dist', 'release-assets', platform));
const sampleMode = args.has('sample');
const artifactDir = sampleMode ? createSampleArtifactDir(outDir) : resolveFromRepo(stringArg('artifact-dir') ?? defaultArtifactDir);
const filePrefix = sanitizeFileName(`900CRM-${releaseVersion}-${platform}`);

if (!existsSync(artifactDir)) {
  fail(`Artifact directory does not exist: ${artifactDir}`);
}

mkdirSync(outDir, { recursive: true });

const artifactPaths = collectReleaseArtifacts(artifactDir);
if (artifactPaths.length === 0) {
  fail(
    `No release artifacts found in ${artifactDir}. Expected files ending in: ${releaseArtifactSuffixes.join(', ')}`
  );
}

const artifacts = [];
for (const artifactPath of artifactPaths) {
  const relativePath = toPosixPath(path.relative(artifactDir, artifactPath));
  const stats = statSync(artifactPath);
  artifacts.push({
    fileName: path.basename(artifactPath),
    relativePath,
    kind: artifactKind(relativePath),
    sizeBytes: stats.size,
    sha256: await sha256File(artifactPath)
  });
}

artifacts.sort((a, b) => a.relativePath.localeCompare(b.relativePath));

const sbom = buildSpdxSbom({
  name: `${appName} ${releaseVersion} dependency inventory`,
  releaseVersion,
  releaseRef,
  releaseSha,
  repository,
  generatedAt
});

const checksumsFileName = `${filePrefix}-SHA256SUMS.txt`;
const metadataFileName = `${filePrefix}-release-metadata.json`;
const sbomFileName = `${filePrefix}-sbom.spdx.json`;

const metadata = {
  schemaVersion: 1,
  app: {
    name: appName,
    version: appVersion,
    identifier: appIdentifier
  },
  product: appName,
  releaseVersion,
  releaseTitle,
  releaseRef,
  releaseSha,
  gitSha: releaseSha,
  generated_at: generatedAt,
  git_sha: releaseSha,
  repository,
  platform,
  generatedAt,
  artifactRoot: toDisplayPath(artifactDir),
  artifacts,
  generatedFiles: {
    checksums: checksumsFileName,
    sbom: sbomFileName
  },
  releaseBoundaries: {
    signed: false,
    notarized: false,
    updateChannel: false,
    telemetry: false,
    crashReporting: false
  }
};

writeFileSync(
  path.join(outDir, checksumsFileName),
  `${artifacts.map((artifact) => `${artifact.sha256}  ${artifact.relativePath}`).join('\n')}\n`
);
writeJson(path.join(outDir, metadataFileName), metadata);
writeJson(path.join(outDir, sbomFileName), sbom);

console.log(`Release metadata written to ${toDisplayPath(path.join(outDir, metadataFileName))}`);
console.log(`Checksums written to ${toDisplayPath(path.join(outDir, checksumsFileName))}`);
console.log(`SBOM written to ${toDisplayPath(path.join(outDir, sbomFileName))}`);
console.log(`Indexed ${artifacts.length} release artifact(s).`);

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
  console.log(`Generate 900CRM release checksums, metadata, and SBOM files.

Usage:
  node scripts/generate-release-manifest.mjs [options]

Options:
  --artifact-dir <dir>       Directory containing release artifacts. Defaults to the Tauri bundle output.
  --out-dir <dir>            Directory for generated metadata files. Defaults to dist/release-assets/<platform>.
  --platform <name>          Platform label, for example windows, macos, linux, or local.
  --release-version <value>  Release/package version label. Defaults to the Tauri app version.
  --release-title <value>    Human-readable release title. Defaults to "<app name> <release version>".
  --release-ref <value>      Git ref or tag name to record. Defaults to the current branch/ref when available.
  --release-sha <value>      Git commit SHA to record. Defaults to the current HEAD when available.
  --generated-at <value>     ISO timestamp or SOURCE_DATE_EPOCH-style integer timestamp.
  --sample                   Create a tiny sample artifact under --out-dir and generate metadata from it.
  --help, -h                 Show this help.

Examples:
  node scripts/generate-release-manifest.mjs --help
  node scripts/generate-release-manifest.mjs --sample --out-dir /tmp/900crm-release-sample --platform local
  node scripts/generate-release-manifest.mjs --artifact-dir target/release/bundle --out-dir release-assets/linux --platform linux --release-version 1.0.0
`);
}

function stringArg(name) {
  const value = args.get(name);
  if (value === undefined || value === true) {
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
  if (!existsSync(filePath)) {
    return undefined;
  }
  return readJson(filePath);
}

function writeJson(filePath, value) {
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function normalizeGeneratedAt(value) {
  if (value === undefined) {
    return new Date().toISOString();
  }

  if (/^\d+$/.test(value)) {
    return new Date(Number(value) * 1000).toISOString();
  }

  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    fail(`Invalid --generated-at value: ${value}`);
  }
  return parsed.toISOString();
}

function repositoryUrl(repositoryValue) {
  if (typeof repositoryValue === 'string') {
    return repositoryValue;
  }
  if (repositoryValue && typeof repositoryValue.url === 'string') {
    return repositoryValue.url;
  }
  return 'https://github.com/900Labs/900CRM';
}

function gitValue(gitArgs) {
  try {
    return execFileSync('git', gitArgs, {
      cwd: repoRoot,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore']
    }).trim();
  } catch {
    return 'unknown';
  }
}

function createSampleArtifactDir(baseOutDir) {
  const sampleDir = path.join(baseOutDir, 'sample-artifacts');
  rmSync(sampleDir, { recursive: true, force: true });
  mkdirSync(sampleDir, { recursive: true });
  writeFileSync(
    path.join(sampleDir, `${sanitizeFileName(`900CRM-${releaseVersion}-${platform}`)}-sample.zip`),
    `${appName} sample release artifact\nversion=${releaseVersion}\nplatform=${platform}\n`
  );
  return sampleDir;
}

function collectReleaseArtifacts(rootDir) {
  const collected = [];
  const pending = [rootDir];

  while (pending.length > 0) {
    const current = pending.pop();
    const entries = readSortedDirEntries(current);

    for (const entry of entries) {
      const fullPath = path.join(current, entry.name);
      if (entry.isDirectory()) {
        pending.push(fullPath);
        continue;
      }

      if (entry.isFile() && isReleaseArtifact(fullPath)) {
        collected.push(fullPath);
      }
    }
  }

  return collected.sort((a, b) => toPosixPath(path.relative(rootDir, a)).localeCompare(toPosixPath(path.relative(rootDir, b))));
}

function readSortedDirEntries(dirPath) {
  return readFileSystemDir(dirPath).sort((a, b) => a.name.localeCompare(b.name));
}

function readFileSystemDir(dirPath) {
  try {
    return readdirSync(dirPath, { withFileTypes: true });
  } catch (error) {
    fail(`Unable to read ${dirPath}: ${error.message}`);
  }
}

function isReleaseArtifact(filePath) {
  const fileName = path.basename(filePath);
  if (ignoredReleaseArtifactPrefixes.some((prefix) => fileName.startsWith(prefix))) {
    return false;
  }

  return releaseArtifactSuffixes.some((suffix) => filePath.endsWith(suffix));
}

function artifactKind(relativePath) {
  if (relativePath.endsWith('.msi')) {
    return 'windows-msi-installer';
  }
  if (relativePath.endsWith('.exe')) {
    return 'windows-nsis-installer';
  }
  if (relativePath.endsWith('.dmg')) {
    return 'macos-dmg';
  }
  if (relativePath.endsWith('.deb')) {
    return 'linux-deb';
  }
  if (relativePath.endsWith('.AppImage')) {
    return 'linux-appimage';
  }
  if (relativePath.endsWith('.rpm')) {
    return 'linux-rpm';
  }
  if (relativePath.endsWith('.tar.gz') || relativePath.endsWith('.zip')) {
    return 'archive';
  }
  return 'release-artifact';
}

function sha256File(filePath) {
  return new Promise((resolve, reject) => {
    const hash = createHash('sha256');
    const stream = createReadStream(filePath);

    stream.on('data', (chunk) => hash.update(chunk));
    stream.on('error', reject);
    stream.on('end', () => resolve(hash.digest('hex')));
  });
}

function buildSpdxSbom({ name, releaseVersion: version, releaseRef: ref, releaseSha: sha, repository: repo, generatedAt: created }) {
  const packageEntries = [
    {
      ecosystem: 'source',
      name: appName,
      version: appVersion ?? version,
      license: rootPackage.license,
      downloadLocation: repo,
      purl: `pkg:github/900Labs/900CRM@${sha}`
    },
    ...npmPackages(),
    ...cargoPackages()
  ];
  const spdxPackages = packageEntries.map((entry, index) => toSpdxPackage(entry, index));

  return {
    spdxVersion: 'SPDX-2.3',
    dataLicense: 'CC0-1.0',
    SPDXID: 'SPDXRef-DOCUMENT',
    name,
    documentNamespace: `https://github.com/900Labs/900CRM/releases/${encodeURIComponent(ref)}/sbom-${sanitizeFileName(version)}-${sanitizeFileName(sha)}`,
    creationInfo: {
      created,
      creators: ['Organization: 900 Labs', 'Tool: scripts/generate-release-manifest.mjs']
    },
    documentDescribes: spdxPackages.length > 0 ? [spdxPackages[0].SPDXID] : [],
    packages: spdxPackages
  };
}

function npmPackages() {
  const lockPath = path.join(repoRoot, 'package-lock.json');
  if (!existsSync(lockPath)) {
    return [];
  }

  const lock = readJson(lockPath);
  const entries = [];
  const seen = new Set();

  for (const [location, info] of Object.entries(lock.packages ?? {})) {
    if (location === '' || !info || typeof info !== 'object') {
      continue;
    }

    const name = typeof info.name === 'string' ? info.name : npmNameFromLocation(location);
    const version = typeof info.version === 'string' ? info.version : undefined;
    if (!name || !version) {
      continue;
    }

    const key = `npm:${name}@${version}:${info.resolved ?? location}`;
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);

    entries.push({
      ecosystem: 'npm',
      name,
      version,
      license: licenseString(info.license),
      downloadLocation: typeof info.resolved === 'string' ? info.resolved : 'NOASSERTION',
      checksum: typeof info.integrity === 'string' ? info.integrity : undefined,
      purl: `pkg:npm/${encodeURIComponent(name)}@${encodeURIComponent(version)}`
    });
  }

  return entries.sort(comparePackageEntries);
}

function npmNameFromLocation(location) {
  const marker = 'node_modules/';
  if (!location.includes(marker)) {
    return undefined;
  }

  const lastNodeModulesIndex = location.lastIndexOf(marker);
  const packagePath = location.slice(lastNodeModulesIndex + marker.length);
  const segments = packagePath.split('/');
  if (segments[0]?.startsWith('@') && segments.length >= 2) {
    return `${segments[0]}/${segments[1]}`;
  }
  return segments[0];
}

function cargoPackages() {
  const lockPath = path.join(repoRoot, 'Cargo.lock');
  if (!existsSync(lockPath)) {
    return [];
  }

  const blocks = readFileSync(lockPath, 'utf8').split(/\n(?=\[\[package\]\])/);
  const entries = [];

  for (const block of blocks) {
    if (!block.startsWith('[[package]]')) {
      continue;
    }

    const fields = {};
    for (const line of block.split('\n')) {
      const match = line.match(/^([A-Za-z0-9_-]+) = "([^"]*)"$/);
      if (match) {
        fields[match[1]] = match[2];
      }
    }

    if (!fields.name || !fields.version) {
      continue;
    }

    entries.push({
      ecosystem: 'cargo',
      name: fields.name,
      version: fields.version,
      license: 'NOASSERTION',
      downloadLocation: fields.source ?? 'NOASSERTION',
      checksum: fields.checksum,
      purl: `pkg:cargo/${encodeURIComponent(fields.name)}@${encodeURIComponent(fields.version)}`
    });
  }

  return entries.sort(comparePackageEntries);
}

function toSpdxPackage(entry, index) {
  const spdxPackage = {
    name: `${entry.ecosystem}:${entry.name}`,
    SPDXID: `SPDXRef-Package-${index}-${sanitizeSpdxId(entry.ecosystem)}-${sanitizeSpdxId(entry.name)}-${sanitizeSpdxId(entry.version)}`,
    versionInfo: entry.version,
    downloadLocation: entry.downloadLocation ?? 'NOASSERTION',
    filesAnalyzed: false,
    licenseConcluded: 'NOASSERTION',
    licenseDeclared: entry.license ?? 'NOASSERTION',
    copyrightText: 'NOASSERTION',
    supplier: 'NOASSERTION'
  };

  if (entry.purl) {
    spdxPackage.externalRefs = [
      {
        referenceCategory: 'PACKAGE-MANAGER',
        referenceType: 'purl',
        referenceLocator: entry.purl
      }
    ];
  }

  if (entry.checksum) {
    spdxPackage.comment = `Lockfile checksum/integrity: ${entry.checksum}`;
  }

  return spdxPackage;
}

function licenseString(value) {
  if (typeof value === 'string' && value.trim()) {
    return value;
  }
  if (Array.isArray(value) && value.length > 0) {
    return value.join(' OR ');
  }
  return 'NOASSERTION';
}

function comparePackageEntries(left, right) {
  return `${left.ecosystem}:${left.name}:${left.version}`.localeCompare(`${right.ecosystem}:${right.name}:${right.version}`);
}

function sanitizeFileName(value) {
  return String(value).replace(/[^A-Za-z0-9._-]+/g, '-').replace(/^-+|-+$/g, '') || 'unknown';
}

function sanitizeSpdxId(value) {
  return sanitizeFileName(value).replace(/\./g, '-');
}

function toPosixPath(value) {
  return value.split(path.sep).join('/');
}

function toDisplayPath(value) {
  const relative = path.relative(repoRoot, value);
  return relative.startsWith('..') ? value : toPosixPath(relative);
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
