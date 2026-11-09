# Sprint 110 - Alpha Packaging Smoke Baseline

Date: 2026-07-08
Branch: `codex/alpha-packaging-smoke-baseline`

## Purpose

Create a repeatable local macOS package smoke baseline without claiming that
900CRM has a complete alpha release. The sprint exists to turn the existing
headless DMG helper, release metadata generator, and artifact verifier into one
maintainer command that can produce local evidence while GitHub Actions package
jobs remain externally blocked.

## What Changed

- Added `npm run release:macos:smoke:local`.
- Added `scripts/run-local-macos-package-smoke.mjs`.
- The helper:
  - builds or reuses the local Tauri `.app`;
  - creates the local headless DMG through the existing DMG helper;
  - verifies the DMG with `hdiutil verify`;
  - copies the DMG into ignored `dist/local-macos-package-smoke/` output;
  - generates macOS-only checksums, release metadata, and SBOM output;
  - verifies the local macOS artifact tree with the existing artifact verifier;
  - mounts the DMG and confirms `900CRM.app` plus the `Applications` symlink are
    present.
- Updated release and alpha-readiness docs to describe this command as local
  unsigned and unnotarized maintainer evidence only.

## Non-Goals

This sprint did not add public installers, GitHub Release publishing, release
tags, signing, macOS notarization, auto-update, telemetry, crash reporting,
Windows/Linux package proof, product behavior, schema changes, MCP behavior, AI
behavior, or sync-server behavior.

## Evidence Boundary

The new command proves only that a maintainer on macOS can create and inspect a
local unsigned, unnotarized headless DMG and validate macOS-only local metadata.
It does not prove Actions-backed artifacts, downloaded workflow artifact
integrity, Intel plus Apple Silicon coverage, Windows/Linux installers, a trusted
end-user installer, or alpha release completion.

## Verification

`npm run release:macos:smoke:local` passed with this local macOS evidence:

- DMG path: `target/release/bundle/dmg/900CRM_1.0.0_aarch64.headless.dmg`
- DMG size: `3890886` bytes (`3.71` MiB)
- DMG SHA-256:
  `5d185806ba5709e88cb78a14bdeb484cedf74e6714166106235978bf44d54f39`
- Local artifact verification root:
  `dist/local-macos-package-smoke/release-download`
- Copied package artifact:
  `dist/local-macos-package-smoke/release-download/900crm-1.0.0-macos-packages/dmg/900CRM_1.0.0_aarch64.headless.dmg`
- Metadata:
  `dist/local-macos-package-smoke/release-download/900crm-1.0.0-macos-release-metadata`
- Artifact verifier command:
  `npm run release:artifacts:verify -- --artifact-root dist/local-macos-package-smoke/release-download --release-version 1.0.0 --platforms macos`
- Mounted-DMG layout smoke passed: `900CRM.app` and the `Applications` symlink
  were present.

The remaining acceptance gates are:

```bash
npm run release:preflight:local
npm run check:release-guardrails
git diff --check
git fsck --full --no-progress
```

Standard source gates should also remain green:

```bash
npm run check
npm run test
npm run build
npm run test:e2e
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

Full command outputs for this sprint are recorded in the pull request and
closeout message rather than committed binary artifact files. Generated package
outputs remain ignored under `target/` and `dist/`.
