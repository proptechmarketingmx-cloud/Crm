# Release Readiness

Date: 2026-06-24
Last updated: 2026-09-10

This document records the current release status for 900CRM and the manual
checks required before publishing a public release. For a phase-by-phase
current-vs-remaining assessment, see
[Alpha Release Readiness Audit](ALPHA_READINESS.md).

## Current Status

900CRM is not published as a packaged public desktop release yet.
Application manifests currently identify the source tree as `0.9.0`
(source-evaluable alpha), not a stable `1.0.0`.

A draft GitHub prerelease `v0.9.0` now holds unsigned Windows and Linux
packages from workflow run 31685003675. They are not public until a
maintainer publishes the draft. Those published draft Linux binaries still
require **glibc 2.39+** (Ubuntu 24.04 / Debian 13). New Linux packages from
this tree are compiled and linked inside an `ubuntu:22.04` container
(glibc 2.35, webkit2gtk-4.1) even though the GitHub job stays on a current
hosted runner. Debian 12 remains a supported runtime, not the link image.
macOS remains deferred.

Local Linux link check (requires Docker; does not install a .deb):

```
bash scripts/ci/linux-ubuntu-22.04-verify.sh
```

Current CI is verification-only, and release packaging is intentionally
separate. Manual release packaging now has an automated Ubuntu preflight gate
before any platform packaging starts:

- it runs on `ubuntu-latest`;
- it verifies sample release manifest generation, the public-release guardrail
  scan, frontend linting, type checks, unit tests, production build, and browser
  smoke tests;
- it verifies Rust formatting, Clippy, workspace checks, and workspace tests;
- package generation does not start unless that preflight passes;
- normal CI still does not build or publish release installers.

Manual release packaging now lives in
[`.github/workflows/release.yml`](../.github/workflows/release.yml). It must be
started with `workflow_dispatch`, runs the preflight gate, then builds Windows,
macOS, and Linux Tauri bundles, uploads workflow artifacts, generates
repo-owned release notes, and generates per-platform SHA-256 checksums, release
metadata, and an SPDX-shaped dependency inventory. The workflow does not
publish a GitHub Release unless
`publish_github_release` is explicitly enabled for a matching `v`-prefixed tag
ref.

Downloaded workflow artifacts can now be checked locally with
`npm run release:artifacts:verify -- --artifact-root release-download
--release-version <version>`. This verifier is integrity evidence for already
downloaded workflow outputs only. It does not generate installers, sign,
notarize, create tags, upload assets, publish a release, or prove end-user
installability.

The current repository can be built and tested locally by contributors, but the
presence of source code, CI checks, or Tauri configuration is not a release
artifact.

Previous external blocker (resolved): GitHub Actions check runs were
previously not starting because of account billing/spending-limit state. As
of 2026-08-07, Actions jobs start and pass normally (the full CI suite ran
green across Ubuntu, Windows, and macOS on recent pull requests). The
remaining release blockers are therefore product/distribution items
(macOS notarization credentials, code signing), not CI availability.

Local macOS package smoke evidence from 2026-06-27 is recorded in
[Sprint 096](sprint_096_macos_package_smoke_evidence_2026-06-27.md). That run
passed `npm run release:preflight:local`, built the optimized macOS release
binary, created `target/release/bundle/macos/900CRM.app`, and launched the
release binary briefly with disposable app data. DMG finalization did not
complete: one attempt hit the sprint cap, one retry failed in the generated
`bundle_dmg.sh`, and a control-tower rerun stalled in `osascript` while
customizing the temporary mounted image. It did not produce
`target/release/bundle/dmg/900CRM_1.0.0_aarch64.dmg`, did not generate package
metadata for a final DMG, and does not prove Windows, Linux, downloaded
workflow artifacts, GitHub Release, signing, notarization, or end-user
installability.

Maintainers can now produce a local headless macOS DMG with
`npm run release:macos:dmg:headless`. The helper builds the Tauri `.app`
bundle with app-only bundling, stages `900CRM.app` plus an `Applications`
symlink, creates a compressed DMG with `hdiutil create -srcfolder`, and verifies
the image with `hdiutil verify`. It intentionally avoids the generated
Finder/AppleScript DMG customization path that blocked Sprint 096. This helper
is local package evidence only: the output is unsigned, unnotarized, not a
manual workflow artifact, not Windows or Linux proof, not GitHub Release proof,
not end-user installability proof, and not alpha release completion.

