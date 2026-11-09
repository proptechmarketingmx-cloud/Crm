# Sprint 062 - Contact/Organization Import Auto-Merge

Date: 2026-06-25
Branch: `codex/contact-org-import-auto-merge`

## Scope

This sprint closes the contact and organization duplicate auto-merge gap for
local desktop imports only. It covers CSV, mapped CSV, JSON, and mapped JSON
imports for contacts and organizations.

Deal duplicate auto-merge remains out of scope.

## Behavior

- The Import/Export wizard shows an opt-in duplicate auto-merge checkbox only
  for contact and organization imports.
- The option is off by default.
- Duplicate preflight still runs before confirmation and remains read-only.
- Confirmation copy changes when auto-merge is enabled.
- Non-duplicate rows still create through the existing import create paths.
- Duplicate contact rows match existing active contacts by email or phone.
- Duplicate organization rows match existing active organizations by name,
  email, or phone.
- A row merges only when all matching rules identify one existing active record.
  Rows with multiple possible matches are skipped with a row-numbered error.
- Merges preserve existing IDs and active rows.
- Merges fill blank existing fields from incoming row values and do not
  overwrite existing non-empty values.
- Merge updates route through existing `crm-core` update services to preserve
  normal sync and audit behavior.
- Import summaries include a `merged` count.
- Automatic pre-import backups are still created before confirmed import writes,
  including auto-merge writes. Backup failure still prevents the import.

## Verification

Focused tests added in this sprint cover:

- contact duplicate auto-merge fill-blank behavior without overwriting;
- organization duplicate auto-merge fill-blank behavior without overwriting;
- non-duplicate row creation during an auto-merge import;
- default-disabled duplicate behavior;
- UI option visibility for contacts/organizations and exclusion for deals;
- enabled option propagation through the frontend import API;
- merged count display in the summary;
- pre-import backup creation before auto-merge writes in the desktop command
  layer.

Full verification commands are run before sprint close and recorded in the
builder final response.

## Non-Goals

- No deal duplicate auto-merge.
- No row-level rollback, partial restore, merge-back, or restore behavior
  changes.
- No custom field, note record, tag, activity, audit-log domain expansion,
  proposed-action, external-client, permission, settings, or relationship
  import/export expansion.
- No remote/cloud/scheduled import/export, sync-server transfer, MCP behavior,
  AI behavior, encryption, release signing/notarization, schema normalization,
  or broad UI redesign.
