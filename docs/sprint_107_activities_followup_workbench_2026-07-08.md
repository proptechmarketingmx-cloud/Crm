# Sprint 107 - Activities Follow-up Workbench

Date: 2026-07-08
Branch: `codex/activities-followup-center`

## Scope

This sprint turns the Activities route from a flat activity list into a daily
follow-up workbench. It stays frontend-only and uses existing activity fields,
relationship breadcrumbs, and update/complete commands.

## Changes

- Added a pure `activityWorkbench` utility for local-day due bucket derivation.
- Added due buckets for:
  - `Overdue`
  - `Today`
  - `This Week`
  - `Later`
  - `Unscheduled`
  - `Completed`
- Added a Follow-up Center summary above the Activities filters with open work
  and priority counts.
- Added due-bucket focus chips while preserving existing type, status, and
  custom-field filters.
- Grouped the visible activity list into bucket sections instead of adding a
  full calendar grid.
- Added row-level quick actions:
  - complete/incomplete using the existing completion commands;
  - snooze to tomorrow using the existing activity update command;
  - direct due-date reschedule using the existing activity update command.
- Preserved contact, organization, and deal relationship breadcrumbs in grouped
  activity rows.
- Fixed frontend date-only activity status derivation so a `YYYY-MM-DD` due
  date remains pending for the full local day instead of becoming overdue after
  midnight UTC.
- Extended the browser Tauri shim to cover existing activity update and
  complete/incomplete commands for smoke testing.
- Added unit coverage for bucket derivation, local-day date handling,
  no-due-date handling, snooze date generation, and date-only status mapping.
- Extended browser-smoke coverage for creating an activity, seeing it in the
  Today bucket, snoozing it into This Week, and completing it into Completed.

## Reviewer Guardrails

The pre-sprint reviewer recommended a frontend-only Activities Follow-up
Workbench:

- Add due focus views and counts for overdue, today, this week, later,
  unscheduled, and completed.
- Group the existing list by due bucket rather than adding a full calendar grid.
- Add quick complete, snooze, and reschedule actions using existing APIs.
- Preserve relationship breadcrumbs and existing filters.
- Fix local-day handling for date-only due dates.
- Do not add schema, Rust storage/service, Tauri command, MCP, AI, sync server,
  release, or packaging changes.

## Explicit Non-Goals

- No full calendar grid.
- No recurring reminders, notification scheduling, or external calendar sync.
- No schema migration.
- No Rust storage, service, or Tauri command changes.
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
npm run test -- activityWorkbench activities
npm run test:e2e -- -g "creates an activity"
```

## Follow-Up

Future activity depth can add a true week/day calendar grid, reminders,
recurrence, saved views, or external calendar sync once the local follow-up
workbench is accepted.
