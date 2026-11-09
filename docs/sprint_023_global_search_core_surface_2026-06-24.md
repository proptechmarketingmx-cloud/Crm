# Sprint 023 - Global Search Core Surface

Date: 2026-06-24
Branch: `codex/global-search-core-surface`
Scope: Stable core, Tauri, and frontend API surface for offline global search without changing the visible SearchBar UI.

## Summary

- Added `CrmCore::global_search` as the public core entry point for cross-entity search.
- Expanded global search results to include contacts, organizations, deals, activities, notes, and tags.
- Added typed search result entity classification via `SearchEntityType`.
- Added storage-owned repository queries for organization, note, and tag search, and preserved storage-owned deal/activity search.
- Added a thin desktop `global_search` Tauri command and registered it with the existing command handler pattern.
- Added `apps/desktop/src/lib/api/search.ts` plus focused Vitest invoke-mapping coverage.
- Added focused Rust coverage proving all required entity types are returned and that blank queries and explicit limits are respected.

## Architecture Decisions

- SQL remains contained in `crates/crm-core/src/storage/search.rs`; Tauri commands and `crm_engine` only orchestrate and map typed records.
- Contacts continue to use the existing contacts FTS repository first, with the current contact fallback behavior preserved.
- Organizations, deals, activities, notes, and tags use storage repository text queries for this sprint.
- The visible `SearchBar.svelte` still calls `searchContacts`; no visible UI replacement or component behavior change was made.
- Frontend code calls the new API wrapper rather than invoking Tauri directly from components.
- No MCP server behavior, AI behavior, sync server behavior, destructive migration, import/export behavior, or broad redesign was changed.

## FTS Parity Gap

This sprint does not add FTS5 tables or triggers for organizations, deals, activities, notes, or tags. That keeps the migration non-destructive and tightly scoped, but it means contact search remains the only FTS-backed entity search path. Full FTS5 parity remains deferred.

Superseded by Sprint 052 on 2026-06-24: schema v10 adds FTS5 tables, active-row backfills, and maintenance triggers for organizations, deals, activities, notes, and tags. Global search now uses those storage-owned FTS repositories first with the Sprint 023 text queries retained as fallback.

## Validation

- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `git diff --check main...HEAD`
- [x] Raw SQL scan in `apps/desktop/src-tauri/src/commands`
- [x] Raw SQL scan in `crates/crm-core/src/crm_engine`
- [x] Plain `.ts` rune scan
