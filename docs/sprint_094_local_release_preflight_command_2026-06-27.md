# Sprint 094 - Local Release Preflight Command

Date: 2026-06-27
Branch: `codex/local-release-preflight-command`
Baseline: `f7e02f9c6eca51c75f8a404163d779049b6cc47c`

## Scope

Add a maintainer-run local preflight command for Phase 6 alpha release
readiness. The command mirrors the manual release packaging workflow preflight
source gates where feasible from an already prepared local checkout.

## Changes

- Added `scripts/run-release-preflight.mjs`, a no-dependency Node runner that
  executes each source gate sequentially with inherited stdio and a clear step
  label.
- Added root `npm run release:preflight:local`.
- Updated release and alpha-readiness docs to describe the command as
  source/preflight evidence only.
- Recorded this sprint in the sprint ledger.

## Local Preflight Gates

The local preflight runs, in order:

1. `npm run release:notes:sample`
2. `npm run release:manifest:sample`
3. `npm run check:release-guardrails`
4. `npm run lint`
5. `npm run check`
6. `npm run test`
7. `npm run build`
8. `npm run test:e2e`
9. `cargo fmt --all -- --check`
10. `cargo clippy --workspace -- -D warnings`
11. `cargo check --workspace`
12. `cargo test --workspace`

## Boundaries

This sprint does not add product behavior, UI, schema, MCP behavior, AI
behavior, sync server behavior, release publishing, tags, installer generation,
signing, notarization, dependency installation, or Playwright browser
installation.

The local preflight is not release proof. It does not replace GitHub Actions
preflight evidence, package matrix artifacts, platform smoke testing, checksums,
release metadata, signing, notarization, or GitHub Release publication evidence.
If local Playwright browser availability or platform prerequisites block the
browser smoke step, that failure remains valid evidence to report.
