# Sprint 095 - Release Artifact Verifier

Date: 2026-06-27
Branch: `codex/release-artifact-verifier`
Baseline: `34667d6ec64f0acba60b621cef24661f44b063bd`

## Scope

Add a deterministic local verifier for artifacts downloaded from the manual
release packaging workflow. This sprint is limited to release artifact
integrity checking and release-readiness documentation. It does not generate
installers, sign, notarize, tag, upload artifacts, publish releases, or change
product runtime behavior.

## Changes

- Added `scripts/verify-release-artifacts.mjs`, a no-dependency Node verifier
  for downloaded workflow artifact directories.
- Added `npm run release:artifacts:verify` for real downloaded artifacts,
  defaulting to `release-download/`.
- Added `npm run release:artifacts:verify:sample`, which creates a deterministic
  synthetic downloaded-artifact tree under ignored
  `dist/release-artifact-verifier-sample/release-download/` and verifies it.
- Added the sample verifier to `npm run release:preflight:local` after sample
  release manifest generation.
- Updated release and alpha-readiness docs so maintainers can distinguish
  downloaded-artifact integrity evidence from installer generation,
  installability, signing, notarization, tagging, upload, and publishing proof.

## Verification Behavior

The verifier requires `windows`, `macos`, and `linux` metadata by default. It
accepts `--artifact-root <dir>`, `--release-version <value>`, and
`--platforms <csv>`.

For each platform, it verifies:

- release metadata, SHA-256 checksum, and SPDX SBOM files exist;
- metadata `schemaVersion` is `1`;
- metadata `platform` is present and unique;
- metadata `releaseVersion` matches `--release-version` when provided;
- metadata artifacts are non-empty and include `fileName`, `relativePath`,
  `kind`, `sizeBytes`, and a 64-character SHA-256 digest;
- package files can be found under the artifact root and match metadata size
  and SHA-256;
- checksum entries match metadata artifacts exactly;
- SBOM JSON parses, uses `SPDX-2.3`, and has non-empty `packages`;
- workflow package kinds are present: Windows `.msi` and NSIS/`.exe`, macOS
  `.dmg`, and Linux `.deb` plus `.AppImage`.

## Non-Goals Preserved

- No installer generation.
- No release publishing or GitHub Release creation.
- No tag creation.
- No signing or macOS notarization.
- No product behavior, UI, schema, MCP, AI, sync server, or release workflow
  packaging semantics changes.

## Verification

Commands run during the sprint:

```bash
npm run release:artifacts:verify:sample
```

Result: passed after rerunning with worktree write permissions for the ignored
`dist/` sample directory.
