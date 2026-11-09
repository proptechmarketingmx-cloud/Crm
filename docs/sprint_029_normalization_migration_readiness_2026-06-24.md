# Sprint 029 - Normalization Migration Readiness

Date: 2026-06-24
Branch: `codex/normalization-migration-readiness-current-main`
Scope: Read-only `crm-core` preflight reporting for future contact and organization normalization.

## Summary

- Added `CrmCore::normalization_migration_preflight()` as the public core service surface.
- Kept all readiness SQL in `crates/crm-core/src/storage/migration_readiness.rs`.
- Reported active legacy organization contacts still stored in `contacts` with `contact_type = 'organization'`.
- Reported active contacts that still have a legacy `org_id` but no normalized `organization_id`.
- Split invalid organization-link debt into separate counts for legacy and normalized references:
  - `contacts_with_invalid_legacy_org_id_links`
  - `contacts_with_invalid_normalized_organization_id_links`
- Reported whether the local backup/restore baseline is available before any destructive migration is considered.
- Added focused Rust tests for clean migrated databases and databases with legacy plus invalid-link debt.

## Safe Next Migration Sequence

1. Create a fresh local backup and validate it successfully.
2. Run `CrmCore::normalization_migration_preflight()`.
3. Resolve invalid legacy `org_id` links and invalid normalized `organization_id` links before data movement.
4. Mirror any remaining valid legacy `org_id` links into `organization_id` without deleting legacy data.
5. Only after the preflight reports no blockers, plan a separate destructive migration for legacy organization contact rows and legacy columns.

## Boundaries

- No destructive migration was added.
- No legacy column or legacy organization-contact removal was added.
- No Tauri command, frontend UI, MCP, AI, or sync server behavior was added.
- No raw SQL was added to Tauri commands or `crm_engine`.
- The preflight is read-only and does not rewrite contacts or organizations.

## Validation

- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `git diff --check main...HEAD`
- [x] Raw SQL boundary scan in Tauri commands and `crm_engine`
- [x] Plain `.ts` rune scan
- [x] Direct Svelte `invoke()` scan
