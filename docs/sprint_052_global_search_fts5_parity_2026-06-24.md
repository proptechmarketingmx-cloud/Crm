# Sprint 052 - Global Search FTS5 Parity

Date: 2026-06-24
Branch: `codex/global-search-fts5-parity`
Scope: Non-destructive FTS5 parity for global search across organizations, deals, activities, notes, and tags.

## Summary

- Bumped the local SQLite schema from v9 to v10.
- Added FTS5 virtual tables for organizations, deals, activities, notes, and tags.
- Backfilled the new FTS tables from active, non-deleted source rows during migration.
- Added insert, update, physical-delete, and soft-delete maintenance triggers so future writes stay searchable.
- Routed non-contact global search repositories through FTS first while retaining the Sprint 023 text queries as safe fallback when FTS is unavailable or returns no rows.
- Preserved existing contact FTS behavior.
- Added focused Rust coverage for v10 table creation, FTS-backed global search, trigger maintenance on updates and soft deletes, and missing-table fallback.

## Architecture Decisions

- The migration is additive. It does not remove tables, columns, or source rows.
- FTS SQL remains in `crates/crm-core/src/storage/db.rs` and `crates/crm-core/src/storage/search.rs`.
- `crm_engine::search` remains orchestration and mapping only.
- Search result ordering keeps the stable cross-entity ordering from Sprint 023. Within each non-contact entity, FTS results order by SQLite FTS rank and then the previous stable secondary sort where practical.
- Fallback is intentionally storage-owned: each non-contact repository tries FTS, returns FTS results when present, and otherwise executes the prior LIKE/text query.

## Superseded Gap

Sprint 023 documented that only contacts had an FTS-backed global search path. Sprint 052 closes that gap for organizations, deals, activities, notes, and tags while preserving text fallback for resilience.

## Non-Goals

- No visible UI change.
- No MCP server/runtime/AI behavior.
- No sync server behavior.
- No release packaging.
- No broad ranking redesign beyond FTS-first entity-local ordering.
- No destructive migration or record rewrite.

## Validation

- [x] `npm run lint`
- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run test:e2e`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace -- -D warnings`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] Raw SQL scan in `apps/desktop/src-tauri/src/commands`
- [x] Raw SQL scan in `crates/crm-core/src/crm_engine`
- [x] `git diff --check main...HEAD`
- [x] `git status --short --branch`
- [x] `git fsck --full --no-progress`
