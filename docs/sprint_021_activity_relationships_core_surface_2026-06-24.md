# Sprint 021 - Activity Relationships Core Surface

Date: 2026-06-24
Branch: `codex/activity-relationships-core-surface`
Scope: Non-destructive activity relationship foundation while preserving legacy `activities.contact_id` and `activities.deal_id` behavior.

## Summary

- Added schema v8 with a first-class `activity_links` table for contact, organization, and deal relationships.
- Backfilled active legacy activity contact/deal mirrors into active activity links only when the referenced contact/deal is active.
- Added `CrmCore` services for listing, adding, and removing activity links with active-reference validation.
- Kept `activities.contact_id` and `activities.deal_id` as compatibility mirrors for explicit contact/deal links; organization links live only in `activity_links`.
- Added thin Tauri commands and frontend API wrappers for activity links.
- Hardened activity update nullable semantics with explicit reset flags for `due_date`, `contact_id`, and `deal_id`.
- Added focused Rust and frontend invoke-mapping tests for migration, mirror alignment, audit/sync writes, and command payloads.

## Architecture Decisions

- Migration v8 is additive only: it creates `activity_links` and indexes without removing or rewriting legacy activity columns.
- Link rows are soft-deleted via `deleted_at`; active uniqueness is enforced by `(activity_id, entity_type, entity_id)` for active rows.
- `CrmCore` owns reference validation and mirror synchronization so commands remain thin and storage stays composable.
- Contact/deal link create/delete paths update the legacy activity mirrors when the explicit link is the mirrored relationship.
- Organization activity links never write legacy activity columns.
- Activity link create/delete and legacy mirror changes record audit/sync entries in the same transaction.
- SQL remains in `crates/crm-core/src/storage/**`; Tauri commands and `crm_engine` do not contain raw SQL.

## Preservation Notes

- No activity relationship UI was added in this sprint.
- No MCP server behavior, AI behavior, sync server behavior, import wizard work, global search work, or broad redesign was changed.
- Legacy `activities.contact_id` and `activities.deal_id` remain present and continue to serve existing list/filter behavior.
- Existing activity create/list/update command behavior is preserved, with explicit reset flags added for nullable update fields.
- Plain `.ts` files do not use Svelte runes.

## Validation

- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] Raw SQL scan in `apps/desktop/src-tauri/src/commands`
- [x] Raw SQL scan in `crates/crm-core/src/crm_engine`
- [x] Plain `.ts` rune scan
- [x] `git diff --check main...HEAD`
