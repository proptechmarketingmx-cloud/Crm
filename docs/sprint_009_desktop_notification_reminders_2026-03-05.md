# Sprint 009 — Desktop Notification Reminders

Date: 2026-03-05 (UTC)
Branch: `sprint-009-desktop-notification-reminders`

## Scope
- Deliver desktop reminder notifications for upcoming activities.
- Add user-configurable reminder preferences in Settings.
- Keep runtime overhead low for constrained hardware.

## Changes
- Added lightweight activity reminder service:
  - `src/lib/services/activityReminders.ts`
  - Polls upcoming activities once per minute.
  - Sends at-most-once desktop notifications per `activity_id + due_date` while app is running.
  - Uses bounded upcoming list and TTL pruning for low overhead.
- Wired reminder service into app lifecycle:
  - `src/routes/+layout.svelte`
  - Starts after settings load; stops cleanly on teardown.
- Added persisted reminder settings in frontend contracts/stores:
  - `src/lib/api/settings.ts`
  - `src/lib/stores/settings.ts`
  - New settings fields:
    - `notificationsEnabled` (`notifications_enabled`)
    - `reminderLeadMinutes` (`reminder_lead_minutes`)
- Added Settings UI controls:
  - `src/routes/Settings.svelte`
  - Toggle: desktop reminders enabled/disabled.
  - Numeric input: lead time in minutes before due date.
- Added DB migration v3 for existing installs:
  - `src-tauri/src/storage/db.rs`
  - Bumped schema version `2 -> 3`.
  - Backfills default settings keys:
    - `notifications_enabled = true`
    - `reminder_lead_minutes = 30`
- Updated settings storage docs:
  - `src-tauri/src/storage/settings.rs`
- Added i18n labels for notification settings in all supported locales:
  - `src/lib/i18n/en.json`
  - `src/lib/i18n/es.json`
  - `src/lib/i18n/fr.json`
  - `src/lib/i18n/ar.json`
  - `src/lib/i18n/sw.json`
  - `src/lib/i18n/hi.json`

## Validation
- `npm run check` -> passed (0 errors, 0 warnings)
- `cargo check --target-dir /tmp/900crm-sprint-009-target` (in `src-tauri/`) -> passed

## Lightweight Guardrail Checklist
- [x] `Offline-first` remains intact: reminders run from local data only.
- [x] `Local-first` remains intact: settings and activity data remain in SQLite.
- [x] No mandatory cloud/proprietary dependency was introduced.
- [x] Polling is lightweight (60s interval, bounded result set, dedupe + TTL pruning).
- [x] Existing installs are migration-safe via schema v3 backfill defaults.
- [x] Changelog and sprint ledger updated chronologically with UTC date.

## Outcome
- 900CRM now supports desktop reminders for upcoming activities with user-controlled lead time.
- The previous v1.1.0 “desktop notifications/reminders” in-progress item is now implemented.
