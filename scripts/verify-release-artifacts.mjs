#!/usr/bin/env node
import { createHash } from 'node:crypto';
import {
  createReadStream,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const defaultArtifactRoot = path.join(repoRoot, 'release-download');
const sampleArtifactRoot = path.join(repoRoot, 'dist', 'release-artifact-verifier-sample', 'release-download');
const defaultPlatforms = ['windows', 'macos', 'linux'];
const packageExtensions = ['.msi', '.exe', '.dmg', '.deb', '.AppImage', '.rpm', '.tar.gz', '.zip'];
const metadataSuffix = '-release-metadata.json';
const checksumSuffix = '-SHA256SUMS.txt';
const sbomSuffix = '-sbom.spdx.json';

const args = parseArgs(process.argv.slice(2));
if (args.has('help') || args.has('h')) {
  printHelp();
  process.exit(0);
}

const sampleMode = args.has('sample');
const releaseVersion = stringArg('release-version') ?? (sampleMode ? '0.0.0-sample' : undefined);
const requiredPlatforms = csvArg('platforms') ?? defaultPlatforms;
const artifactRoot = resolveFromRepo(stringArg('artifact-root') ?? (sampleMode ? sampleArtifactRoot : defaultArtifactRoot));

if (sampleMode) {
  createSampleDownloadedArtifacts(artifactRoot, releaseVersion);
}

await verifyArtifactRoot({ artifactRoot, releaseVersion, requiredPlatforms });

async function verifyArtifactRoot({ artifactRoot: root, releaseVersion: expectedReleaseVersion, requiredPlatforms: platforms }) {
  if (!existsSync(root)) {
    fail(`Artifact root does not exist: ${toDisplayPath(root)}`);
  }

  const allFiles = collectFiles(root);
  const metadataFiles = allFiles.filter((filePath) => path.basename(filePath).endsWith(metadataSuffix));
  if (metadataFiles.length === 0) {
    fail(`No ${metadataSuffix} files found under ${toDisplayPath(root)}.`);
  }

  const metadataByPlatform = new Map();
  const summaries = [];

  for (const metadataPath of metadataFiles) {
    const metadata = readJson(metadataPath);
    const platform = validateMetadataEnvelope(metadata, metadataPath, expectedReleaseVersion);
    if (metadataByPlatform.has(platform)) {
      fail(`Multiple metadata files found for platform "${platform}".`);
    }
    metadataByPlatform.set(platform, metadataPath);

    const checksumPath = findNamedFile(allFiles, metadata.generatedFiles?.checksums, metadataPath, checksumSuffix);
    const sbomPath = findNamedFile(allFiles, metadata.generatedFiles?.sbom, metadataPath, sbomSuffix);

    validateChecksums(checksumPath, metadata);
    validateSbom(sbomPath);
    await validateArtifacts(root, allFiles, metadata);
    validateExpectedPackageKinds(metadata);

    summaries.push({
      platform,
      metadataPath,
      checksumPath,
      sbomPath,
      artifactCount: metadata.artifacts.length
    });
  }

  const missingPlatforms = platforms.filter((platform) => !metadataByPlatform.has(platform));
  if (missingPlatforms.length > 0) {
    fail(`Missing required platform metadata: ${missingPlatforms.join(', ')}.`);
  }

  summaries.sort((left, right) => left.platform.localeCompare(right.platform));
  for (const summary of summaries) {
    console.log(
      `Verified ${summary.platform}: ${summary.artifactCount} package artifact(s), ${toDisplayPath(summary.metadataPath)}, ${toDisplayPath(summary.checksumPath)}, ${toDisplayPath(summary.sbomPath)}`
    );
  }
  console.log(`Release artifact verification passed for ${platforms.join(', ')} under ${toDisplayPath(root)}.`);
}

function validateMetadataEnvelope(metadata, metadataPath, expectedReleaseVersion) {
  const label = toDisplayPath(metadataPath);

  if (!metadata || typeof metadata !== 'object' || Array.isArray(metadata)) {
    fail(`Metadata is not a JSON object: ${label}`);
  }
  if (metadata.schemaVersion !== 1) {
    fail(`Metadata schemaVersion must be 1 in ${label}.`);
  }
  if (!isNonEmptyString(metadata.platform)) {
    fail(`Metadata platform must be a non-empty string in ${label}.`);
  }
  if (!isNonEmptyString(metadata.releaseVersion)) {
    fail(`Metadata releaseVersion must be a non-empty string in ${label}.`);
  }
  if (expectedReleaseVersion && metadata.releaseVersion !== expectedReleaseVersion) {
    fail(
      `Metadata releaseVersion mismatch in ${label}: expected ${expectedReleaseVersion}, found ${metadata.releaseVersion}.`
    );
  }
  if (!Array.isArray(metadata.artifacts) || metadata.artifacts.length === 0) {
    fail(`Metadata artifacts must be a non-empty array in ${label}.`);
  }

  const seenRelativePaths = new Set();
  for (const artifact of metadata.artifacts) {
    validateMetadataArtifact(artifact, label);
    if (seenRelativePaths.has(artifact.relativePath)) {
      fail(`Duplicate metadata artifact relativePath "${artifact.relativePath}" in ${label}.`);
    }
    seenRelativePaths.add(artifact.relativePath);
  }

  return metadata.platform;
}

function validateMetadataArtifact(artifact, metadataLabel) {
  if (!artifact || typeof artifact !== 'object' || Array.isArray(artifact)) {
    fail(`Metadata artifact entry is not an object in ${metadataLabel}.`);
  }

  for (const field of ['fileName', 'relativePath', 'kind']) {
    if (!isNonEmptyString(artifact[field])) {
      fail(`Metadata artifact field "${field}" must be a non-empty string in ${metadataLabel}.`);
    }
  }

  if (artifact.fileName !== path.posix.basename(artifact.relativePath)) {
    fail(`Metadata artifact fileName must match relativePath basename for ${artifact.relativePath}.`);
  }
  if (path.isAbsolute(artifact.relativePath) || artifact.relativePath.includes('\\')) {
    fail(`Metadata artifact relativePath must be a relative POSIX path: ${artifact.relativePath}.`);
  }
  if (artifact.relativePath.split('/').includes('..')) {
    fail(`Metadata artifact relativePath must not traverse directories: ${artifact.relativePath}.`);
  }
  if (!Number.isSafeInteger(artifact.sizeBytes) || artifact.sizeBytes <= 0) {
    fail(`Metadata artifact sizeBytes must be a positive safe integer for ${artifact.relativePath}.`);
  }
  if (!/^[a-f0-9]{64}$/i.test(artifact.sha256)) {
    fail(`Metadata artifact sha256 must be a 64-character hex digest for ${artifact.relativePath}.`);
  }
}

function validateChecksums(checksumPath, metadata) {
  const entries = parseChecksumFile(checksumPath);
  const expected = new Map(metadata.artifacts.map((artifact) => [artifact.relativePath, artifact.sha256.toLowerCase()]));

  if (entries.size !== expected.size) {
    fail(
      `Checksum entry count mismatch for ${metadata.platform}: expected ${expected.size}, found ${entries.size} in ${toDisplayPath(checksumPath)}.`
    );
  }

  for (const [relativePath, sha256] of expected) {
    const actual = entries.get(relativePath);
    if (actual === undefined) {
      fail(`Checksum file is missing metadata artifact ${relativePath}.`);
    }
    if (actual !== sha256) {
      fail(`Checksum mismatch for ${relativePath}: metadata ${sha256}, checksum file ${actual}.`);
    }
  }

  for (const relativePath of entries.keys()) {
    if (!expected.has(relativePath)) {
      fail(`Checksum file contains artifact not present in metadata: ${relativePath}.`);
    }
  }
}

function parseChecksumFile(checksumPath) {
  const entries = new Map();
  const text = readFileSync(checksumPath, 'utf8');
  for (const [index, line] of text.split(/\r?\n/).entries()) {
    if (!line.trim()) {
      continue;
    }
    const match = line.match(/^([a-f0-9]{64})\s+(.+)$/i);
    if (!match) {
      fail(`Invalid checksum line ${index + 1} in ${toDisplayPath(checksumPath)}.`);
    }
    const relativePath = toPosixPath(match[2].trim());
    if (entries.has(relativePath)) {
      fail(`Duplicate checksum entry for ${relativePath} in ${toDisplayPath(checksumPath)}.`);
    }
    entries.set(relativePath, match[1].toLowerCase());
  }
  return entries;
}

function validateSbom(sbomPath) {
  const sbom = readJson(sbomPath);
  if (!sbom || typeof sbom !== 'object' || Array.isArray(sbom)) {
    fail(`SBOM is not a JSON object: ${toDisplayPath(sbomPath)}.`);
  }
  if (sbom.spdxVersion !== 'SPDX-2.3') {
    fail(`SBOM spdxVersion must be SPDX-2.3 in ${toDisplayPath(sbomPath)}.`);
  }
  if (!Array.isArray(sbom.packages) || sbom.packages.length === 0) {
    fail(`SBOM packages must be a non-empty array in ${toDisplayPath(sbomPath)}.`);
  }
}

async function validateArtifacts(root, allFiles, metadata) {
  for (const artifact of metadata.artifacts) {
    const candidate = await findMatchingPackageFile(root, allFiles, artifact);
    const stats = statSync(candidate);
    if (stats.size !== artifact.sizeBytes) {
      fail(
        `Package size mismatch for ${artifact.relativePath}: metadata ${artifact.sizeBytes}, file ${stats.size} at ${toDisplayPath(candidate)}.`
      );
    }
    const sha256 = await sha256File(candidate);
    if (sha256 !== artifact.sha256.toLowerCase()) {
      fail(
        `Package sha256 mismatch for ${artifact.relativePath}: metadata ${artifact.sha256}, file ${sha256} at ${toDisplayPath(candidate)}.`
      );
    }
  }
}

async function findMatchingPackageFile(root, allFiles, artifact) {
  const relativePath = toPosixPath(artifact.relativePath);
  const candidateFiles = allFiles.filter((filePath) => {
    if (!isPackageFile(filePath)) {
      return false;
    }
    const rootRelative = toPosixPath(path.relative(root, filePath));
    return rootRelative === relativePath || rootRelative.endsWith(`/${relativePath}`) || path.basename(filePath) === artifact.fileName;
  });

  if (candidateFiles.length === 0) {
    fail(`Could not find package file for metadata artifact ${relativePath}.`);
  }

  for (const candidate of candidateFiles) {
    const stats = statSync(candidate);
    if (stats.size !== artifact.sizeBytes) {
      continue;
    }
    const sha256 = await sha256File(candidate);
    if (sha256 === artifact.sha256.toLowerCase()) {
      return candidate;
    }
  }

  fail(`Found package candidate(s) for ${relativePath}, but none matched metadata size and sha256.`);
}

function validateExpectedPackageKinds(metadata) {
  const expectations = {
    windows: [
      { label: 'msi', matches: (artifact) => artifact.kind.includes('msi') || artifact.fileName.endsWith('.msi') },
      { label: 'nsis/exe', matches: (artifact) => artifact.kind.includes('nsis') || artifact.fileName.endsWith('.exe') }
    ],
    macos: [{ label: 'dmg', matches: (artifact) => artifact.kind.includes('dmg') || artifact.fileName.endsWith('.dmg') }],
    linux: [
      { label: 'deb', matches: (artifact) => artifact.kind.includes('deb') || artifact.fileName.endsWith('.deb') },
      {
        label: 'AppImage',
        matches: (artifact) => artifact.kind.toLowerCase().includes('appimage') || artifact.fileName.endsWith('.AppImage')
      }
    ]
  };

  for (const expectation of expectations[metadata.platform] ?? []) {
    if (!metadata.artifacts.some(expectation.matches)) {
      fail(`Missing expected ${metadata.platform} package kind: ${expectation.label}.`);
    }
  }
}

function findNamedFile(allFiles, expectedName, metadataPath, fallbackSuffix) {
  const candidates = [];
  if (isNonEmptyString(expectedName)) {
    candidates.push(...allFiles.filter((filePath) => path.basename(filePath) === expectedName));
  }
  if (candidates.length === 0) {
    const metadataBase = path.basename(metadataPath, metadataSuffix);
    candidates.push(
      ...allFiles.filter((filePath) => path.basename(filePath) === `${metadataBase}${fallbackSuffix}`)
    );
  }
  if (candidates.length !== 1) {
    fail(
      `Expected exactly one ${fallbackSuffix} file for ${toDisplayPath(metadataPath)}, found ${candidates.length}.`
    );
  }
  return candidates[0];
}

function createSampleDownloadedArtifacts(root, version) {
  rmSync(root, { recursive: true, force: true });
  mkdirSync(root, { recursive: true });

  const packagePlan = {
    windows: [
      { relativePath: 'msi/900CRM-0.0.0-sample-windows.msi', kind: 'windows-msi-installer' },
      { relativePath: 'nsis/900CRM-0.0.0-sample-windows-setup.exe', kind: 'windows-nsis-installer' }
    ],
    macos: [{ relativePath: 'dmg/900CRM-0.0.0-sample-macos.dmg', kind: 'macos-dmg' }],
    linux: [
      { relativePath: 'deb/900CRM-0.0.0-sample-linux.deb', kind: 'linux-deb' },
      { relativePath: 'appimage/900CRM-0.0.0-sample-linux.AppImage', kind: 'linux-appimage' }
    ]
  };

  for (const platform of defaultPlatforms) {
    const packageRoot = path.join(root, `900crm-${version}-${platform}-packages`);
    const metadataRoot = path.join(root, `900crm-${version}-${platform}-release-metadata`);
    mkdirSync(packageRoot, { recursive: true });
    mkdirSync(metadataRoot, { recursive: true });

    const artifacts = [];
    for (const plannedArtifact of packagePlan[platform]) {
      const relativePath = plannedArtifact.relativePath.replaceAll('0.0.0-sample', version);
      const filePath = path.join(packageRoot, ...relativePath.split('/'));
      mkdirSync(path.dirname(filePath), { recursive: true });
      writeFileSync(
        filePath,
        `900CRM deterministic sample package\nplatform=${platform}\nkind=${plannedArtifact.kind}\nreleaseVersion=${version}\nfile=${relativePath}\n`
      );
      const stats = statSync(filePath);
      artifacts.push({
        fileName: path.posix.basename(relativePath),
        relativePath,
        kind: plannedArtifact.kind,
        sizeBytes: stats.size,
        sha256: sha256Buffer(readFileSync(filePath))
      });
    }

    const filePrefix = `900CRM-${sanitizeFileName(version)}-${platform}`;
    const checksumsFileName = `${filePrefix}${checksumSuffix}`;
    const metadataFileName = `${filePrefix}${metadataSuffix}`;
    const sbomFileName = `${filePrefix}${sbomSuffix}`;
    const metadata = {
      schemaVersion: 1,
      product: '900CRM',
      releaseVersion: version,
      releaseTitle: '900CRM sample release',
      releaseRef: 'local-sample',
      releaseSha: 'local-sample',
      platform,
      generatedAt: '2026-06-27T00:00:00.000Z',
      artifactRoot: `sample/${platform}`,
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
    const sbom = {
      spdxVersion: 'SPDX-2.3',
      dataLicense: 'CC0-1.0',
      SPDXID: 'SPDXRef-DOCUMENT',
      name: `900CRM ${version} ${platform} sample SBOM`,
      creationInfo: {
        created: '2026-06-27T00:00:00.000Z',
        creators: ['Tool: scripts/verify-release-artifacts.mjs --sample']
      },
      packages: [
        {
          name: `source:900CRM-${platform}`,
          SPDXID: `SPDXRef-Package-900CRM-${platform}`,
          versionInfo: version,
          downloadLocation: 'NOASSERTION',
          filesAnalyzed: false,
          licenseConcluded: 'NOASSERTION',
          licenseDeclared: 'Apache-2.0',
          copyrightText: 'NOASSERTION'
        }
      ]
    };

    artifacts.sort((left, right) => left.relativePath.localeCompare(right.relativePath));
    writeFileSync(
      path.join(metadataRoot, checksumsFileName),
      `${artifacts.map((artifact) => `${artifact.sha256}  ${artifact.relativePath}`).join('\n')}\n`
    );
    writeJson(path.join(metadataRoot, metadataFileName), metadata);
    writeJson(path.join(metadataRoot, sbomFileName), sbom);
  }

  console.log(`Created deterministic sample downloaded artifacts under ${toDisplayPath(root)}.`);
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
  console.log(`Verify downloaded 900CRM manual release workflow artifacts.

Usage:
  node scripts/verify-release-artifacts.mjs [options]

Options:
  --artifact-root <dir>       Downloaded workflow artifact root. Defaults to release-download.
  --release-version <value>   Require every metadata file to use this release version.
  --platforms <csv>           Required platforms. Defaults to windows,macos,linux.
  --sample                    Create and verify deterministic synthetic artifacts under dist/.
  --help, -h                  Show this help.

Examples:
  npm run release:artifacts:verify -- --artifact-root release-download --release-version 1.0.0
  npm run release:artifacts:verify:sample
`);
}

function stringArg(name) {
  const value = args.get(name);
  if (value === undefined || value === true) {
    return undefined;
  }
  return String(value);
}

function csvArg(name) {
  const value = stringArg(name);
  if (!value) {
    return undefined;
  }
  return value
    .split(',')
    .map((entry) => entry.trim())
    .filter(Boolean);
}

function resolveFromRepo(value) {
  return path.isAbsolute(value) ? value : path.join(repoRoot, value);
}

function collectFiles(root) {
  const files = [];
  const pending = [root];
  while (pending.length > 0) {
    const current = pending.pop();
    for (const entry of readdirSync(current, { withFileTypes: true }).sort((left, right) => left.name.localeCompare(right.name))) {
      const fullPath = path.join(current, entry.name);
      if (entry.isDirectory()) {
        pending.push(fullPath);
      } else if (entry.isFile()) {
        files.push(fullPath);
      }
    }
  }
  return files.sort((left, right) => toPosixPath(left).localeCompare(toPosixPath(right)));
}

function readJson(filePath) {
  try {
    return JSON.parse(readFileSync(filePath, 'utf8'));
  } catch (error) {
    fail(`Unable to parse JSON ${toDisplayPath(filePath)}: ${error.message}`);
  }
}

function writeJson(filePath, value) {
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function isPackageFile(filePath) {
  return packageExtensions.some((suffix) => filePath.endsWith(suffix));
}

function isNonEmptyString(value) {
  return typeof value === 'string' && value.trim().length > 0;
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

function sha256Buffer(buffer) {
  return createHash('sha256').update(buffer).digest('hex');
}

function sanitizeFileName(value) {
  return String(value).replace(/[^A-Za-z0-9._-]+/g, '-').replace(/^-+|-+$/g, '') || 'unknown';
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
