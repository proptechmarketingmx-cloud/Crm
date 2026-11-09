# Sprint 049 - README Screenshot Baseline

Date: 2026-06-24
Branch: `codex/readme-screenshot-baseline`
Scope: Public README screenshot assets and documentation wiring only.

## Summary

- Replaced the five top-level README screenshot placeholders with committed PNG assets under `docs/assets/readme/`.
- Added `scripts/capture-readme-screenshots.mjs` as an opt-in helper for regenerating the README screenshots from the Vite-rendered browser app shell.
- Repaired the dashboard capture so the public README overview image waits for rendered KPI values, report values, and seeded recent activity instead of loading placeholders.
- Moved the Data Management screenshot from the opening overview to the Backup / Restore section.
- Kept the README release wording source-only; this sprint does not add or imply packaged desktop releases.
- Updated the sprint ledger with the screenshot baseline entry.

## Screenshot Assets

- `docs/assets/readme/900crm-data-management.png`
- `docs/assets/readme/900crm-contacts.png`
- `docs/assets/readme/900crm-pipeline.png`
- `docs/assets/readme/900crm-activities.png`
- `docs/assets/readme/900crm-dashboard.png`

## Capture Method

The screenshots were generated with:

```bash
node scripts/capture-readme-screenshots.mjs
```

The helper starts the desktop Vite app on localhost, installs a test-only Tauri IPC shim before app code loads, navigates the same hash routes used by the browser smoke tests, seeds the contacts, pipeline, and activities screens through visible UI actions, and writes the PNGs to `docs/assets/readme/`. The helper is not part of `npm run test:e2e`, so normal E2E CI should not dirty the worktree.

## Boundaries

- No product/runtime behavior changes.
- No Rust, schema, MCP, AI, sync server, release packaging, CI, or dependency changes.
- No generated test reports, traces, build output, `node_modules`, local paths, or packaged-release claims are part of the committed docs/assets change.

## Notes

The contacts, pipeline, and activities screenshots use clearly synthetic records created through the visible browser-smoke UI. The dashboard screenshot uses the same test-only Tauri IPC shim as the browser smoke path and waits for concrete KPI, report, and recent-activity values before capture. This sprint did not change product/runtime loader behavior.

## Verification

Closeout verification for this sprint:

- [x] README/docs screenshot-placeholder search returned no matches.
- [x] README image links exist and are non-empty.
- [x] Screenshot dimensions and file sizes inspected: five PNGs at 1380 x 952, each under 120 KB.
- [x] `npm run check` - passed with 0 errors and 0 warnings.
- [x] `npm run test` - passed, 19 files and 84 tests.
- [x] `npm run test:e2e` - passed, 6 Playwright Chromium tests.
- [x] `npm run build` - passed; Vite/SvelteKit static build completed.
- [x] `git diff --check main...HEAD` - passed with no output.
- [x] `git status --short --branch` - only intended sprint files were modified or untracked before commit.
- [x] `git fsck --full --no-progress` - exited 0 and reported one dangling commit.
