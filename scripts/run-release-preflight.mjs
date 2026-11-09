#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const npmCommand = process.platform === 'win32' ? 'npm.cmd' : 'npm';

const steps = [
  {
    label: 'Generate sample release notes',
    command: npmCommand,
    args: ['run', 'release:notes:sample']
  },
  {
    label: 'Generate sample release manifest',
    command: npmCommand,
    args: ['run', 'release:manifest:sample']
  },
  {
    label: 'Verify sample downloaded release artifacts',
    command: npmCommand,
    args: ['run', 'release:artifacts:verify:sample']
  },
  {
    label: 'Check public release guardrails',
    command: npmCommand,
    args: ['run', 'check:release-guardrails']
  },
  {
    label: 'Lint frontend tooling',
    command: npmCommand,
    args: ['run', 'lint']
  },
  {
    label: 'Check frontend types',
    command: npmCommand,
    args: ['run', 'check']
  },
  {
    label: 'Test frontend',
    command: npmCommand,
    args: ['run', 'test']
  },
  {
    label: 'Build frontend',
    command: npmCommand,
    args: ['run', 'build']
  },
  {
    label: 'Browser smoke test',
    command: npmCommand,
    args: ['run', 'test:e2e']
  },
  {
    label: 'Check Rust formatting',
    command: 'cargo',
    args: ['fmt', '--all', '--', '--check']
  },
  {
    label: 'Lint Rust',
    command: 'cargo',
    args: ['clippy', '--workspace', '--', '-D', 'warnings']
  },
  {
    label: 'Check Rust workspace',
    command: 'cargo',
    args: ['check', '--workspace']
  },
  {
    label: 'Test Rust workspace',
    command: 'cargo',
    args: ['test', '--workspace']
  }
];

console.log('Running local release preflight source gates.');
console.log('This command does not install system packages, install Playwright browsers, build installers, sign, notarize, tag, publish, or prove platform installability.');

for (const [index, step] of steps.entries()) {
  const commandText = [step.command, ...step.args].join(' ');
  console.log(`\n[${index + 1}/${steps.length}] ${step.label}`);
  console.log(`$ ${commandText}`);

  const result = spawnSync(step.command, step.args, {
    cwd: repoRoot,
    env: process.env,
    stdio: 'inherit'
  });

  if (result.error) {
    console.error(`\nLocal release preflight failed during "${step.label}".`);
    console.error(`Could not start command: ${commandText}`);
    console.error(result.error.message);
    process.exit(1);
  }

  if (result.status !== 0) {
    console.error(`\nLocal release preflight failed during "${step.label}".`);
    if (result.signal) {
      console.error(`Command was terminated by signal ${result.signal}: ${commandText}`);
    } else {
      console.error(`Command exited with status ${result.status}: ${commandText}`);
    }
    process.exit(result.status ?? 1);
  }
}

console.log('\nLocal release preflight completed successfully.');
