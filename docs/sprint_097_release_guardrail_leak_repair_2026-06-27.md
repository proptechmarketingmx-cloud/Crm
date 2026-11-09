# Sprint 097 - Release Guardrail Leak Repair

Date: 2026-06-27
Branch: `codex/release-guardrail-leak-repair`
Baseline: `2ba401a8b3e77dc1d038eb8c4deeacb100405c2a`
Merge: `12a6283cbec5e2efecb49ec7ac456f7d545c2171`

## Scope

Repair the public release guardrail failure introduced by Sprint 096
documentation. The post-merge guardrail scan caught machine-specific local path
leaks in the macOS package smoke evidence note.

This sprint was documentation-only. It did not change product behavior, UI,
schema, MCP behavior, AI behavior, sync server behavior, packaging
configuration, signing, notarization, publishing, or release claims.

## Changes

- Replaced a leaked absolute worktree path in the captured `bundle_dmg.sh`
  failure text with the repo-relative generated script path.
- Replaced generated temporary DMG mount paths with generic temporary mount
  wording.
- Preserved the technical meaning: the generated DMG script failed or stalled,
  temporary mounts were detached, and no final Sprint 096 DMG was produced.

## Verification

- Builder and control tower both ran `npm run check:release-guardrails`; the
  scan passed after the repair and scanned 343 tracked text files.
- `git diff --cached --check` passed before commit.
- Reviewer accepted the staged patch and confirmed the local-path leak was
  removed without changing release/package claims.

## CI Status

GitHub Actions did not start verification steps for PR #99 because of the
known external billing/spending-limit blocker. The check-run job had no steps,
and the annotation said the job was not started because recent account payments
failed or the spending limit needed to be increased.
