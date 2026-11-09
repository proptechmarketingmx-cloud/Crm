# Sprint 098 - Headless macOS DMG Helper

Date: 2026-06-27
Branch: `codex/headless-macos-dmg-helper`
Baseline: `12a6283cbec5e2efecb49ec7ac456f7d545c2171`

## Scope

Add narrow maintainer-local tooling for creating a basic macOS DMG without the
generated Finder/AppleScript customization path that blocked Sprint 096. This
is local headless package evidence only.

This sprint does not add product behavior, UI behavior, schema changes, MCP
behavior, AI behavior, sync server behavior, signing, notarization, publishing,
release tags, GitHub Release automation, or cross-platform package proof.

## Changes

- Added `scripts/build-macos-dmg-headless.mjs`, a macOS-only helper that builds
  the Tauri `.app` bundle with app-only bundling, stages `900CRM.app` plus an
  `Applications` symlink, creates a compressed DMG with `hdiutil create
  -srcfolder`, verifies it with `hdiutil verify`, and prints the output path,
  size, and SHA-256.
- Added `npm run release:macos:dmg:headless` as the root maintainer command.
- Updated release and alpha-readiness docs so the helper is framed as unsigned,
  unnotarized, local package evidence only.
- Added this sprint note and the Sprint 098 ledger row.

## Boundaries

The headless DMG helper intentionally avoids Tauri's generated
`bundle_dmg.sh` Finder/AppleScript customization flow. The output is not a
signed artifact, not a notarized artifact, not a workflow artifact, not Windows
or Linux proof, not GitHub Release proof, not downloaded-artifact verifier
proof, and not alpha release completion.

Generated package outputs remain under ignored `target/` paths and must not be
committed.

## Verification

Commands run during the sprint:

```bash
npm run release:macos:dmg:headless
```

Result: passed. The command:

- built the frontend production bundle;
- built the optimized Tauri release binary;
- bundled `target/release/bundle/macos/900CRM.app`;
- skipped signing through the explicit `--no-sign` app-bundle build;
- created `target/release/bundle/dmg/900CRM_1.0.0_aarch64.headless.dmg`;
- verified the DMG with `hdiutil verify`.

Produced local artifact:

```text
path: target/release/bundle/dmg/900CRM_1.0.0_aarch64.headless.dmg
size: 3858075 bytes (3.68 MiB)
sha256: b4bdbd2ce805032c8822a6ac359d71e0e7ec223617f3afe2b9978c739ab87c91
```

Additional verification:

```bash
npm run check:release-guardrails
npm run check
node --check scripts/build-macos-dmg-headless.mjs
git diff --check
git status --short --branch --ignored
```

The generated DMG and build outputs are ignored and were not committed.
