# Sprint 063 - Deal Import Auto-Merge

Date: 2026-06-25
Branch: `codex/deal-import-auto-merge`

## Scope

This sprint closes the documented import/export gap for deal duplicate
auto-merge during local desktop imports.

Implemented scope:

- Deal duplicate auto-merge is available for CSV, mapped CSV, JSON, and mapped
  JSON imports.
- The option is explicit and remains off by default.
- Deal matching uses the same active-deal duplicate rule as deal preflight:
  exact title after trimming, case-insensitively.
- Non-duplicate deal rows continue to create normally.
- Rows that match multiple active deals are skipped with row-numbered errors.
- `ImportResult.merged` counts successful deal merges.
- Deal import Tauri commands still run through the automatic pre-import backup
  guard before create or merge writes.
- The Import/Export UI now exposes the auto-merge checkbox for contacts, deals,
  and organizations with entity-neutral confirmation and result copy.

## Merge Policy

Deal merge behavior is conservative:

- existing titles are preserved;
- `expected_close` and `notes` fill only when the existing field is blank and
  the incoming field is nonblank;
- `value` fills only when the existing value is `0.0` and the imported value is
  nonzero;
- `currency` and `stage` are not overwritten, including default `USD` and
  `Lead` values.

The value rule is documented because the flat import row stores deal value as
text and cannot distinguish a blank value from an explicit zero once it reaches
the normalized import row.

## Verification

Focused tests added or updated in this sprint cover:

- direct CSV deal auto-merge success with non-duplicate row creation;
- mapped CSV deal auto-merge using title matching;
- direct JSON deal auto-merge preserving existing nonblank/nondefault values;
- mapped JSON ambiguous title matches skipped with row-numbered errors;
- default-disabled duplicate behavior;
- deal auto-merge backup ordering in the desktop command helper;
- frontend API option propagation for deal import commands;
- Import/Export UI visibility and option propagation for deals.

Full verification commands are run before sprint close and recorded in the
builder final response.

## Non-Goals

- No contact or organization behavior rewrite beyond shared option/copy
  plumbing.
- No import rollback, partial restore, merge-back, or completed-import undo
  engine.
- No remote, cloud, or scheduled import/export.
- No custom field, notes-as-records, tags, activities, audit-log domain
  expansion, proposed actions, external clients, permissions, settings, or
  relationship import/export.
- No MCP, AI, sync-server behavior, encryption, release signing/notarization,
  release packaging, or schema normalization.
