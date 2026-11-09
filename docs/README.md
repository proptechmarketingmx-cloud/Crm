# 900CRM Documentation Index

There is no GitHub wiki for this repository (`hasWikiEnabled` is false).
Public documentation lives in `docs/` and the root [README](../README.md).

This directory contains two kinds of documentation:

- Current product truth: documents that describe the app, data model, release
  status, privacy posture, import/export behavior, backup/restore behavior, and
  accepted MCP boundary as they exist now.
- Historical sprint evidence: `sprint_*.md` files, `sprint_ledger.md`, and
  dated handoff notes. These are audit records. They are not current product
  truth.

Product HEAD for this index: `e3ec7b93` on `main` (2026-08-17 daily-CRM work
through #177, including owner names and the morning-queue owner filter).

## Start Here

| Need | Document |
|---|---|
| What the app is, how to run it from source | [Root README](../README.md) |
| Product status, feature depth, and competitive gaps | [Product Review and Competitive Benchmark](PRODUCT_REVIEW_AND_BENCHMARK.md) |
| Alpha source vs distribution readiness | [Alpha Release Readiness Audit](ALPHA_READINESS.md) |
| Release packaging and artifact requirements | [Release Readiness](RELEASE.md) |
| v1.0 signing and installer checklist | [Release Roadmap](ROADMAP.md) |
| Speed, UX, and older-machine plan (not the signing checklist) | [Product Roadmap](PRODUCT_ROADMAP.md) |
| Local schema, migrations, and data boundaries | [Data Model](DATA_MODEL.md) |
| Import/export formats and limitations | [Import and Export](IMPORT_EXPORT.md) |
| Backup and destructive restore workflow | [Backup and Restore](BACKUP_RESTORE.md) |
| Privacy, local data, and network behavior | [Privacy](PRIVACY.md) |
| MCP current scope and deferred future scope | [MCP Readiness Baseline](MCP_READINESS.md) |
| Public release hygiene checklist | [Open Source Guardrail Checklist](OPEN_SOURCE_GUARDRAIL_CHECKLIST.md) |
| Architecture and design rationale | [Architecture](../ARCHITECTURE.md) |
| How to contribute, translate, and open PRs | [Contributing](../CONTRIBUTING.md) |
| Notable changes | [Changelog](../CHANGELOG.md) |

`docs/ROADMAP.md` is the v1.0 release and signing checklist. It is not a
product-improvement strategy. `docs/PRODUCT_ROADMAP.md` is the speed/UX/older-machine
plan. It is not the signing/release checklist.

## Sprint Records

`sprint_*.md` files and [Sprint Ledger](sprint_ledger.md) are historical audit
notes. They record what each narrow sprint changed, what it did not change,
and which verification commands ran at the time. Prefer the current-truth
documents above when making product, release, or roadmap decisions.

`handoff_repo_audit_fixes_2026-08-16.md` is a dated handoff note from that
audit pass. Treat it as history.

## Documentation Rules

- Do not make README or release docs claim installable public artifacts until
  package artifacts exist and have been smoke-tested.
- Keep future MCP network server, real sync transport, built-in AI behavior,
  signing, notarization, telemetry, and auto-update work clearly labeled as
  deferred unless an implementation sprint actually lands it.
- When product behavior changes, update the durable current-state document
  first, then add a sprint note if the work needs audit evidence.
- Do not put local machine paths, private hostnames, secrets, tokens, or real
  customer data in public docs.
