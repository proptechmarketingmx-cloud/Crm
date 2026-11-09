# Sprint 046 - E2E Workflow Smoke Coverage

Date: 2026-06-24
Branch: `codex/e2e-workflow-smoke-coverage`
Scope: Frontend smoke-test/docs expansion, plus one narrow modal lifecycle testability fix required for existing UI dialogs to render in the Vite browser harness.

## Summary

- Expanded browser-smoke coverage beyond static route rendering into realistic Vite-rendered UI workflows.
- Extracted the test-only Tauri IPC shim into a reusable E2E helper.
- Added in-memory mock state for contacts, organizations, deals, activities, activity links, and global search results.
- Kept unknown Tauri invokes fail-loud so new UI command usage is explicit in the smoke harness.
- Replaced `Modal.svelte`'s direct `onDestroy` import with `onMount` cleanup after the Vite browser-smoke path resolved `onDestroy` through Svelte's server entry and blocked existing modal rendering.

## Changed Files

- `apps/desktop/src/lib/components/Modal.svelte`
- `e2e/app-shell.smoke.spec.ts`
- `e2e/tauri-shim.ts`
- `e2e/workflows.smoke.spec.ts`
- `docs/sprint_046_e2e_workflow_smoke_coverage_2026-06-24.md`
- `docs/sprint_ledger.md`

## Workflow Coverage

- Create a contact through the Contacts page add-contact modal, verify it appears in the Contacts table, then verify global search shows the created contact.
- Create an organization through the Organizations page modal and verify it appears in Organizations.
- Create a deal through the Pipeline add-deal modal and verify it appears on the Pipeline board.
- Create an activity through the Activities quick-add UI and verify it appears in Activities.
- Keep the existing app-shell/dashboard/hash-route smoke checks passing through the shared shim.

## Verification

- `npm run test:e2e` - passed, 6 Playwright Chromium tests.
- `npm run check` - passed, 0 Svelte errors and 0 warnings.
- `npm run test` - passed, 16 test files and 78 tests.
- `git diff --check main...HEAD` - passed with no whitespace errors.
- `git status --short --branch` - checked before commit.
- `git fsck --full --no-progress` - passed; reported dangling commit `10624c1cb973bff9eebabbc81e0fa62c9a568dd9`, which is acceptable.
- Full Cargo verification was skipped because this sprint did not touch Rust, Cargo files, schemas, or native Tauri automation.

## Non-Goals

- No app semantics or product behavior changes; the only app-source change is a lifecycle cleanup that allows existing modal UI to render in the browser harness.
- No Rust, schema, MCP, AI, sync server, release packaging, sample data, or CI changes.
- No native Tauri automation and no native file-dialog automation.
