# Sprint 045 - Public Docs Local Path Scrub

Date: 2026-06-24
Branch: `codex/public-doc-local-path-scrub`
Scope: Documentation-only public-readiness hygiene for historical sprint notes and the sprint ledger.

## Summary

- Scanned public-facing documentation, samples, and workflow files for local-machine absolute paths and user-specific identifiers called out by the release-readiness gate.
- Replaced historical Cargo command examples that embedded machine-specific target directories with repo-relative `cargo` commands.
- Preserved the historical sprint evidence and pass/fail notes while removing machine-local path details.
- Added this sprint note and recorded Sprint 045 in the sprint ledger.

## Changed Files

- `docs/sprint_014_backup_ui_surface_2026-06-23.md`
- `docs/sprint_016_notes_tags_core_surface_2026-06-24.md`
- `docs/sprint_017_notes_tags_contract_hardening_2026-06-24.md`
- `docs/sprint_018_notes_tags_ui_surface_2026-06-24.md`
- `docs/sprint_019_deal_relationships_core_surface_2026-06-24.md`
- `docs/sprint_020_deal_relationships_ui_surface_2026-06-24.md`
- `docs/sprint_021_activity_relationships_core_surface_2026-06-24.md`
- `docs/sprint_045_public_doc_local_path_scrub_2026-06-24.md`
- `docs/sprint_ledger.md`

## Verification

- Diff whitespace check passed.
- Public-document local path and user-identifier scan passed with no remaining matches.
- Cargo target-directory scan passed with no remaining repository-specific target path examples.
- Git status was checked before commit.
- Repository integrity check was run after the docs-only edits.

## Non-Goals

- No app or source behavior under `apps/` or `crates/` changed.
- No MCP, AI, sync server, schema, UI, or CI behavior changed.
- No samples were modified because the scan did not show real private data there.
- No full npm or Cargo verification was run because this sprint only changes public documentation.
