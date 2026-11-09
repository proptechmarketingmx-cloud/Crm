# Sprint 040 - Required Docs Baseline

Date: 2026-06-24
Branch: `codex/required-docs-baseline`
Scope: Documentation-only baseline for spec-required public docs missing from
current main.

## Summary

- Added `docs/DATA_MODEL.md` for the current source-derived local data model,
  schema version, migration history, core entities, audit/sync/proposed-action
  readiness, external-client readiness, backups, and compatibility caveats.
- Added `docs/IMPORT_EXPORT.md` for current local CSV import/export behavior,
  mapped contact/organization wizard flow, duplicate preflight behavior, backup
  relationship, and known unimplemented surfaces.
- Added `docs/PRIVACY.md` for offline-first behavior, local storage, no
  telemetry/cloud requirement, backup/export privacy, MCP/AI non-behavior, and
  security caveats without claiming encryption or app-lock features.
- Added narrow README and architecture links to the new required docs.

## Boundaries

- Docs-only sprint.
- No Rust, TypeScript, Svelte, Cargo, npm, package, schema, command-handler,
  UI, MCP runtime, AI, sync-server, listener, token, or dependency behavior was
  changed.
- Wording is intended to be public/open-source safe and source-derived.

## Validation

Validation was run after the docs-only changes:

- [x] `git diff --check main...HEAD`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] Raw SQL boundary scan in Tauri commands and `crm_engine`
- [x] Plain `.ts` rune scan
- [x] Direct Svelte `invoke()` scan
- [x] `git fsck --full --no-progress`
