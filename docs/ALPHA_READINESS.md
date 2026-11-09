# Alpha Release Readiness Audit

Date: 2026-06-27
Last updated: 2026-09-10
Branch: `codex/alpha-release-readiness-audit`
Baseline: canonical `main` before this branch was `c912f669baf7bfdaac9dab163a15e1e0fe08c6ab`.

This audit maps the requested Build Phases 0-7 to the current repository docs,
workflow files, and sprint ledger state. It is an evidence report only; it does
not add runtime behavior, schema, UI, MCP behavior, AI behavior, sync server
behavior, dependencies, release publishing, or package-generation changes.

## Executive Verdict

900CRM appears close to the requested alpha end state when run from source:
Phases 0-5 are materially complete for the current accepted scope, with the
caveats documented below. Phase 6, Alpha Release, is not complete because the
project still lacks produced and published Windows, macOS, and Linux installer
evidence. Phase 7 is complete only for the currently accepted local stdio MCP
scope. A separate future network MCP server package remains deferred unless it
is intentionally pulled into scope later.

The product-depth review in
[Product Review and Competitive Benchmark](PRODUCT_REVIEW_AND_BENCHMARK.md)
adds an important caveat: source readiness does not mean the app already feels
competitive in daily CRM use. Recent post-MVP work has started navigation
grouping, first-run onboarding, sample data, and contact-level Customer 360
summary. The next product phase should keep deepening customer/account
workspaces, pipeline guidance, activities/calendar workflows, reports, and
navigation structure before broad new platform work.

The previously active GitHub Actions billing/spending-limit blocker was
resolved as of 2026-08-07: the CI suite now runs green across Ubuntu, Windows,
and macOS. The remaining Phase 6 (Alpha Release) gaps are distribution items
(macOS signing/notarization credentials, code signing), not CI availability.

Local verification can increase confidence in source health, but it does not
prove installability for non-technical users. Installability needs actual
platform packages, checksums/metadata, and platform smoke evidence.

## Status Vocabulary

- `Currently accepted`: repo evidence supports the current scoped requirement;
  this does not mean every possible future enhancement is finished.
- `Materially complete`: the phase appears close enough for alpha-source use,
  with known caveats documented in current repo docs.
- `Incomplete/blocker`: required alpha proof is missing or blocked.
- `Deferred`: intentionally future work, not counted as complete.

## Phase Map

### Phase 0 - Repository Foundation, Mission, And Guardrails

Status: `Currently accepted` / materially complete.

Current evidence:

- `README.md` presents the project mission, offline-first target, supported
  development setup, release status, roadmap, and public links.
- `ARCHITECTURE.md` documents the Tauri/Svelte/Rust two-process architecture,
  data flow, offline-first principles, public baseline docs, and key design
  decisions.
- `CONTRIBUTING.md` documents development commands, CI requirements, branch and
  commit conventions, and contributor expectations.
- `docs/OPEN_SOURCE_GUARDRAIL_CHECKLIST.md` captures the recurring
  offline-first, local-first, low-resource, open-source, and workflow checks.
- `package.json` exposes root verification and release-metadata helper scripts.

Remaining caveats:

- GitHub Actions now provides fresh passing evidence (the billing/spending-limit
  blocker noted above was resolved as of 2026-08-07).
- The foundation is sufficient for source contributors, but does not satisfy
  non-technical installation or release distribution by itself.

### Phase 1 - Offline Desktop CRM Product Baseline

Status: `Currently accepted` / materially complete.

Current evidence:

- `README.md` documents Contacts, Leads, Organizations, Pipeline, Activities,
  Dashboard, Reports, Review, Search, Import/Export, Backup/Restore, Settings
  panes, i18n, and offline-first behavior as current user-facing surfaces.
- `ARCHITECTURE.md` maps the desktop shell, Tauri IPC command layer,
  `crm-core`, storage, stores, routes, and frontend API wrappers.
- `docs/DATA_MODEL.md` states the current schema is version 14 and documents
  contacts (including person lifecycle and optional owner), organizations,
  deals, activities, notes, tags, custom fields, entity links, saved views,
  settings, search, reports, audit, sync metadata, proposed actions, backups,
  and migration history.
- `docs/sprint_ledger.md` records completed feature and hardening sprints for
  core CRM workflows through Sprint 092.

Remaining caveats:

- Several compatibility bridges remain intentional: legacy organization
  contacts, legacy relationship mirror fields, mirrored tag stores, and
  `content`/`body` note columns.
- This phase is assessed as source/current-scope readiness. It is not proof of
  packaged cross-platform desktop installation.

### Phase 2 - Local Data Durability, Privacy, And Accountability

Status: `Currently accepted` / materially complete.

Current evidence:

- `docs/DATA_MODEL.md` documents local SQLite as the source of truth, WAL mode,
  migration versioning, audit log rows, sync changelog rows, external-client
  readiness records, proposed actions, and backup metadata.
