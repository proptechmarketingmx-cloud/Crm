# Sprint 065 - Custom Field Import/Export

Date: 2026-06-25
Branch: `codex/custom-field-import-export`

## Scope

Close the documented local import/export gap for contact and deal custom field
values in the existing local CSV/JSON import/export surface.

Implemented scope:

- Contact and deal CSV/JSON exports include active custom field values using
  stable `custom:` columns or JSON keys.
- Contact and deal CSV/JSON imports can set existing custom field values from
  direct `custom:` source fields.
- Contact and deal mapped CSV/JSON imports expose supported custom field targets
  in the existing import wizard and send `custom:` mappings to the existing
  Tauri commands.
- Duplicate auto-merge fills missing or blank contact/deal custom field values
  without overwriting existing non-empty custom values.
- Row-level rollback snapshots include contact/deal custom field values, so
  created-row rollback removes imported custom values and merge rollback restores
  custom values changed by the import.
- Duplicate preflight remains read-only. Custom field values are validated as
  supported targets but do not participate in duplicate detection.
- Automatic pre-import backup behavior remains the first write guard for all
  supported import paths.
- Tauri import/export commands remain thin; custom field behavior lives in
  `crm-core` services, CSV/JSON helpers, and existing custom field storage.

## Convention

The stable import/export target convention is `custom:<field_name>` when an
active field name is unique for the entity type. The prefix keeps custom field
targets separate from flat fields such as `email`, `title`, and `notes`.

Custom field names are user-readable and are not schema-unique. If two active
custom field definitions for the same entity type share a field name, the
target convention becomes `custom:<field_name>#<field_id>` for those duplicate
names. That deterministic fallback keeps exports complete and lets imports map
back to the intended existing custom field definition without guessing.
Literal `%` and `#` characters in the field name portion are escaped as `%25`
and `%23`.

## Non-Goals

- No activity import/export.
- No organization custom fields.
- No custom field definition import/export or schema normalization.
- No remote, cloud, scheduled, MCP, AI, sync-server, or transport behavior.
- No encryption, release signing/notarization, or release packaging behavior.
- No broad Import/Export modal redesign.

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

- Contact/deal custom field import/export is local-only and uses existing
  custom field storage validation/upsert behavior.
- Duplicate preflight intentionally ignores custom values; it continues to use
  email/phone for contacts and title for deals.
- Organization and activity custom field values remain outside the import/export
  surface.
