#!/usr/bin/env node
import { execFileSync } from 'node:child_process';
import { readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const trackedFiles = execFileSync('git', ['ls-files', '-z'], {
  cwd: repoRoot,
  encoding: 'utf8'
})
  .split('\0')
  .filter(Boolean)
  .sort();

// The verifier contains the literal patterns it searches for, so it skips itself.
const skippedPathPrefixes = [
  '.git/',
  '.svelte-kit/',
  'scripts/verify-public-release-guardrails.mjs',
  'dist/',
  'node_modules/',
  'target/',
  'apps/desktop/.svelte-kit/',
  'apps/desktop/build/',
  'apps/desktop/dist/',
  'apps/desktop/node_modules/',
  'apps/desktop/src-tauri/target/'
];

const skippedBinaryExtensions = new Set([
  '.avif',
  '.bmp',
  '.dmg',
  '.exe',
  '.gif',
  '.ico',
  '.icns',
  '.jpeg',
  '.jpg',
  '.mov',
  '.mp4',
  '.msi',
  '.otf',
  '.pdf',
  '.png',
  '.rpm',
  '.ttf',
  '.webp',
  '.woff',
  '.woff2',
  '.zip'
]);

const findings = [];
let scannedFiles = 0;

for (const relativePath of trackedFiles) {
  if (shouldSkipPath(relativePath)) {
    continue;
  }

  const absolutePath = path.join(repoRoot, relativePath);
  const stats = statSync(absolutePath);
  if (!stats.isFile()) {
    continue;
  }

  const content = readFileSync(absolutePath);
  if (isBinaryContent(content)) {
    continue;
  }

  const text = content.toString('utf8');
  scannedFiles += 1;
  scanText(relativePath, text);
}

if (findings.length > 0) {
  console.error(`Public release guardrail scan failed with ${findings.length} finding(s):`);
  for (const finding of findings) {
    console.error(`- ${finding.file}:${finding.line} [${finding.category}] ${finding.message}`);
    console.error(`  ${finding.preview}`);
  }
  process.exit(1);
}

console.log(`Public release guardrail scan passed: scanned ${scannedFiles} tracked text file(s).`);

function shouldSkipPath(relativePath) {
  if (skippedPathPrefixes.some((prefix) => relativePath.startsWith(prefix))) {
    return true;
  }
  return skippedBinaryExtensions.has(path.extname(relativePath).toLowerCase());
}

function isBinaryContent(content) {
  const sample = content.subarray(0, Math.min(content.length, 8192));
  return sample.includes(0);
}

function scanText(relativePath, text) {
  const lineStarts = buildLineStarts(text);

  scanRegex(relativePath, text, lineStarts, 'local-path', /\/Users\/[A-Za-z0-9._-]+(?:\/[^\s`"'<>)]*)?/g, 'User-specific macOS home path');
  scanRegex(relativePath, text, lineStarts, 'local-path', /\/Volumes\/[A-Za-z0-9._ -]+(?:\/[^\s`"'<>)]*)?/g, 'Machine-specific mounted volume path');
  scanRegex(relativePath, text, lineStarts, 'local-path', /\/private\/tmp\/[A-Za-z0-9._/-]+/g, 'Machine-specific private temporary path');
  scanRegex(relativePath, text, lineStarts, 'local-path', /\b[A-Za-z]:\\Users\\[^\s`"'<>)]*/g, 'User-specific Windows home path');
  scanRegex(relativePath, text, lineStarts, 'local-path', /\b900CRM-worktrees\b/g, 'Local worktree path leaked into tracked files');
  scanRegex(relativePath, text, lineStarts, 'local-path', /\bDesktop\/Codex\b/g, 'Local Desktop Codex path leaked into tracked files');
  scanRegex(relativePath, text, lineStarts, 'local-path', /\bCodexLocal\b/g, 'Local CodexLocal path leaked into tracked files');
  scanRegex(relativePath, text, lineStarts, 'local-user', /\bsamir(?:usani|usani|usani)?\b|\bsamrusani\b/gi, 'User-specific identifier leaked into tracked files');

  scanRegex(relativePath, text, lineStarts, 'private-host', /\bhttps?:\/\/(?:10(?:\.\d{1,3}){3}|192\.168(?:\.\d{1,3}){2}|172\.(?:1[6-9]|2\d|3[01])(?:\.\d{1,3}){2})(?::\d+)?[^\s`"'<>)]*/g, 'RFC1918 private network URL');
  scanRegex(relativePath, text, lineStarts, 'private-host', /\bhttps?:\/\/[^\s`"'<>/]+\.(?:internal|corp)(?::\d+)?[^\s`"'<>)]*/gi, 'Internal hostname URL');

  scanRegex(relativePath, text, lineStarts, 'secret', /-----BEGIN [A-Z ]*PRIVATE KEY-----/g, 'Private key block');
  scanRegex(relativePath, text, lineStarts, 'secret', /\bghp_[A-Za-z0-9_]{20,}\b/g, 'GitHub personal access token literal');
  scanRegex(relativePath, text, lineStarts, 'secret', /\bgithub_pat_[A-Za-z0-9_]{20,}\b/g, 'GitHub fine-grained token literal');
  scanRegex(relativePath, text, lineStarts, 'secret', /\bsk-[A-Za-z0-9]{32,}\b/g, 'Model/API secret key literal');
  scanRegex(relativePath, text, lineStarts, 'secret', /\bxox[baprs]-[A-Za-z0-9-]{20,}\b/g, 'Slack token literal');
  scanRegex(relativePath, text, lineStarts, 'secret', /\bAKIA[0-9A-Z]{16}\b/g, 'AWS access key ID literal');
  scanRegex(relativePath, text, lineStarts, 'secret', /\bAWS_SECRET_ACCESS_KEY\s*[:=]\s*['"]?[A-Za-z0-9/+=]{20,}['"]?/g, 'AWS secret access key assignment');

  scanSecretAssignments(relativePath, text, lineStarts);
}

function scanRegex(relativePath, text, lineStarts, category, regex, message) {
  for (const match of text.matchAll(regex)) {
    addFinding(relativePath, lineStarts, match.index ?? 0, category, message, match[0]);
  }
}

function scanSecretAssignments(relativePath, text, lineStarts) {
  const assignmentPattern = /\b(api[_-]?key|access[_-]?token|auth[_-]?token|client[_-]?secret|secret[_-]?key|password|token)\b\s*[:=]\s*(['"])([^'"\n]*)\2/gi;
  for (const match of text.matchAll(assignmentPattern)) {
    const key = match[1];
    const value = match[3].trim();
    if (isPlaceholderSecretValue(value)) {
      continue;
    }
    addFinding(
      relativePath,
      lineStarts,
      match.index ?? 0,
      'secret',
      `Possible non-placeholder ${key} assignment`,
      `${key}: ${redact(value)}`
    );
  }
}

function isPlaceholderSecretValue(value) {
  if (value.length === 0) {
    return true;
  }

  const normalized = value.toLowerCase();
  if (value.length < 16) {
    return true;
  }

  return (
    normalized.startsWith('rollback-token') ||
    normalized.startsWith('test-') ||
    normalized.startsWith('example') ||
    normalized.startsWith('dummy') ||
    normalized.startsWith('fake') ||
    normalized.startsWith('placeholder') ||
    normalized.startsWith('redacted') ||
    normalized.startsWith('changeme') ||
    normalized.startsWith('your-') ||
    normalized.startsWith('your_') ||
    /^\*+$/.test(value) ||
    /^x+$/i.test(value) ||
    /^<[^>]+>$/.test(value)
  );
}

function buildLineStarts(text) {
  const lineStarts = [0];
  for (let index = 0; index < text.length; index += 1) {
    if (text[index] === '\n') {
      lineStarts.push(index + 1);
    }
  }
  return lineStarts;
}

function lineForIndex(lineStarts, index) {
  let low = 0;
  let high = lineStarts.length - 1;
  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    if (lineStarts[mid] <= index) {
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }
  return high + 1;
}

function addFinding(relativePath, lineStarts, index, category, message, matchedText) {
  findings.push({
    file: relativePath,
    line: lineForIndex(lineStarts, index),
    category,
    message,
    preview: redact(String(matchedText).replace(/\s+/g, ' ').slice(0, 180))
  });
}

function redact(value) {
  if (value.length <= 12) {
    return value;
  }
  return `${value.slice(0, 6)}...${value.slice(-4)}`;
}
