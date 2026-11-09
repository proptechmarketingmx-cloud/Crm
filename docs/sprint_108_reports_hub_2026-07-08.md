# Sprint 108 - Reports Hub

Date: 2026-07-08
Branch: `codex/reports-hub`

## Scope

This sprint moves reporting out of the Dashboard and into a dedicated Reports
route. It stays frontend-only and uses the existing report IPC contracts for
pipeline and activity metrics.

## Changes

- Added a `/reports` hash route and Workspace sidebar navigation item.
- Added a Reports page with:
  - pipeline win rate, open deals, closed won/lost counts, and total deal count;
  - current stage distribution;
  - current-stage funnel ratios using the existing `transition_metrics`
    contract;
  - activity completion, overdue rate, completed/pending/overdue counts;
  - due buckets for overdue, today, next 7 days, later, and no due date;
  - activity type completion breakdown.
- Labeled funnel ratios as current-stage comparisons rather than historical
  conversion rates.
- Removed embedded full report cards from Dashboard so Dashboard remains focused
  on KPIs, first-run guidance, upcoming activity, and quick actions.
- Preserved partial-failure behavior: pipeline and activity reports fail
  independently so one unavailable report does not blank the whole route.
- Added report API invoke tests.
- Added a Svelte component test for loaded report data and partial-error
  rendering.
- Added a regression test proving activity report data can render while the
  pipeline report is still loading.
- Added localized Reports navigation and page copy for every supported locale.
- Extended browser-smoke coverage for direct `/reports` hash loading and
  sidebar navigation.
- Updated the product benchmark and sprint ledger.

## Reviewer Guardrails

The pre-sprint reviewer recommended a narrow frontend-only Reports Hub:

- Use the existing `get_pipeline_conversion_report` and
  `get_activity_funnel_report` APIs.
- Add `/reports` routing and Workspace navigation.
- Move the full report surface out of Dashboard.
- Surface current stage distribution, current-stage funnel ratios, activity due
  buckets, and activity type breakdown.
- Avoid claiming historical conversion because stage history is not stored.
- Defer saved filters, report persistence, exportable snapshots, source/owner
  breakdowns, schema work, Rust/Tauri command changes, MCP, AI, sync server,
  release, and packaging work.

## Explicit Non-Goals

- No schema migration or stage-history model.
- No Rust storage, service, or Tauri command changes.
- No saved report filters, report persistence, or file export.
- No source/owner breakdowns.
- No MCP, AI, sync server, release, or packaging work.

## Verification

Run for sprint acceptance:

```sh
npm run check
npm run test
npm run build
npm run test:e2e
npm run check:release-guardrails
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
git diff --check
git fsck --full --no-progress
```

Focused checks run during implementation:

```sh
npm run check
npm run test
```

## Follow-Up

Future reporting depth should add saved filters, exportable snapshots, stale deal
reporting, owner/source dimensions where supported, and true historical
conversion after the data model records stage transitions.
