# Sprint 109 - Local Automation Lite

Date: 2026-07-08
Branch: `codex/local-automation-lite`

## Scope

This sprint adds explicit local next-action suggestions without background
automation, saved rules, schedulers, schema changes, or automatic writes.

The product rule is simple: 900CRM can suggest a next action, but the user must
click through and save the draft before any activity is created.

## Changes

- Added a frontend `localAutomation` utility for:
  - Pipeline stage-move follow-up suggestion drafts;
  - Dashboard overdue/today attention summaries using local-day activity
    bucketing.
- Added a dismissible Pipeline prompt after a successful open-stage deal move
  when the deal has no linked next activity and activity context is available.
- The prompt opens the existing Add Activity modal with deal/contact/account,
  subject, type, due date, and notes prefilled.
- No activity is created until the user saves the modal.
- Added Dashboard "Needs Attention" strip for overdue and due-today follow-ups.
- Dashboard attention uses `listActivities` plus frontend local-day bucketing
  instead of backend overdue stats.
- Added Add Activity modal prefill support for subject, type, due date, and
  notes while preserving existing relationship prefill behavior.
- Added import wizard guidance for contact source/tag columns and tag-link local
  ID requirements.
- Added operational import guidance copy to every supported locale file using
  English safety text, matching existing local-file warning policy.
- Added focused unit/component tests for local automation rules, Dashboard
  attention rendering, Pipeline prompt behavior, import guidance keys, and
  locale guidance availability.
- Updated the product benchmark roadmap and sprint ledger.

## Reviewer Guardrails

The pre-sprint reviewer recommended a frontend-only "explicit local prompts,
never silent automation" scope:

- Suggest a follow-up after a user moves an open deal to another open stage only
  when no linked next activity exists.
- Use the existing Add Activity modal for the draft write path.
- Add Dashboard overdue/today attention using frontend local-day bucketing, not
  backend overdue stats.
- Keep import source/tag work to guidance only because contacts do not have a
  first-class source field and tag links require local IDs.
- Defer auto-tagging, background schedulers, saved rules, recurrence, schema
  changes, MCP, AI, sync server, release, and packaging work.

## Explicit Non-Goals

- No automatic activity creation.
- No background scheduler, saved automation rules, recurrence, or notification
  changes.
- No schema, Rust storage, Tauri command, MCP, AI, sync server, release, or
  packaging changes.
- No import auto-tagging or source-field mutation.
- No tag creation from contact import rows.

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

Future automation depth should start with saved local rules and recurrence only
after the user-facing suggestion model proves trustworthy. Auto-tagging imports
should remain deferred until import rollback semantics explicitly include tag
side effects.
