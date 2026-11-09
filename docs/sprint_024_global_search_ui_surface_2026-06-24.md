# Sprint 024 - Global Search UI Surface

Date: 2026-06-24
Branch: `codex/global-search-ui-surface`
Scope: Frontend-only wiring of the visible global SearchBar to the Sprint 023 global search API.

## Summary

- Replaced the visible SearchBar contact-only lookup with the existing `globalSearch(query, limit)` frontend API wrapper.
- Expanded shared UI search result typing to support contacts, organizations, deals, activities, notes, and tags.
- Added pure frontend helpers for global result mapping, localized type labels with safe fallbacks, and stable badge classes.
- Preserved SearchBar debounce, loading, dropdown, no-results, Escape/blur, selection callback, and clear-on-select behavior.
- Guarded async search responses with a request sequence so older responses cannot overwrite newer typed queries.
- Added focused Vitest coverage for UI result mapping, type labels, and badge class coverage across all six entity types.

## Boundaries

- No Rust, storage, schema, Tauri command, MCP, AI, sync, import/export, or broad redesign changes were made.
- Svelte components continue to use frontend API wrappers rather than direct Tauri `invoke()` calls.
- No Svelte runes were added outside `.svelte.ts` files.

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
- [x] Direct `invoke()` scan in Svelte components/routes