Maintainers can also run `npm run release:macos:smoke:local` for a fuller local
macOS package smoke baseline. That command uses the headless DMG helper,
verifies the DMG with `hdiutil verify`, copies the DMG into an ignored local
artifact layout under `dist/local-macos-package-smoke/`, generates macOS-only
checksums, release metadata, and SBOM output, runs the release artifact verifier
with `--platforms macos`, and mounts the DMG to confirm `900CRM.app` and the
`Applications` symlink are present. It is still local unsigned and unnotarized
maintainer evidence only; it does not replace workflow artifacts, platform
installer smoke, signing, notarization, downloaded artifact verification for
real workflow output, GitHub Release proof, or alpha release completion.

## Verification Checklist

Before a release candidate is tagged or published, maintainers should complete
the following checks from a clean checkout. The command-only gates are enforced
automatically by the manual release packaging workflow preflight before package
artifacts are built; the platform, data, signing, and release-note checks remain
manual.

- [ ] Confirm GitHub Actions jobs can start and pass. (This was previously
  blocked by an account billing/spending-limit issue, since resolved as of
  2026-08-07; the CI suite now runs green across Ubuntu, Windows, and macOS.)
- [ ] Confirm the repository contains no local machine paths, private hostnames,
  secrets, tokens, or real customer data in source, docs, scripts, samples, or
  packaged assets.
- [ ] Run `npm ci`.
- [ ] Run `npm run release:preflight:local` as a maintainer local source gate
  when GitHub Actions preflight evidence is unavailable or before dispatching
  the manual release packaging workflow. This is source/preflight evidence only;
  it does not prove installer generation, signing, notarization, GitHub Release
  publication, or end-user installability.
- [ ] For local macOS package evidence, run
  `npm run release:macos:dmg:headless` and preserve the printed output path,
  size, SHA-256, and `hdiutil verify` result. This proves only that a local
  unsigned, unnotarized headless DMG can be created from the app bundle; it does
  not replace workflow macOS artifacts, signing, notarization, release metadata,
  downloaded-artifact verification, Windows/Linux package proof, GitHub Release
  proof, or end-user release completion.
- [ ] For a fuller local macOS smoke baseline, run
  `npm run release:macos:smoke:local` and preserve the printed repo-relative DMG
  path, size, SHA-256, local artifact verification root, metadata path, and
  mounted-DMG layout smoke result. This is still macOS-only, local, unsigned,
  unnotarized maintainer evidence.
- [ ] Run `npm run release:notes:sample`.
- [ ] Run `npm run release:manifest:sample`.
- [ ] Run `npm run release:artifacts:verify:sample` to exercise the downloaded
  artifact verifier with deterministic synthetic artifacts.
- [ ] After the manual workflow produces real artifacts, download the workflow
  artifacts and run `npm run release:artifacts:verify -- --artifact-root
  release-download --release-version <version>`.
