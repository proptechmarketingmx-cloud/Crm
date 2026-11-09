# Sprint 042 - PR CI Foundation

Date: 2026-06-24
Branch: `codex/pr-ci-foundation`
Scope: Add the first honest GitHub Actions PR CI workflow now that the local
verification gates pass.

## Summary

- Added `.github/workflows/ci.yml` for pull requests and pushes to `main`.
- Scoped CI to `ubuntu-latest` only for this sprint; release packaging and
  cross-platform installer builds remain out of scope.
- Installed the Linux system packages needed for Tauri/Rust workspace checks on
  Ubuntu.
- Used `npm ci`, Node.js 20, Rust stable, and the documented local verification
  gates without changing application behavior or dependencies.
- Aligned README and CONTRIBUTING project-structure/CI wording with the initial
  Ubuntu-only PR CI workflow.

## Boundaries

- No product/source behavior changes.
- No schema changes.
- No UI changes.
- No MCP server, runtime, listener, token, AI, sync server, release workflow, or
  installer packaging changes.
- No dependency or lockfile changes.

## Validation

Validation was run after adding the CI workflow and docs:

- [x] `git diff --check main...HEAD`
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] Raw SQL boundary scan in Tauri commands and `crm_engine`
- [x] Plain `.ts` rune scan
- [x] Direct Svelte `invoke()` scan
- [x] `git fsck --full --no-progress`

`git fsck --full --no-progress` exited 0 and reported existing dangling commit
`10624c1cb973bff9eebabbc81e0fa62c9a568dd9`; no object corruption was reported.
