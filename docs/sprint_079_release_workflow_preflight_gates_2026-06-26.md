# Sprint 079 - Release Workflow Preflight Gates

Date: 2026-06-26
Branch: `codex/release-workflow-preflight-gates`
Scope: Automated preflight verification for manual release packaging.

## Summary

- Added a `preflight` job to the `Manual Release Packaging` workflow.
- The package matrix now depends on `preflight`, so Windows, macOS, and Linux
  package jobs do not start until release guardrails and documented verification
  commands pass on Ubuntu.
- The preflight installs the same Ubuntu system dependencies used by CI before
  running frontend, Playwright, Rust, and Tauri-related checks.
- Release publishing semantics are unchanged: GitHub Release attachment still
  requires explicit `publish_github_release` input and a matching `v`-prefixed
  tag ref.
- Platform manual smoke, signing, and macOS notarization remain manual or not
  implemented.

## Changed Files

- `.github/workflows/release.yml`
- `docs/RELEASE.md`
- `docs/sprint_079_release_workflow_preflight_gates_2026-06-26.md`
- `docs/sprint_ledger.md`

## Verification

- [x] `npm run check:release-guardrails`
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `npm run test:e2e`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] Raw SQL boundary scans in Tauri commands and `crm_engine`
- [x] `git diff --check`
- [x] `git fsck --full --no-progress`

## Non-Goals

- No product runtime behavior changed.
- No UI, schema, import/export, backup, MCP, AI, sync server, signing,
  notarization, release publishing semantics, or package artifact semantics
  changed.
- No npm or Rust dependencies were added.
