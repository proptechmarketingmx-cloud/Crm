# Sprint 017 - Notes/Tags Contract Hardening

Date: 2026-06-24
Branch: `codex/notes-tags-contract-hardening`
Scope: Harden notes/tags service and API contracts without route/UI, MCP, AI, or sync-server behavior changes.

## Summary

- Tightened tag apply/remove idempotency so duplicate operations no longer create extra `audit_log` or `sync_changelog` entries.
- Kept legacy `entity_tags` compatibility while preserving target `tag_links` mirroring.
- Tightened `update_tag` color semantics across API, Tauri command, service, and storage boundaries.
- Added Rust service tests for duplicate tag apply/remove and update color reset behavior.
- Added frontend API tests proving omitted color is omitted while explicit `null` and blank color send `reset_color: true` reset requests.

## Architecture Decisions

- Storage tag-link methods now return whether they changed persisted link state. `CrmCore` records audit/sync entries only when storage reports an actual apply/remove change.
- Duplicate apply remains successful and idempotent when an active link already exists; duplicate remove remains successful and idempotent when no active link exists.
- `update_tag` color uses an explicit command/service contract: omitted `color` with no reset flag means no color change, `reset_color: true` means reset, and a nonblank `color` value means set that color.
- The frontend API translates explicit `null` and blank color values into `reset_color: true` and does not rely on `color: null` at the Tauri IPC boundary.
- The legacy `tags.color` column is `NOT NULL`, so reset/clear behavior is defined as resetting to the default color `#6366f1` rather than storing SQL `NULL`.
- Tag name update behavior and duplicate-name validation remain unchanged.

## Preservation Notes

- No notes/tags route or UI components were added.
- No destructive migration was added, and the legacy `entity_tags` table remains compatible.
- SQL remains inside `crates/crm-core/src/storage/**`; Tauri commands remain thin command-to-service wrappers.
- MCP server behavior, AI behavior, sync server behavior, unrelated cleanup, and styling were not changed.
- Plain `.ts` files still do not use Svelte runes.

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
- [x] `git diff --check origin/main...HEAD`