- [ ] Run `npm run check:release-guardrails`.
- [ ] Run `npm run lint`.
- [ ] Run `npm run check`.
- [ ] Run `npm run test`.
- [ ] Run `npm run build`.
- [ ] Run `npm run test:e2e`.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --workspace -- -D warnings`.
- [ ] Run `cargo check --workspace`.
- [ ] Run `cargo test --workspace`.
- [ ] Manually smoke test the desktop app on every target platform that will
  receive an installer.
- [ ] Refresh the application icon set before publishing. The source SVG is
  `apps/desktop/src-tauri/icons/app-icon.svg`; regenerate all platform icon
  assets (PNG, `.icns`, `.ico`, and the Android/iOS sets under
  `apps/desktop/src-tauri/icons/`) from it using the Tauri icon generator
  (`npm run tauri -- icon <path-to-source-svg>`) so packaged installers and
  taskbar/dock/Store icons are consistent and up to date.
- [ ] Confirm that code signing and notarization are still required and
  arranged before public distribution. The repository does not perform signing
  or macOS notarization today (see "Still Not Implemented" below); unsigned and
  unnotarized packages are release-candidate evidence only and must not be
  distributed to non-technical end users.
- [ ] Manually import the synthetic CSV samples under `samples/` for contacts
  and organizations.
- [ ] Manually create, edit, search, export, back up, validate, and restore a
  small local dataset.
- [ ] Confirm the privacy, import/export, backup/restore, MCP readiness, README,
  and changelog docs still describe only implemented behavior.
- [ ] Confirm the release notes list known gaps and deferred systems clearly.


### Public Release Guardrail Scan

`npm run check:release-guardrails` runs a deterministic scan over tracked
repository text files. It catches high-confidence local-machine path leaks,
private/internal host URLs, and literal credential patterns before a release or
PR is accepted. The scan intentionally does not treat ordinary variable names
such as `token`, placeholder values, empty passwords, or generic `/tmp/...` test
paths as release blockers.

This automated check is a guardrail, not a substitute for manual review.
Maintainers must still confirm that samples, screenshots, docs, and packaged
assets do not contain real customer data or other sensitive material that a
pattern scan cannot reliably identify.

## Release Artifacts and Metadata

A public desktop release should include, at minimum:

- Windows installer: `.msi` or `.exe`.
- macOS disk image: `.dmg` (currently deferred — requires Apple Developer ID
  signing + notarization, not yet available).
- Linux package: `.deb`.
- Linux portable image: `.AppImage`.
- Checksums for every uploaded artifact.
- Software bill of materials (SBOM) for release contents.
- Release notes with known limitations and verification scope.
- License and third-party notice material where required by dependencies.
- Platform-specific signing and notarization evidence where applicable.

## Manual Packaging Workflow

Maintainers can create unsigned release-candidate packages from a clean checkout
by dispatching the `Manual Release Packaging` workflow with:

- `release_version`: the version label recorded in artifact metadata.
- `release_title`: optional human-readable release title.
- `release_tag`: required only when attaching artifacts to a GitHub Release.
- `publish_github_release`: default `false`; set to `true` only from the
  matching `refs/tags/v...` workflow ref.
- `release_draft`: default `true` for guarded GitHub Release creation.

The workflow produces two artifact groups for each platform:

- desktop packages built by Tauri under
  `target/release/bundle`;
- release notes from `scripts/generate-release-notes.mjs`, written to
  `release-notes.md` for guarded GitHub Release creation;
- release metadata from `scripts/generate-release-manifest.mjs`, including
  `*-SHA256SUMS.txt`, `*-release-metadata.json`, and `*-sbom.spdx.json`.

The packaging matrix depends on the `preflight` job. That job installs the same
Ubuntu system dependencies used by CI, runs `npm ci`, generates the sample
release notes and sample release manifest, checks public release guardrails,
runs the documented frontend and browser smoke gates, and runs the documented
Rust formatting, lint, check, and test gates. A failing preflight blocks all
package jobs before any release artifacts are generated.

The release-note and manifest scripts are also locally runnable for validation:

```bash
npm run release:preflight:local
npm run release:macos:dmg:headless
npm run release:macos:smoke:local
npm run release:notes:sample
node scripts/generate-release-notes.mjs --help
npm run release:manifest:sample
node scripts/generate-release-manifest.mjs --help
npm run release:artifacts:verify:sample
npm run release:artifacts:verify -- --artifact-root release-download --release-version 1.0.0
node scripts/verify-release-artifacts.mjs --help
```

`npm run release:preflight:local` is the maintainer-run local equivalent of the
manual release workflow's source preflight. It runs the sample release notes,
sample release manifest, deterministic sample downloaded-artifact verification,
public release guardrail scan, frontend lint, frontend type check, frontend
tests, frontend build, browser smoke test, Rust formatting, Rust Clippy, Rust
workspace check, and Rust workspace test commands in sequence, failing on the
first failed command with the step label. It intentionally does not install
Ubuntu system dependencies, install Playwright browsers, run package matrix
jobs, generate installers, sign, notarize, tag, upload artifacts, or publish a
GitHub Release. If Playwright browsers or platform prerequisites are missing
locally, the browser smoke step fails and maintainers should record that failure
honestly rather than treating the local preflight as release proof.

`npm run release:macos:dmg:headless` is a macOS-only maintainer helper for the
Sprint 096 DMG blocker. It builds or reuses
`target/release/bundle/macos/900CRM.app`, creates an ignored output such as
`target/release/bundle/dmg/900CRM_1.0.0_aarch64.headless.dmg`, verifies that
DMG with `hdiutil verify`, and prints its size and SHA-256. It does not invoke
the normal Tauri DMG Finder/AppleScript customization flow, generate release
metadata, sign, notarize, tag, upload artifacts, publish a GitHub Release, or
prove Windows/Linux installability.

`npm run release:macos:smoke:local` is a macOS-only maintainer wrapper around
the headless DMG helper plus the existing release metadata and artifact
verification scripts. It writes generated evidence under ignored `dist/` and
`target/` directories, verifies only `macos` metadata with the artifact
verifier, and performs a mounted-DMG layout smoke for `900CRM.app` and the
`Applications` symlink. It should be treated as repeatable local smoke evidence,
not as release workflow proof or end-user distribution readiness.

### Downloaded Artifact Verification

After a successful manual release packaging workflow run, maintainers should
download all workflow artifacts into a single `release-download/` directory and
run:

```bash
npm run release:artifacts:verify -- --artifact-root release-download --release-version <version>
```

The verifier expects metadata for `windows`, `macos`, and `linux` by default.
**Note:** the manual packaging workflow currently builds only `windows` and
`linux`; macOS (`.dmg`) is deferred until Apple Developer ID signing +
notarization credentials are available (see "Still Not Implemented" below).
When verifying a Windows+Linux-only run, pass `--platforms windows,linux` so
the verifier does not expect macOS metadata that was intentionally not produced.
It checks that each platform has release metadata, SHA-256 checksums, and an
SPDX 2.3 SBOM; that metadata uses schema version 1 and the requested release
version; that metadata artifacts have `fileName`, `relativePath`, `kind`,
`sizeBytes`, and 64-character SHA-256 values; that package files under the
download root match the metadata size and SHA-256; that checksum entries match
metadata exactly; that SBOM JSON has non-empty packages; and that workflow
package kinds are present for Windows (`.msi` and NSIS/`.exe`) and Linux
(`.deb` and `.AppImage`). macOS (`.dmg`) verification applies only once macOS
builds are re-enabled with signing/notarization.

`npm run release:artifacts:verify:sample` creates a deterministic synthetic
downloaded-artifact tree under ignored `dist/release-artifact-verifier-sample/`
and verifies it. This sample command is a local script test, not release
artifact evidence.

If real release artifacts are not present and `--sample` is not used, the script
fails with a message naming the missing artifact directory or expected package
suffixes.

## Alpha Readiness Audit

The current alpha-readiness evidence is tracked in
[Alpha Release Readiness Audit](ALPHA_READINESS.md). That audit treats source
feature readiness and release distribution proof separately: Phases 0-5 are
materially complete for the current accepted source scope, Phase 6 remains
incomplete until packaged installer and release evidence exists, and Phase 7 is
complete only for the accepted local stdio MCP scope while future network MCP
server work remains deferred.

## Still Not Implemented

The following release systems remain intentionally not implemented in the
current repository state:

- artifact signing;
- macOS notarization;
- automatic publishing by default;
- release attachment automation without an explicit `publish_github_release`
  dispatch input and matching `refs/tags/v...` guard;
- upgrade/update channel;
- release telemetry or crash reporting.

These gaps are release-engineering work for a future sprint. They are not app
runtime behavior, schema behavior, MCP behavior, AI behavior, or sync-server
behavior.

## Sample Data

The repository includes small synthetic CSV samples for manual import smoke
testing:

- `samples/contacts.csv`
- `samples/organizations.csv`

The sample rows are fake, culturally neutral, and not intended to represent real
people or organizations. They use the current documented CSV fields in
[Import and Export](IMPORT_EXPORT.md).