- `docs/BACKUP_RESTORE.md` documents local backup creation, validation,
  destructive restore confirmation, automatic pre-import backups, and
  summary-scoped import rollback boundaries.
- `docs/PRIVACY.md` documents offline-first operation, no telemetry or
  analytics, local storage, optional network touchpoints, MCP/AI non-behavior,
  and security caveats.

Remaining caveats:

- 900CRM does not provide application-level database encryption, app lock,
  encrypted backups, encrypted exports, or active sync credential management.
- Real multi-device sync transport is not implemented; only local sync metadata
  and settings/status foundations exist.
- Optional email settings, if entered, are local settings strings; stronger
  secret storage is deferred.

### Phase 3 - Import, Export, Backup, And Data Operations

Status: `Currently accepted` / materially complete.

Current evidence:

- `docs/IMPORT_EXPORT.md` documents current local CSV and JSON import/export
  for contacts, deals, activities, organizations, notes, tag definitions,
  custom field definitions, and tag links.
- `docs/IMPORT_EXPORT.md` also documents export-only audit logs, proposed
  actions, external clients, and external-client permissions.
- `docs/BACKUP_RESTORE.md` documents automatic backups before supported
  write-import flows and separate row-level rollback behavior.
- `samples/contacts.csv` and `samples/organizations.csv` provide synthetic
  manual import smoke data referenced by `docs/RELEASE.md`.

Remaining caveats:

- Audit logs, proposed actions, external clients, and external-client
  permissions are export-only diagnostics, not import/activation/replay paths.
- There is no cloud import/export service or automatic upload.
- Backups and exports are local unencrypted files; users must choose trusted or
  encrypted storage when data sensitivity requires it.

### Phase 4 - Verification, Public Docs, And Release Guardrails

Status: `Currently accepted` / materially complete for source verification;
Actions evidence is now available (billing blocker resolved 2026-08-07).

Current evidence:

- `.github/workflows/ci.yml` defines Ubuntu verification for PRs and `main`,
  including release-note sample generation, release-manifest sample generation,
  release guardrails, frontend checks/tests/build, browser smoke tests, Rust
  formatting, Clippy, workspace check, and workspace tests.
- `docs/RELEASE.md` records the manual release checklist, release artifact
  expectations, release guardrail scan, and intentionally not implemented
  release systems.
- `scripts/verify-public-release-guardrails.mjs` is wired through
  `npm run check:release-guardrails` and is documented in
  `docs/OPEN_SOURCE_GUARDRAIL_CHECKLIST.md`.
- `docs/sprint_078_public_release_guardrail_scan_2026-06-26.md` through
  `docs/sprint_081_release_notes_generation_gate_2026-06-26.md` record the
  public-release guardrail, preflight, manifest sample, and release-note sample
  gates.

Remaining caveats:

- Fresh GitHub Actions evidence is now available (the billing/spending-limit
  blocker was resolved 2026-08-07).
- Source verification does not replace platform installer generation or manual
  cross-platform desktop smoke testing.

### Phase 5 - External Client, Proposed Action, And SDK Readiness

Status: `Currently accepted` / materially complete for the current local
readiness scope.

Current evidence:

- `docs/DATA_MODEL.md` documents external clients, active permission modes
  (`disabled`, `read_only`, `draft_only`), permission rows, permission/audit
  evidence, proposed actions, and the reviewed `create_activity_draft`
  execution path after approval.
- `docs/MCP_READINESS.md` documents the optional `crm-sdk` facade, local
  external-client permission evaluation, audit evidence, pending proposed
  actions, and the current accepted local stdio MCP boundary.
- `README.md` and `ARCHITECTURE.md` both state that MCP is optional and not
  started by the desktop app.

Remaining caveats:

- Broader write modes such as `write_with_confirmation` and `write_allowed`
  remain inactive.
- There is no built-in AI agent, model-provider integration, token/secret
  surface, network listener, or sync-server behavior in this phase.

### Phase 6 - Alpha Release

Status: `Incomplete/blocker`.

Current evidence:

- `README.md` states that pre-built release installers are not published yet.
- The README roadmap still leaves `Windows, macOS, Linux release installers`
  unchecked.
- `docs/RELEASE.md` states that normal CI is verification-only and does not
  build or publish release installers.
- `.github/workflows/release.yml` defines a manual release packaging workflow
  with preflight, package matrix, artifact upload, release metadata, checksums,
  SBOM-shaped output, and guarded optional GitHub Release publishing.
- `npm run release:preflight:local` now gives maintainers a local source
  preflight that mirrors the workflow preflight commands where feasible without
  installing OS packages, installing Playwright browsers, building installers,
  signing, notarizing, uploading artifacts, or publishing a release.
- `npm run release:artifacts:verify` now gives maintainers a deterministic
  local integrity check for already downloaded manual workflow package,
  checksum, metadata, and SBOM artifacts once Actions can produce them.
