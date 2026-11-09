# Release Roadmap

Date: 2026-08-13
Status: Source-evaluable alpha (public), version `0.9.0`. Binary release in progress.

The speed/UX/older-machine plan lives in [PRODUCT_ROADMAP.md](PRODUCT_ROADMAP.md).
This file stays the v1.0 release and signing checklist.

This document is the single source of truth for what remains before 900CRM
ships a real, distributable release. It replaces ad-hoc checklist notes and
is kept in sync with the current codebase state.

---

## Current State

900CRM is a **public, source-evaluable alpha**. Contributors can clone, build,
test, and run the app from source. All CI gates pass across Ubuntu, Windows,
and macOS. The repository is governed (branch protection, issue templates,
security scanning, release guardrails).

What is NOT done: a distributable binary release with installers that a
non-technical user can install.

---

## v1.0 Release Criteria

These must ALL be satisfied before a public `v1.0.0` tag is cut.

### Blockers (cannot ship without these)

- [ ] **Apple Developer ID + notarization.** macOS `.dmg` builds are removed
      from the release workflow until this is available. Without it, Gatekeeper
      blocks installation for the target audience.
- [ ] **Windows code-signing certificate.** Without it, SmartScreen shows a
      warning. An EV cert removes the warning entirely; an OV cert is the
      minimum.
- [ ] **First tagged release with real installers.** Run the Manual Release
      Packaging workflow, produce Windows `.msi`/`.exe` + Linux `.deb`/
      `.appimage`, verify artifacts, and publish a GitHub Release.
- [ ] **Auto-update channel configured end-to-end.** The updater plugin is
      wired up. What remains: add `TAURI_SIGNING_PRIVATE_KEY` and
      `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` as GitHub Actions secrets, produce
      a signed release, and verify the updater can fetch + apply an update.
      A new updater keypair was generated on 2026-08-13 because the previous
      private key was not available on the maintainer machine and no public
      installer yet depends on the old pubkey. The public key is in
      `tauri.conf.json`. The private key and password are held outside the
      repo and must stay backed up.

### Strongly Recommended (before promoting beyond alpha)

- [x] **Restore reopen-failure resilience, TS/Svelte ESLint, bounded list
      windows.** Landed in `#142`. Remaining: true SQL `LIMIT`/`OFFSET` for
      deals/activities, and `f64` → integer minor-unit currency.
- [ ] **Translations reviewed by native speakers.** All 10 locales now have
      100% key parity. The non-English translations are machine-assisted
      and need community review before they are considered production-quality.
- [ ] **Product-depth pass.** See `docs/PRODUCT_REVIEW_AND_BENCHMARK.md` for
      the competitive gap analysis and recommended depth sprints.
- [ ] **Replace provisional app icon** with official 900 Labs branding.

### Nice to Have (post-v1.0)

- [ ] **macOS builds re-enabled** once Apple Developer ID is available.
- [ ] **Auto-update silent/automatic mode** (currently user-initiated only).
- [ ] **Application-level data encryption** (SQLCipher or OS keychain).
- [ ] **Multi-device sync transport** (local changelog foundation exists).
- [ ] **Plugin system** (v2.0 roadmap item).

---

## Release Process (when criteria are met)

1. Ensure all CI gates pass on `main`.
2. Run the **Manual Release Packaging** workflow (`release.yml`) with
   `release_version`, `publish_github_release: true`, `release_draft: true`,
   and a matching `v`-prefixed tag.
3. Download artifacts and run
   `npm run release:artifacts:verify -- --artifact-root release-download
   --release-version <version> --platforms windows,linux`.
4. Smoke-test installers on each target platform.
5. Publish the GitHub Release (un-draft).
6. Verify the updater endpoint (`latest.json`) is reachable and the app can
   self-update.

---

## Translations Note

All 10 supported languages (en, fr, es, ar, sw, pt, vi, ha, bn, hi) now have
100% translation-key parity. The non-English translations were produced with
machine assistance and need review by native speakers. To contribute a
correction, see the [Translation Guide](../CONTRIBUTING.md#translation-guide).

---

## Updater Signing Keys

The Tauri updater requires a signing keypair:

- **Public key** — committed in `apps/desktop/src-tauri/tauri.conf.json` under
  `plugins.updater.pubkey`. Safe to share.
- **Private key + password** — held securely by maintainers. Must be added as
  GitHub Actions secrets `TAURI_SIGNING_PRIVATE_KEY` and
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` before the first signed release.
  **If these are lost, all installed copies can never update** — back them up.
