# Sprint 069 - Tag Import/Export Foundation

Date: 2026-06-25
Branch: `codex/tag-import-export-foundation`

## Scope

Close the next local/offline import/export gap for reusable tag definitions and
local tag links.

Implemented behavior:

- Reusable tag definitions can be exported to CSV and JSON with `name` and
  optional `color`.
- Tag definition imports support CSV and JSON, including mapped source fields in
  the existing Import/Export modal.
- Tag definition import creates missing active tag names through
  `CrmCore::create_tag`, preserving normal tag validation, audit, sync
  changelog, and default-color behavior.
- Existing active tag names are skipped rather than merged or updated because
  tag duplicate rules are not safe without a stronger identity model.
- Local tag links can be exported to CSV and JSON with `entity_type`,
  `entity_id`, and `tag_id`.
- Tag link import validates supported parent types, active parent rows, and
  active reusable tag IDs before writing.
- Tag link import creates missing active local links through
  `CrmCore::apply_tag_to_entity`, preserving normal audit, sync changelog, and
  compatibility mirror behavior. Existing active links are skipped.
- Tag preflight is read-only. It validates required fields and local references,
  and returns zero duplicate warnings for both tag definitions and tag links.
- Completed tag imports return row-level rollback plans for created rows.
  Created tag definitions are soft-deleted only when their post-import snapshot
  still matches and no active links reference them. Created tag links are
  removed only when the exact target `tag_links` row created by the import is
  still active, so rollback skips if the user removed and later reapplied the
  same local tag triple.
- Frontend API command maps and the existing Import/Export modal expose Tag
  Definitions and Tag Links as selectable CSV/JSON import/export entities
  without adding a broad tag-management route or redesign.

## Non-Goals

- No `tag_name` import for tag links; link rows use local `tag_id` only to avoid
  portable identity semantics that the current data model cannot guarantee.
- No audit log, proposed actions, external clients, permissions, settings,
  backup metadata, sync changelog, or custom field definition import/export.
- No remote/cloud/scheduled import/export.
- No MCP runtime, AI behavior, sync-server behavior, release packaging,
  destructive schema migration, encryption, or broad UI redesign.
- No importing or exporting tag IDs through the tag definition format,
  timestamps, deleted rows, device IDs, or backup metadata.

## Verification

Completed locally on the Sprint 069 worktree:

- `npm run lint` - passed after installing locked dependencies with `npm ci`.
- `npm run check` - passed with 0 Svelte errors and 0 warnings.
- `npm run test` - passed: 20 files, 140 tests.
- `npm run build` - passed. Vite emitted the existing browser compatibility
  notices for externalized `node:async_hooks`.
- `npm run test:e2e` - passed: 6 Chromium tests. The web server emitted the
  existing `NO_COLOR`/`FORCE_COLOR` warning.
- `cargo fmt --all -- --check` - passed.
- `cargo clippy --workspace -- -D warnings` - passed.
- `cargo check --workspace` - passed.
- `cargo test --workspace` - passed: 130 `crm-core` tests, 11 Tauri library
  tests, and doc-tests with 17 ignored examples.
- Raw SQL scan in `apps/desktop/src-tauri/src/commands` - no keyword matches.
- Raw SQL scan in `crates/crm-core/src/crm_engine` - no keyword matches.
- `git diff --check` - passed with no whitespace errors.
- `git fsck --full --no-progress` - exited 0 with existing output:
  `dangling commit 10624c1cb973bff9eebabbc81e0fa62c9a568dd9`.
