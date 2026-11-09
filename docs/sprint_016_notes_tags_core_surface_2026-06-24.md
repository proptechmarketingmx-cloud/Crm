# Sprint 016 - Notes/Tags Core Surface Preserve

Date: 2026-06-24
Branch: `codex/notes-tags-core-surface-preserve`
Scope: Generic `crm-core`, Tauri command, and frontend API surface for notes and tags without route/UI work.

## Summary

- Added `CrmCore` note services for create, get, list by entity, update, and soft delete.
- Added `CrmCore` tag services for create, get, list, update, soft delete, apply to entity, remove from entity, and list by entity.
- Added thin Tauri note/tag command modules and registered them in the existing desktop command handler.
- Added frontend API wrappers and focused invoke-mapping tests for notes and tags.
- Added focused Rust tests for note audit/sync, note soft delete, tag audit/sync, tag apply/remove/list, entity validation, and compatibility behavior.

## Architecture Decisions

- Entity references are limited to `contact`, `organization`, `deal`, and `activity`; service methods validate both entity type and row existence before writes or scoped list reads.
- All note/tag mutations enter through `CrmCore`, run inside a transaction, and write both `audit_log` and `sync_changelog`.
- SQL remains inside `crates/crm-core/src/storage/**`; Tauri commands and services do not introduce raw SQL.
- Note storage writes both legacy `content` and target `body`, and reads `body` when present with `content` as the compatibility fallback.
- Tag apply/remove keeps legacy `entity_tags` behavior while mirroring active links into target `tag_links`.
- Tag delete is a soft delete using the existing target compatibility columns, with active links hidden or removed from both link tables.

## Preservation Notes

- No notes or tags route/UI was added.
- No destructive migration was added, and no legacy table or column was removed.
- Existing `entity_tags` readers continue to see links created by the new apply service.
- MCP, AI, sync server, contacts, organizations, backup, email, reminders, reports, custom fields, locales, and pipeline behavior were not intentionally changed.
- Frontend changes are API wrappers/tests only; no unrelated styling was touched.

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
