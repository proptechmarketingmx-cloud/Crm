# Sprint 044 - Release Readiness Truth Baseline

Date: 2026-06-24
Branch: `codex/release-readiness-truth-baseline`
Scope: Documentation and sample-data baseline for honest Phase 6 release
readiness without implementing release packaging automation.

## Summary

- Added `docs/RELEASE.md` with the current verification-only CI status, manual
  release checklist, future artifact requirements, and explicit not-yet-built
  release packaging boundaries.
- Added small synthetic CSV import samples for contacts and organizations under
  `samples/`.
- Updated README release/download/roadmap wording so public docs no longer claim
  published installers or release artifacts.
- Updated CHANGELOG release wording so planned 1.0.0 platform artifacts are
  described as future targets, not already-published installers.
- Linked CONTRIBUTING to the release readiness checklist while preserving the
  existing verification-only CI boundary.

## Boundaries

- No product/source behavior changes.
- No UI changes.
- No schema changes.
- No release packaging workflow, signing, notarization, checksum, SBOM, or
  GitHub release automation.
- No MCP runtime, AI behavior, sync server behavior, or import/export behavior
  changes.

## Sample Data

- `samples/contacts.csv` uses the documented contact import fields:
  `first_name`, `last_name`, `org_name`, `email`, `phone`, `address`, `city`,
  `country`, and `notes`.
- `samples/organizations.csv` uses the documented organization import fields:
  `name`, `email`, `phone`, `website`, `address_line1`, `address_line2`,
  `city`, `region`, `country`, `postal_code`, and `description`.
- Deal samples remain deferred because the current deal import path is legacy
  and does not have the same mapped wizard/preflight surface as contacts and
  organizations.

## Validation

Local gates run for closeout:

- [x] `git diff --check`
- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `npm run test:e2e`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
