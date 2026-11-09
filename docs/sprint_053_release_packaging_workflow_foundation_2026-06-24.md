# Sprint 053 - Release Packaging Workflow Foundation

Date: 2026-06-24
Branch: `codex/release-packaging-workflow-foundation`
Scope: Conservative manual release-packaging foundation for release-candidate artifacts and metadata.

## Summary

- Added a manually dispatched GitHub Actions release workflow for Windows, macOS, and Linux Tauri package builds.
- Added a deterministic Node-based release manifest helper that generates SHA-256 checksums, release metadata, and an SPDX-shaped dependency inventory from built artifacts.
- Added local sample-mode manifest generation so maintainers can validate metadata output without existing release bundles.
- Documented the guarded release process in `docs/RELEASE.md` and narrowed README release-status wording.

## Workflow Decisions

- The release workflow is `workflow_dispatch` only and does not run on pushes, pull requests, or tags automatically.
- Repository permissions stay `contents: read` by default. Only the optional publish job has `contents: write`.
- Publishing is off by default and additionally requires `publish_github_release: true`, a non-empty `release_tag` that starts with `v`, and a workflow ref matching `refs/tags/<release_tag>`.
- Package artifacts and release metadata are uploaded as workflow artifacts per platform. Cross-platform aggregation is intentionally left to the manual release process and optional guarded publish job.
- Existing Tauri v2 workspace commands remain the build path; no product runtime behavior changed.
- The workflow passes bundle options through the root npm script as `npm run tauri -- build -- --bundles <bundle-list>` so npm forwards `--bundles` to the Tauri CLI.
- The Tauri build step sets `CI=true`, matching the successful local macOS package probe used for the workflow repair.

## Manifest Decisions

- `scripts/generate-release-manifest.mjs` uses only built-in Node modules.
- The default artifact scan path is `target/release/bundle`, matching the workspace-level Tauri output path.
- The script fails with a helpful message if the artifact directory is missing or contains no supported package suffixes.
- The script ignores Tauri/create-dmg `rw.*` intermediate disk images and records only final package artifacts.
- `--sample` creates a tiny local fixture artifact under the selected output directory for dry-run validation.
- Metadata records app name/version/identifier, release title/version/ref, git SHA, generated timestamp, artifact sizes, and SHA-256 checksums.
- The SBOM output is SPDX-shaped and sourced from `package-lock.json`, `Cargo.lock`, and the source package record.

## Non-Goals

- No signing.
- No macOS notarization.
- No auto-update channel.
- No telemetry or crash reporting.
- No secrets, credentials, MCP, AI, sync server, schema, UI, import/export, or runtime product changes.
- No automatic GitHub release publishing.

## Validation

- [x] `node scripts/generate-release-manifest.mjs --help`
- [x] dry-run/sample manifest generation using temporary files
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `npm run test:e2e`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `git diff --check main...HEAD`
- [x] `git fsck --full --no-progress`

## Repair Validation

- [x] `npm run tauri -- build -- --help` verified the extra npm delimiter reaches the Tauri CLI.
- [x] `CI=true npm run tauri -- build -- --bundles dmg` produced `target/release/bundle/dmg/900CRM_1.0.0_aarch64.dmg`.
- [x] `node scripts/generate-release-manifest.mjs --artifact-dir target/release/bundle --out-dir release-assets/s053-actual-release-manifest --platform macos --release-version 1.0.0 --release-title "900CRM 1.0.0" --release-ref local-probe --release-sha local-probe --generated-at 2026-06-24T00:00:00.000Z` indexed only `dmg/900CRM_1.0.0_aarch64.dmg`.
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `git diff --check main...HEAD`
- [x] `git fsck --full --no-progress`
