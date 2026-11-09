# Sprint 057: JSON Import Preflight

Date: 2026-06-25
Branch: `codex/json-import-preflight`

## Scope

- Added read-only duplicate preflight for direct JSON imports of contacts, deals,
  and organizations.
- Reused the existing JSON row parsers and existing duplicate preflight row
  logic so JSON duplicate semantics match CSV duplicate semantics.
- Added thin Tauri commands and typed frontend API wrappers for JSON duplicate
  preflight.
- Updated the Import/Export modal so JSON file selection runs duplicate
  preflight before any import write, shows duplicate warnings when present, and
  requires explicit confirmation before import.

## Duplicate Semantics

- Contacts: case-insensitive exact email match and exact phone match after
  trimming.
- Organizations: case-insensitive exact name and email matches plus exact phone
  match after trimming.
- Deals: case-insensitive exact title match after trimming the imported title.
- JSON row numbering follows existing JSON import behavior: the first array item
  is row 2.

## Non-Goals

- No JSON field mapping.
- No JSON browser preview.
- No duplicate auto-merge.
- No import rollback.
- No automatic backup before import.
- No broader entity, custom field, note, tag, activity, audit, proposed action,
  external client, or permission import/export.
- No MCP, AI, sync-server, encryption, release signing/notarization, or schema
  normalization behavior.

## Verification Checklist

- `npm run lint`
- `npm run check`
- `npm run test`
- `npm run build`
- `npm run test:e2e`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo check --workspace`
- `cargo test --workspace`
- Raw SQL scans in `apps/desktop/src-tauri/src/commands` and
  `crates/crm-core/src/crm_engine`
- `git diff --check main...HEAD`
- `git fsck --full --no-progress`

## Acceptance Notes

- JSON preflight is read-only and covered by focused Rust tests that assert no
  entity, audit, or sync rows are written by preflight.
- Frontend API tests cover the new JSON preflight command names and generic
  entity routing.
- Component coverage verifies that JSON duplicate warnings require continue and
  confirm actions before the import command is called.
