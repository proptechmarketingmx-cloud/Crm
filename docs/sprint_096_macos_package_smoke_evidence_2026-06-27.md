# Sprint 096 - macOS Package Smoke Evidence

Date: 2026-06-27
Branch: `codex/macos-package-smoke-evidence`
Baseline: `8082d62b338694d707fe888e66b30fbacf089273`

## Scope

Produce local macOS package-build and smoke evidence from this Mac as partial
Phase 6 alpha-release evidence while GitHub Actions remains externally blocked.
This sprint does not claim alpha release completion, Windows package proof,
Linux package proof, GitHub Release proof, signing, notarization, publishing,
or release workflow artifact proof.

## Commands And Results

```bash
npm ci
```

Result: passed. Installed 243 packages and audited 245 packages. `npm` reported
16 dependency audit findings: 1 low, 9 moderate, 5 high, and 1 critical. This
sprint did not remediate dependency versions.

```bash
npm run release:preflight:local
```

Result: passed. The local preflight completed all 13 source gates:

- sample release notes generation;
- sample release manifest generation;
- deterministic sample downloaded-artifact verification;
- public release guardrail scan over 342 tracked text files;
- frontend lint;
- frontend type check with `svelte-check` reporting 0 errors and 0 warnings;
- frontend Vitest suite with 21 files and 155 tests passing;
- frontend production build;
- Playwright browser smoke with 6 tests passing;
- Rust formatting check;
- Rust Clippy workspace lint;
- Rust workspace check;
- Rust workspace tests, including 150 `crm-core` tests, 21 `crm-mcp` library
  tests, 28 `crm-mcp` CLI tests, 7 `crm-sdk` tests, and 12 desktop Tauri
  library tests passing.

```bash
npm --workspace apps/desktop run tauri -- build --bundles dmg
```

First result: partial/blocked by the 10-minute control-tower cap. The command:

- completed the frontend production build;
- completed the optimized Tauri release build;
- built the release binary at `target/release/ninehundredcrm`;
- built the app bundle at `target/release/bundle/macos/900CRM.app`;
- entered DMG bundling for
  `target/release/bundle/dmg/900CRM_1.0.0_aarch64.dmg`;
- produced a temporary writable image at
  `target/release/bundle/macos/rw.32336.900CRM_1.0.0_aarch64.dmg`;
- did not produce a final `.dmg` under `target/release/bundle/dmg/` before the
  sprint cap.

The package process was stopped with `kill 32336 28837 28810` and exited with
code 143. The temporary `rw.*.dmg` is not release evidence and was not used to
claim a successful package.

Replacement-builder retry:

```bash
npm --workspace apps/desktop run tauri -- build --bundles dmg
```

Result: failed before the cap with exit code 1 in the generated
`target/release/bundle/dmg/bundle_dmg.sh` step. The retry:

- rebuilt the frontend production bundle;
- completed the optimized Tauri release build in `1m 50s`;
- rebuilt the release binary at `target/release/ninehundredcrm`;
- rebuilt the app bundle at `target/release/bundle/macos/900CRM.app`;
- entered DMG bundling for
  `target/release/bundle/dmg/900CRM_1.0.0_aarch64.dmg`;
- produced another temporary writable image at
  `target/release/bundle/macos/rw.39399.900CRM_1.0.0_aarch64.dmg`;
- did not produce a final `.dmg` under `target/release/bundle/dmg/`.

The retry failed with:

```text
failed to bundle project error running bundle_dmg.sh: `failed to run target/release/bundle/dmg/bundle_dmg.sh`
```

The generated DMG script did not surface a more specific inner `hdiutil` or
AppleScript error in the captured command output. A previous temporary image
mount, `/dev/disk4` with a generated `dmg.*` volume name, was confirmed to belong to
`target/release/bundle/macos/rw.32336.900CRM_1.0.0_aarch64.dmg` and was
detached cleanly with `hdiutil detach /dev/disk4`. A follow-up `hdiutil info`
showed no remaining 900CRM temporary DMG mount.

Control-tower reproduction:

```bash
npm --workspace apps/desktop run tauri -- build --bundles dmg
```

Result: interrupted after reaching the same generated DMG script. The command
rebuilt the frontend, rebuilt the optimized release binary, and entered:

```text
Running bundle_dmg.sh
```

Process inspection showed the script waiting in Finder customization:

```text
bash .../target/release/bundle/dmg/bundle_dmg.sh --volname 900CRM --icon 900CRM.app 180 170 --app-drop-link 480 170 --window-size 660 400 --hide-extension 900CRM.app 900CRM_1.0.0_aarch64.dmg 900CRM.app
/usr/bin/osascript /var/folders/.../createdmg.tmp... dmg.iKPmAs
```

