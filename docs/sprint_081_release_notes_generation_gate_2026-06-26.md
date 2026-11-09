# Sprint 081 - Release Notes Generation Gate

Date: 2026-06-26
Branch: `codex/release-notes-generation-gate`
Scope: Repo-owned release notes generation with a deterministic sample gate.

## Summary

- Added `scripts/generate-release-notes.mjs` as a built-in Node generator for
  the release notes previously embedded in the release workflow YAML.
- Added `npm run release:notes:sample` to write deterministic sample release
  notes under `dist/release-sample/release-notes.md`.
- Added the sample release notes gate to normal CI and to the manual release
  packaging preflight before package matrix jobs can start.
- Updated the guarded GitHub Release publish job to call the repo-owned script
  instead of writing `release-notes.md` inline.
- Updated release documentation and the sprint ledger to make the release notes
  gate part of the documented release process.

## Changed Files

- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `docs/RELEASE.md`
- `docs/sprint_081_release_notes_generation_gate_2026-06-26.md`
- `docs/sprint_ledger.md`
- `package.json`
- `scripts/generate-release-notes.mjs`

## Verification

- [x] `npm run release:notes:sample`
- [x] `npm run release:manifest:sample`
- [x] `npm run check:release-guardrails`
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `git diff --check`
- [x] `git fsck --full --no-progress`

## Non-Goals

- No product UI, schema, MCP, AI, sync server, backup behavior, or Tauri
  packaging semantics changed.
- No signing, notarization, auto-publishing, release attachment semantics,
  release manifest behavior, update channel, telemetry, crash reporting, or
  credential-handling behavior changed.
- No npm or Rust dependencies were added.
