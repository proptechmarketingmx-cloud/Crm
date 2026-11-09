# Sprint 012 — Optional Email Integration (IMAP/SMTP)

Date: 2026-03-05 (UTC)
Branch: `sprint-012-email-integration-optional`

## Scope
- Add optional email integration settings for IMAP/SMTP endpoints.
- Add lightweight server connection checks without introducing heavy always-on email sync.
- Add contact-level compose action using local `mailto:` integration.

## Changes
- Added backend email integration command module:
  - `src-tauri/src/commands/email_commands.rs`
  - New command: `test_email_server_connection`
  - Performs lightweight DNS + TCP reachability checks for SMTP/IMAP endpoints.
  - Includes best-effort plaintext protocol probing when applicable.
- Registered new command in Tauri:
  - `src-tauri/src/commands/mod.rs`
  - `src-tauri/src/lib.rs`
- Added settings schema defaults for email integration:
  - `src-tauri/src/storage/db.rs`
  - Bumped DB schema version `3 -> 4`
  - New settings keys:
    - `email_integration_enabled`
    - `smtp_host`, `smtp_port`, `smtp_username`, `smtp_password`, `smtp_from`
    - `imap_host`, `imap_port`, `imap_username`, `imap_password`
- Extended settings contracts/store:
  - `src/lib/api/settings.ts`
  - `src/lib/stores/settings.ts`
- Added frontend email API utilities:
  - `src/lib/api/email.ts`
  - Tauri wrapper for connection tests + `mailto:` compose helper.
- Added optional email settings UI:
  - `src/routes/Settings.svelte`
  - IMAP/SMTP host/port/credential fields with persisted settings.
  - Per-protocol connection test actions and status messages.
- Added contact-level email compose action:
  - `src/routes/ContactDetail.svelte`
  - Opens local mail client for the contact email via `mailto:`.
- Added i18n coverage for new settings labels/messages across all supported locales:
  - `src/lib/i18n/en.json`
  - `src/lib/i18n/es.json`
  - `src/lib/i18n/fr.json`
  - `src/lib/i18n/ar.json`
  - `src/lib/i18n/sw.json`
  - `src/lib/i18n/hi.json`
  - `src/lib/i18n/pt.json`
  - `src/lib/i18n/vi.json`

## Validation
- `npm run check` -> passed (0 errors, 0 warnings)
- `cargo check --target-dir /tmp/900crm-sprint-012-target` -> passed
- Locale key parity vs English base:
  - `fr/es/ar/sw/hi/pt/vi`: missing=0, extra=0

## Lightweight Guardrail Checklist
- [x] `Offline-first` remains intact: core CRM features remain fully local/offline.
- [x] `Local-first` remains intact: email settings are stored locally in SQLite.
- [x] No mandatory proprietary/cloud dependency introduced.
- [x] Integration remains optional and user-controlled (`email_integration_enabled`).
- [x] No heavy background polling/sync jobs were added.
- [x] Changelog and sprint ledger updated chronologically with UTC date.
- [x] Work completed on a dedicated sprint branch.

## Outcome
- 900CRM now has optional IMAP/SMTP configuration and lightweight connection testing.
- Users can launch local email composition from contact detail without cloud lock-in.
- The roadmap item for optional email integration is now implemented.
