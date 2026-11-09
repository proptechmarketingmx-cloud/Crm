# Sprint 006 — Reports Metrics Backend

Date: 2026-03-05 (UTC)
Branch: `sprint-006-reports-metrics-backend`

## Scope
- Implement backend reporting metrics for pipeline conversion and activity funnels.
- Expose metrics over typed Tauri IPC commands for future reporting UI.
- Add migration/index hardening for low-resource analytics query paths.

## Changes
- Added reporting storage module:
  - `src-tauri/src/storage/reporting.rs`
  - Implements:
    - `get_pipeline_conversion_report(...)`
    - `get_activity_funnel_report(...)`
  - Returns structured stage, transition, type, and due-bucket aggregates.
- Added reporting command module:
  - `src-tauri/src/commands/report_commands.rs`
  - New IPC endpoints:
    - `get_pipeline_conversion_report`
    - `get_activity_funnel_report`
- Wired modules and command registration:
  - `src-tauri/src/storage/mod.rs`
  - `src-tauri/src/commands/mod.rs`
  - `src-tauri/src/lib.rs`
- Added reporting-focused schema migration:
  - Bumped DB schema version from `1` to `2`.
  - Added `migrate_v2()` in `src-tauri/src/storage/db.rs` with indexes:
    - `idx_deals_created_at`
    - `idx_deals_deleted_stage`
    - `idx_activities_created_at`
    - `idx_activities_type`
    - `idx_activities_completed_due`
- Added frontend API wrappers for reporting contracts:
  - `src/lib/api/reports.ts`

## Validation
- `npm run check` -> passed (0 errors, 0 warnings).
- `cargo check --target-dir /tmp/900crm-sprint-006-target` (in `src-tauri/`) -> passed.

## Outcome
- Reporting backend contracts are now production-ready for Sprint 007 dashboard/report UI integration.
- Query/index paths are hardened for lower-resource hardware and larger local datasets.
