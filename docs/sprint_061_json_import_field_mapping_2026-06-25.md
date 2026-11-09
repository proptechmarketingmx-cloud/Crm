# Sprint 061: JSON Import Field Mapping

Date: 2026-06-25

Branch: `codex/json-import-field-mapping`

## Scope

Closed the import/export documentation gap for JSON field mapping. The sprint
adds mapped JSON import and duplicate preflight support for contacts, deals, and
organizations while preserving the existing JSON preview, automatic pre-import
backup, and import-summary restore behavior.

## Changes

- Added mapped JSON parsers in `crm-core` that reuse the existing import column
  mapping type and validation rules.
- Added `crm-core` mapped JSON import and preflight methods for contacts, deals,
  and organizations.
- Added thin Tauri commands for mapped JSON import and preflight. Confirmed
  mapped JSON imports still run through the existing pre-import backup wrapper.
- Added TypeScript API wrappers for entity-specific and generic mapped JSON
  import/preflight commands.
- Updated the import/export modal so JSON imports show a read-only source-key
  preview, then field mapping, then duplicate preflight and confirmation.
- Auto-mapped matching supported JSON keys and common aliases through the same
  mapping helper used by CSV imports.
- Updated import/export documentation and this sprint ledger entry.

## Verification

- Added core tests for mapped JSON import, mapped JSON duplicate preflight, JSON
  mapping validation, and source-key JSON preview semantics.
- Added API wrapper tests for mapped JSON commands across contacts, deals, and
  organizations.
- Added component tests for JSON preview plus mapping controls, auto-mapped JSON
  preflight/confirmation, nonstandard source-key mapping, duplicate mapping
  validation, duplicate-warning confirmation gating, and summary restore
  preservation.

## Non-Goals

- No duplicate auto-merge.
- No row-level rollback, partial restore, merge-back, or restore behavior
  changes.
- No expansion into custom fields, separate notes, tags, activities, audit logs,
  proposed actions, external clients, permissions, settings, relationships, MCP,
  AI, cloud, sync-server, scheduled import/export, encryption, release signing,
  notarization, or schema normalization.
