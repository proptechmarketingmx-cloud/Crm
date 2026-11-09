# Sprint 080 - Release Manifest Sample Gate

Date: 2026-06-26
Branch: `codex/release-manifest-sample-gate`
Scope: Enforce sample release manifest generation before release packaging.

## Summary

- Added `npm run release:manifest:sample` to the required CI verification job.
- Added `npm run release:manifest:sample` to the manual release packaging
  `preflight` job before package matrix jobs can start.
- Updated release documentation so the verification checklist and manual
  packaging workflow explicitly state that sample manifest generation is
  enforced before packaging.
- Kept the existing manifest script behavior and release packaging semantics
  unchanged.

## Changed Files

- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `docs/RELEASE.md`
- `docs/sprint_080_release_manifest_sample_gate_2026-06-26.md`
- `docs/sprint_ledger.md`

## Verification

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

- No product UI, schema, MCP, AI, sync server, backup, or Tauri packaging
  semantics changed.
- No signing, notarization, publishing, or release attachment behavior changed.
- No manifest script scope or artifact-generation behavior changed.
- No npm or Rust dependencies were added.
