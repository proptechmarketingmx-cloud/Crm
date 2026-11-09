# Sprint 050: Canonical Repository URL Alignment

Date: 2026-06-24
Branch: `codex/canonical-repository-url-alignment`

## Scope

Align current public self-repository references with the canonical repository:

`https://github.com/900Labs/900CRM`

This sprint is limited to public docs, plugin docs, package metadata, workspace Cargo metadata, and the app-visible Settings GitHub link. It does not change product behavior, dependencies, lockfiles, schema, MCP, AI, sync server, release packaging, or CI semantics.

## Evidence

- `git remote -v` points at `https://github.com/900Labs/900CRM`.
- `gh repo view 900Labs/900CRM` resolves.
- The legacy hyphenated/lowercase owner-repo slug does not resolve.
- Existing current public self-links used non-canonical hyphenated and lowercase owner-repo variants across README, architecture, changelog, contribution, plugin, package, Cargo, and Settings surfaces.

## Changes

- Updated README badge, action, language anchor, clone command, good-first-issue, discussions, and issue tracker links to `900Labs/900CRM`.
- Updated architecture, changelog, contribution, and plugin documentation self-links to `900Labs/900CRM`.
- Updated `package.json` repository metadata to `https://github.com/900Labs/900CRM`.
- Updated workspace `Cargo.toml` package metadata to `https://github.com/900Labs/900CRM`.
- Updated the Settings About GitHub link to `https://github.com/900Labs/900CRM`.

## Verification Plan

- Confirm no active current self-links remain for the non-canonical hyphenated or lowercase owner-repo variants in public/docs/app surfaces.
- Confirm canonical `github.com/900Labs/900CRM` references appear in the expected files.
- Run the standard frontend, Rust, diff, status, and repository integrity checks before closing the sprint.
