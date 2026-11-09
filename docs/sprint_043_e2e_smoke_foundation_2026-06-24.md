# Sprint 043 - E2E Smoke Foundation

Date: 2026-06-24
Branch: `codex/e2e-smoke-foundation`

## Scope

Sprint 043 adds a narrow Playwright browser-smoke foundation for build spec section 17.3. The suite validates the Vite-rendered app shell and stable hash-route rendering only.

## Changes

- Added root `npm run test:e2e` using Playwright.
- Added Chromium-only Playwright configuration with a Vite web server.
- Added browser smoke coverage for the dashboard shell plus Contacts, Pipeline, and Settings hash routes.
- Added a test-only Tauri IPC shim for read-only startup and route data calls.
- Tightened the existing hash-route renderer so direct hash URLs are read during browser client startup.
- Added CI steps to install Playwright Chromium and run the smoke suite.
- Documented that this is browser/Vite smoke coverage, not native Tauri automation.

## Explicit Boundaries

- No new product features or broad UI changes.
- No native Tauri dialog automation.
- No release packaging work.
- No MCP runtime, AI behavior, sync server behavior, or schema changes.
- No broad UI changes.

## Verification

Expected local gate:

```bash
npm run test:e2e
```

Full sprint acceptance also keeps the existing lint, type-check, unit-test, build, Rust formatting, clippy, check, and test gates.