- `docs/sprint_096_macos_package_smoke_evidence_2026-06-27.md` records a local
  macOS run where `npm run release:preflight:local` passed, the optimized
  macOS release binary built, `target/release/bundle/macos/900CRM.app` was
  created, and the release binary launched briefly with disposable app data.
  DMG finalization did not complete: one attempt hit the sprint cap, one retry
  failed in the generated `bundle_dmg.sh`, and a control-tower rerun stalled in
  `osascript` while customizing the temporary mounted image. No final
  `target/release/bundle/dmg/900CRM_1.0.0_aarch64.dmg` was produced.
- `npm run release:macos:dmg:headless` now provides a macOS-only maintainer
  helper that bypasses Finder/AppleScript DMG customization, creates a basic
  compressed DMG from `900CRM.app` plus an `Applications` symlink, verifies it
  with `hdiutil verify`, and prints its size and SHA-256. This is local
  unsigned, unnotarized package evidence only; it is not workflow artifact
  proof, not Windows/Linux proof, not GitHub Release proof, and not release
  completion.
- `npm run release:macos:smoke:local` now provides a repeatable local macOS
  package smoke wrapper. It runs the headless DMG helper, verifies the image,
  generates macOS-only checksums, release metadata, and SBOM output under ignored
  local artifact directories, runs the artifact verifier with `--platforms
  macos`, and mounts the DMG to confirm `900CRM.app` plus the `Applications`
  symlink are present. This remains local unsigned and unnotarized maintainer
  evidence only.
- `docs/RELEASE.md` lists manual release verification and intentionally not
  implemented release systems such as signing, notarization, default automatic
  publishing, upgrade/update channel, telemetry, and crash reporting.

Remaining before Phase 6 can be called complete:

- ~~Resolve the GitHub Actions billing/spending-limit blocker so jobs can start.~~
  (Resolved 2026-08-07; CI runs green across Ubuntu/Windows/macOS.)
- Run the manual release packaging workflow from the intended ref and preserve
  evidence that preflight and all package jobs completed.
- Produce and inspect Windows and Linux package artifacts. (macOS `.dmg` is
  deferred until Apple Developer ID signing + notarization are available.)
- Produce workflow-backed macOS package evidence. A local headless DMG helper
  and local macOS smoke wrapper can create and inspect a basic unsigned and
  unnotarized DMG for maintainer evidence, but they do not replace manual
  workflow artifacts, signing, notarization, downloaded-artifact verification,
  cross-platform package proof, or end-user install smoke.
- Produce and preserve checksums, release metadata, SBOM-shaped dependency
  inventory, and release notes.
- Run the downloaded-artifact verifier against the real workflow artifacts and
  preserve the command output as release evidence.
- Smoke test the packaged desktop app on each target platform.
- Publish or draft a GitHub Release only through the guarded release process.
- Keep signing, notarization, auto-update, telemetry, and crash reporting marked
  as not implemented until a future sprint intentionally adds them.

Local `npm`, `cargo`, browser, and `npm run release:preflight:local` checks can
be green while this phase remains incomplete. They do not prove that a
non-technical user can install and launch 900CRM from a release artifact.

### Phase 7 - Separate MCP Server Package

Status: `Currently accepted` for local stdio scope; `Deferred` for future
network MCP server/package scope.

Current evidence:

- `docs/MCP_READINESS.md` now separates the current accepted local stdio
  checklist from the deferred future MCP checklist.
- The current accepted checklist is complete for an optional local `crm-mcp`
  package with deterministic metadata probes, a disabled-by-default runtime
  guard/config/status model, a config-gated local stdio loop, SDK-routed read
  tools, and the reviewed `create_activity_draft` pending-action flow.
- `docs/MCP_READINESS.md` explicitly states that the desktop app and `crm-core`
  do not start an MCP server, bind a localhost listener, expose prompt/resource
  surfaces, or manage MCP tokens/secrets.
- `README.md` states that MCP support is optional, local stdio only for the
  accepted scope, and not a network server, token surface, AI agent, or direct
  write runtime.

Remaining/deferred if Phase 7 is interpreted as a separate future MCP server
package:

- Network listener binding and exact localhost validation at bind time.
- Explicit server process enablement UX and operational docs.
- Authentication token/client secret design, storage, rotation, revocation, and
  auditability.
- Prompt and resource surface design, implementation, and tests.
- Broader write modes or execution paths beyond the reviewed
  `create_activity_draft` flow.
- Packaging, release, and installation proof for the separate MCP package.

The current accepted Phase 7 conclusion is therefore narrow: the local stdio
MCP boundary is accepted; a future network server package is not complete.

## Primary Alpha Gap

The main remaining alpha gap is distribution proof, not source feature shape.
The repository has a manual packaging workflow and release metadata helpers.
The Actions billing/spending-limit blocker was resolved (2026-08-07) and the CI
suite runs green across Ubuntu, Windows, and macOS, but a manual packaging run
that produces and smoke-tests real platform artifacts (and, for macOS,
signing/notarization credentials) has not yet been completed. Until platform
artifacts are produced and smoke-tested, 900CRM should continue to be described
as source-evaluable rather than installable by non-technical alpha users.
