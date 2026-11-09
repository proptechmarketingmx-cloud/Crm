# Sprint 078 - Public Release Guardrail Scan

Date: 2026-06-26
Branch: `codex/public-release-guardrail-scan`
Scope: High-confidence public-release hygiene automation for tracked repository files.

## Summary

- Added `scripts/verify-public-release-guardrails.mjs`, a dependency-free Node script that scans tracked text files from `git ls-files`.
- Wired `npm run check:release-guardrails` into package scripts and CI.
- The scan catches high-confidence local-machine path leaks, private/internal host URLs, private key/token literals, and non-placeholder credential assignments while avoiding ordinary `token` fields, empty passwords, placeholder values, and generic `/tmp/...` test paths.
- Scrubbed the remaining machine-specific private temporary release-manifest output path from Sprint 053 documentation while preserving the verification evidence.
- Updated release and guardrail docs so maintainers know the automated scan complements, but does not replace, manual real-customer-data review.

## Changed Files

- `.github/pull_request_template.md`
- `.github/workflows/ci.yml`
- `docs/OPEN_SOURCE_GUARDRAIL_CHECKLIST.md`
- `docs/RELEASE.md`
- `docs/sprint_053_release_packaging_workflow_foundation_2026-06-24.md`
- `docs/sprint_078_public_release_guardrail_scan_2026-06-26.md`
- `docs/sprint_ledger.md`
- `package.json`
- `scripts/verify-public-release-guardrails.mjs`

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
- [x] `git fsck --full --no-progress`

## Non-Goals

- No product runtime behavior changed.
- No UI, schema, import/export, backup, MCP, AI, sync server, release publishing, signing, notarization, or package-building behavior changed.
- No npm dependencies were added.
- The scan is intentionally high-confidence; manual release review still owns real-customer-data and contextual sensitivity checks.
