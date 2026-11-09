# Sprint 018 - Notes/Tags UI Surface

Date: 2026-06-24
Branch: `codex/notes-tags-ui-surface`
Scope: Reusable generic notes/tags frontend panels integrated for contacts and organizations only.

## Summary

- Added reusable `EntityNotesPanel` and `EntityTagsPanel` Svelte components backed by the existing generic notes/tags API wrappers.
- Added contact detail generic notes and tags panels while preserving the existing legacy `contact.notes` and legacy string tag controls under concise legacy labels.
- Added an Organizations row action that opens an unframed modal surface for managing generic organization notes and tags.
- Added notes/tags UI i18n keys to all registered locale JSON files.

## Architecture Decisions

- The reusable panels take `entityType` and `entityId` props and keep their own load, save, delete, apply, remove, loading, and error state.
- Notes are displayed newest first and use the existing `createNote`, `listNotesForEntity`, `updateNote`, and `deleteNote` wrappers.
- Tags load both the entity's tags and all available tags, use all tags as suggestions, create a tag when the entered name does not already exist, then apply or remove links through the existing wrappers.
- Contact integration keeps generic notes/tags separate from legacy contact fields so no migration or destructive behavior is implied.
- Organization integration uses the existing Organizations route and modal pattern instead of introducing a new detail route.

## Preservation Notes

- Existing legacy contact notes and string tags remain editable and are not migrated or removed.
- No Tauri, Rust, MCP, AI, sync server, or database migration behavior was changed.
- No raw SQL was introduced outside storage modules.
- Plain `.ts` files still do not use Svelte runes.
- No broad frontend testing setup was introduced; the existing frontend API test runner remains unchanged.

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
- [x] Locale JSON parse/key parity check
- [x] `git diff --check origin/main...HEAD`