`hdiutil info` showed the temporary image mounted with a generated `dmg.*`
volume name
from `target/release/bundle/macos/rw.46208.900CRM_1.0.0_aarch64.dmg`. After a
bounded wait with no progress, the control tower interrupted the packaging
command with `Ctrl-C`, then detached the generated temporary mount with:

```bash
hdiutil detach <temporary-dmg-mount>
```

Result: `"disk4" ejected.` A final `hdiutil info` check showed no remaining
900CRM temporary mount.

Artifact observations after all attempts:

```text
target/release/ninehundredcrm                                      7.5M
target/release/bundle/macos/900CRM.app/Contents/MacOS/ninehundredcrm 7.5M
target/release/bundle/macos/rw.32336.900CRM_1.0.0_aarch64.dmg       32M
target/release/bundle/macos/rw.39399.900CRM_1.0.0_aarch64.dmg       32M
target/release/bundle/macos/rw.46208.900CRM_1.0.0_aarch64.dmg       32M
```

SHA-256 values recorded for traceability only:

```text
17afabac022d87953b394f0e963994688c4d7491e8c4b317e7b1a9cc11d37225  target/release/ninehundredcrm
17afabac022d87953b394f0e963994688c4d7491e8c4b317e7b1a9cc11d37225  target/release/bundle/macos/900CRM.app/Contents/MacOS/ninehundredcrm
cb0f76a5e25597f50ac462c0bc52564cbfd979b338f1d1f1b7b874aa13514ed4  target/release/bundle/macos/rw.32336.900CRM_1.0.0_aarch64.dmg
25d6b9577e6a666d107fab3c9de5177104f3e8287f1c3dd25d76b1c23234b2fe  target/release/bundle/macos/rw.39399.900CRM_1.0.0_aarch64.dmg
bda6f2e79e1b190f35ff4baebbc062c8f25ef276fdfc655eda14bffb4b33f3d1  target/release/bundle/macos/rw.46208.900CRM_1.0.0_aarch64.dmg
```

## Smoke Evidence

The final DMG did not complete, so no installed-from-DMG smoke was performed.
As a limited release-binary smoke, the built binary was launched with disposable
home/data paths:

```bash
mkdir -p "$TMPDIR/900crm-sprint096-home" "$TMPDIR/900crm-sprint096-data"
env HOME="$TMPDIR/900crm-sprint096-home" XDG_DATA_HOME="$TMPDIR/900crm-sprint096-data" ./target/release/ninehundredcrm
```

Result: the process stayed running for 5 seconds with no immediate loader or
runtime error and was stopped with `Ctrl-C`, exiting 130. The launch created
only disposable app data under the temporary smoke `HOME`, in
`Library/Application Support/com.900labs.crm/`, with `900crm.db`,
`900crm.db-shm`, and `900crm.db-wal`.

This is a binary-start smoke only. It is not GUI automation, not a DMG install
smoke, and not end-user installability proof.

## Metadata And Verification Limitation

`scripts/generate-release-manifest.mjs` was not run for the local macOS package
because the final `.dmg` package did not exist. The script intentionally ignores
temporary `rw.*` images, and using an incomplete temporary image would distort
the release evidence.

`npm run release:artifacts:verify -- --platforms macos` was also not run
against local package evidence because there was no final macOS package
metadata/checksum/SBOM set to verify. The verifier is workflow-layout-specific:
it expects an artifact root containing package files plus matching release
metadata, SHA-256 checksum, and SBOM artifacts. It remains appropriate for
downloaded workflow outputs or for a completed local package that has been
structured with honestly generated metadata.

## Phase 6 Impact

This sprint is completed as documentation of a local macOS package-build
attempt with an explicit final-DMG blocker. It reduces one local macOS
source/build confidence gap: the local source preflight passed, the optimized
macOS release binary built, the app bundle was created, and the binary started
against disposable app data.

Phase 6 remains incomplete. Missing proof still includes:

- completed macOS DMG finalization; local attempts reached the generated
  `bundle_dmg.sh`, with the final reproduction stalling in `osascript` while
  customizing the temporary mounted image before producing a final `.dmg`;
- generated macOS package checksums, release metadata, and SBOM for a final
  package;
- macOS DMG install/open smoke;
- Windows and Linux packages and smoke tests;
- Actions-backed package artifacts;
- downloaded-artifact verifier evidence for real workflow outputs;
- GitHub Release draft or publication proof;
- signing and notarization evidence.

Ignored generated outputs remain under `node_modules/`, `dist/`, `target/`,
`apps/desktop/.svelte-kit/`, `apps/desktop/build/`, and related build-output
directories. No binary package artifacts are committed.
