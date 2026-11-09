# Sprint 027 - Import Field Mapping Core Surface

Date: 2026-06-24
Branch: `codex/import-field-mapping-core-surface`
Scope: Core/Tauri/frontend API foundation for mapped CSV import and preflight for contacts and organizations.

## Completed

- Added `ImportColumnMapping` for frontend-provided source-header to target-field mappings, with `null` skip semantics.
- Added mapped CSV parsers for contacts and organizations that preserve source row numbers and skip blank required `first_name` / `name` rows consistently with existing imports.
- Added defensive mapping validation for unknown target fields, stale source headers, and duplicate non-null target assignments.
- Added `CrmCore` mapped preflight and import methods for contacts and organizations.
- Kept mapped imports routed through the existing `create_contact(...)` and `create_organization(...)` service methods so validation, audit, and sync semantics remain centralized.
- Kept mapped preflight read-only and reused the existing duplicate-warning report shape.
- Added thin Tauri commands and frontend API wrappers for the four mapped operations.
- Added focused Rust and frontend API tests for mapped import/preflight behavior and payload shape.

## Follow-Up Work

- Field mapping UI remains a follow-up sprint.
- Duplicate warning rendering remains a follow-up sprint.
- Confirm-import wizard flow remains a follow-up sprint.

## Boundaries

- No Import Wizard UI was added.
- Existing unmapped import/export command names and frontend wrappers remain available.
- No destructive migrations, legacy column removals, MCP behavior, AI behavior, or sync server behavior were changed.
- No SQL or business logic was added to Tauri commands or frontend code.
