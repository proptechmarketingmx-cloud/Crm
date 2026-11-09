# Contributing to 900CRM

Welcome, and thank you for considering a contribution to 900CRM. Every contribution — no matter how small — matters.

This project exists to serve people everywhere: developers in communities with limited connectivity, beginners new to open source, and experienced engineers who want to build something meaningful. **You belong here.** If you have never contributed to an open-source project before, this guide is written especially for you. We will walk you through everything.

If you get stuck at any point, please open a [discussion](https://github.com/900Labs/900CRM/discussions) and ask. There are no dumb questions.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Code Style](#code-style)
- [Commit Messages](#commit-messages)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Types of Contributions](#types-of-contributions)
- [Translation Guide](#translation-guide)
- [Module Ownership](#module-ownership)
- [Plugin Development Guide](#plugin-development-guide)
- [Issue Reporting Guide](#issue-reporting-guide)
- [Architecture Decision Records](#architecture-decision-records)
- [Good First Issues](#good-first-issues)

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold it. Please read it before contributing. We take it seriously — everyone deserves to feel safe and respected here.

Violations can be reported to **conduct@900labs.com**.

---

## Getting Started

Here is how to go from zero to a running development environment.

### 1. Fork the repository

On GitHub, click the **Fork** button at the top right of the repository page. This creates your own copy of the project under your GitHub account.

### 2. Clone your fork

```bash
git clone https://github.com/YOUR_USERNAME/900crm.git
cd 900crm
```

### 3. Add the upstream remote

```bash
git remote add upstream https://github.com/900Labs/900CRM.git
git remote -v
# You should see both origin (your fork) and upstream (the main repo)
```

### 4. Create a branch

Always work on a branch, never directly on `main`:

```bash
git checkout -b your-branch-name
```

Use a descriptive branch name:
- `fix/contact-import-crash-on-utf8`
- `feature/activity-reminders`
- `i18n/add-portuguese-translation`
- `docs/fix-architecture-diagram`

### 5. Install dependencies

Make sure all [prerequisites](README.md#prerequisites) are installed first, then:

```bash
# Install root workspace dependencies, including the npm-managed Tauri CLI
npm install
```

### 6. Run in development mode

```bash
npm run tauri -- dev
```

The 900CRM window should open. You are ready to work.

### 7. Keep your branch up to date

For longer-running work, sync regularly with upstream:

```bash
git fetch upstream
git rebase upstream/main
```

---

## Development Environment

### Prerequisites

| Tool | Version | How to install |
|---|---|---|
| Node.js | 20 or later | [nodejs.org](https://nodejs.org/) |
| npm | 10 or later | Included with Node.js |
| Rust (stable) | 1.70 or later | [rustup.rs](https://rustup.rs/) |
| Git | Any recent version | [git-scm.com](https://git-scm.com/) |

Platform-specific system dependencies are listed in the [README Quick Start](README.md#quick-start).

### Recommended VS Code Extensions

| Extension | Purpose |
|---|---|
| `rust-lang.rust-analyzer` | Rust language server — autocomplete, errors, go-to-definition |
| `svelte.svelte-vscode` | Svelte 5 syntax, linting, and component intelligence |
| `esbenp.prettier-vscode` | Auto-format on save |
| `tauri-apps.tauri-vscode` | Tauri-specific snippets and helpers |
| `EditorConfig.EditorConfig` | Respects `.editorconfig` settings |

### Useful Commands

```bash
# Start the desktop app with hot-reload
npm run tauri -- dev

# Start only the Svelte dev server
npm run dev

# Type-check the desktop frontend
npm run check

# Check the Rust workspace
cargo check --workspace

# Lint the frontend workspace
npm run lint

# Format frontend and docs files
npm run format

# Run Rust linting
cargo clippy --workspace -- -D warnings

# Format Rust code
cargo fmt --all

# Run Rust unit and integration tests
cargo test --workspace

# Run frontend tests (vitest)
npm run test

# Build the frontend production bundle
npm run build

# Manual desktop package build (not CI release automation)
npm run tauri -- build
```

`npm run lint` uses ESLint's flat config for JavaScript repo tooling and config
files (`.js`, `.mjs`, and `.cjs`). TypeScript and Svelte diagnostics are covered
by `npm run check`; extending ESLint to those file types should add the required
parser/plugin dependencies in the same change.

---

## Project Structure

Understanding where things live helps you know where to make changes.

```
900crm/
├── apps/
│   └── desktop/
│       ├── src/                  # Svelte 5 frontend source
│       │   ├── lib/
│       │   │   ├── components/   # Reusable Svelte components
│       │   │   ├── stores/       # Svelte 5 rune-based reactive state
│       │   │   ├── api/          # Typed wrappers around Tauri invoke calls
│       │   │   ├── i18n/         # Translation JSON files + i18n loader
│       │   │   ├── services/     # Frontend service helpers
│       │   │   └── utils/        # Date formatting, CSV helpers, validators
│       │   ├── routes/           # SvelteKit routes and route components
│       │   ├── app.css           # Global styles and design tokens
│       │   ├── app.d.ts
│       │   └── app.html
│       ├── src-tauri/            # Tauri shell and IPC command layer
│       │   ├── src/
│       │   │   ├── main.rs       # Native entry point
│       │   │   ├── lib.rs        # Tauri app setup, plugin registration
│       │   │   ├── state.rs      # Managed application state
│       │   │   └── commands/     # Tauri #[tauri::command] IPC handlers
│       │   ├── capabilities/     # Tauri v2 IPC permission definitions
│       │   ├── icons/            # App icons (all required sizes)
│       │   └── tauri.conf.json   # App identifier, window config, bundle config
│       ├── package.json
│       ├── svelte.config.js
│       ├── tsconfig.json
│       └── vite.config.ts
├── crates/
│   ├── crm-core/                 # Shared Rust CRM domain, storage, services
│   │   └── src/
│   │       ├── crm_engine/       # Business logic, decoupled from Tauri
│   │       ├── storage/          # Database layer (rusqlite)
│   │       ├── domain/           # Domain models
│   │       ├── services/         # Application services
│   │       ├── search/           # Search support
│   │       └── import_export/    # CSV import/export support
│   ├── crm-mcp/                  # MCP integration placeholder
│   └── crm-sdk/                  # SDK placeholder
├── scripts/                      # Root verification helpers
├── plugins/                      # Plugin directory (v2.0 planned)
│   └── README.md                 # Plugin development guide
└── .github/
    ├── workflows/
    │   └── ci.yml                # Ubuntu PR and main-branch verification
    └── pull_request_template.md  # Pull request checklist and guardrails
```

---

## Code Style

Consistent code style makes the project easier to read and review for everyone, including people who are reading it in their second or third language.

### Rust

- **Formatting:** Run `cargo fmt --all` before committing. CI will reject unformatted code.
- **Linting:** Run `cargo clippy --workspace -- -D warnings`. Fix all warnings. Do not use `#[allow(...)]` without a comment explaining why.
- **Documentation:** All public functions, structs, enums, and traits must have `///` doc comments. Explain *what* the item does and *why* it exists.

  ```rust
  /// Moves a deal from one pipeline stage to another.
  ///
  /// Updates the deal's `stage_id` and records the transition in the
  /// `changelog` table for sync purposes. Returns an error if either
  /// the deal or target stage does not exist.
  pub fn move_deal_to_stage(
      conn: &Connection,
      deal_id: i64,
      stage_id: i64,
  ) -> Result<Deal> {
      // ...
  }
  ```

- **Error handling:** Use `anyhow::Result` for commands; use `thiserror` for domain-specific error types. Do not use `.unwrap()` or `.expect()` in production code — use `?` or explicit handling.
- **No unsafe code** without prior discussion. If you believe it is necessary, open an issue first.

### TypeScript

- **Strict mode:** `apps/desktop/tsconfig.json` has `strict: true`. Do not disable it.
- **Explicit return types** on all exported functions.
- **JSDoc on exports:**

  ```typescript
  /**
   * Formats a deal's monetary value for display.
   * @param value - The deal value in base currency units
   * @param currency - ISO 4217 currency code (e.g. "KES", "USD")
   * @returns Formatted string, e.g. "KES 150,000"
   */
  export function formatDealValue(value: number, currency: string): string {
    // ...
  }
  ```

- **No `any`:** Use `unknown` for genuinely unknown types and narrow explicitly.

### Svelte (Svelte 5)

- **Use runes:** Use `$state`, `$derived`, `$effect`, and `$props`. Do not use the legacy `writable`/`readable` store API for new code.
- **Component documentation header:** Every `.svelte` file should start with:

  ```svelte
  <!--
    @component
    DealCard

    Renders a single deal card on the kanban pipeline board.
    Shows the deal name, value, contact, and days since last activity.

    Props:
      deal: The deal object to display
      onMove: Called when the user drags the card to a new stage
      onClick: Called when the user clicks the card to open the detail panel
  -->
  ```

- **Typed props:**

  ```svelte
  <script lang="ts">
    interface Props {
      deal: Deal;
      onMove: (dealId: number, stageId: number) => void;
      onClick: (dealId: number) => void;
    }
    let { deal, onMove, onClick }: Props = $props();
  </script>
  ```

- **Accessibility:** All interactive elements must have accessible labels. Svelte's accessibility warnings are enabled and treated as errors in CI.

### CSS

- Reference CSS custom properties defined in `apps/desktop/src/app.css`. Do not hardcode colour values, spacing, or font sizes.

  ```css
  /* Good */
  .deal-card {
    background: var(--color-surface);
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
  }
  ```

---

## Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/).

### Format

```
<type>(<scope>): <short description>

[optional body]

[optional footer]
```

### Types

| Type | When to use |
|---|---|
| `feat` | A new feature visible to users |
| `fix` | A bug fix |
| `docs` | Documentation changes only |
| `style` | Formatting, whitespace — no logic change |
| `refactor` | Code restructuring — no feature change, no bug fix |
| `perf` | Performance improvement |
| `test` | Adding or fixing tests |
| `chore` | Build system, dependency updates, CI changes |
| `i18n` | Translation or internationalization changes |

### Scope (optional but encouraged)

Use the area of the codebase: `contacts`, `pipeline`, `activities`, `dashboard`, `search`, `sync`, `import`, `i18n`, `ci`, etc.

### Examples

```
feat(pipeline): add drag-and-drop deal reordering within a stage

fix(contacts): prevent crash when importing CSV with empty name field

i18n(ar): add Arabic translations for pipeline view

chore(deps): update tauri to 2.2.0

test(sync): add changelog merge conflict resolution tests

docs: add module ownership table to CONTRIBUTING.md
```

### Breaking Changes

```
feat(storage)!: change database schema to support custom fields

BREAKING CHANGE: Schema migrates from version 2 to version 3 automatically
on first launch. Data from v1.0.0 is fully preserved.
```

---

## Testing

### Running Tests

```bash
# Run all Rust unit and integration tests
cargo test --workspace

# Run a specific Rust test
cargo test --workspace test_contact_import_utf8

# Run frontend unit tests (vitest)
npm run test

# Run browser smoke tests (Playwright Chromium against Vite)
npm run test:e2e

# Run frontend tests in watch mode
npm run test -- --watch

# Type-check the frontend
npm run check
```

### Writing Rust Tests

Add unit tests at the bottom of each file in a `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_contact_creates_and_retrieves() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_database(&db_path).unwrap();

        let new_contact = NewContact {
            name: "Amara Diallo".to_string(),
            email: Some("amara@example.com".to_string()),
            phone: None,
            tags: vec!["lead".to_string()],
        };

        let contact = create_contact(&conn, new_contact).unwrap();
        assert_eq!(contact.name, "Amara Diallo");
        assert_eq!(contact.email, Some("amara@example.com".to_string()));

        let retrieved = get_contact(&conn, contact.id).unwrap();
        assert_eq!(retrieved.id, contact.id);
    }
}
```

### Writing Frontend Tests (vitest)

```typescript
// apps/desktop/src/lib/utils/formatters.test.ts
import { describe, it, expect } from 'vitest';
import { formatDealValue } from './formatters';

describe('formatDealValue', () => {
  it('formats USD correctly', () => {
    expect(formatDealValue(150000, 'USD')).toBe('USD 150,000');
  });

  it('formats KES correctly', () => {
    expect(formatDealValue(500000, 'KES')).toBe('KES 500,000');
  });

  it('handles zero', () => {
    expect(formatDealValue(0, 'USD')).toBe('USD 0');
  });
});
```

### What Needs Tests

When adding new code, write tests for:
- Data transformation and business logic functions
- Error cases (invalid input, missing records, empty data)
- Database CRUD operations (can you save and retrieve correctly?)
- CSV import edge cases (UTF-8, special characters, empty fields, duplicate records)
- Sync merge logic (conflict resolution, last-write-wins behavior)

---

## Pull Request Process

### Before You Open a PR

1. All tests pass: `cargo test --workspace` and `npm run test`
2. No lint errors: `npm run lint` and `cargo clippy --workspace -- -D warnings`
3. Code is formatted: `npm run format` and `cargo fmt --all`
4. If your change touches the UI, include a screenshot or screen recording
5. Update documentation affected by your change
6. Complete the mission guardrails checklist in `docs/OPEN_SOURCE_GUARDRAIL_CHECKLIST.md`

### PR Description

Fill in all sections of the PR template:
- What does this PR do?
- Why is this change needed?
- How was it tested?
- Screenshots (for UI changes)
- Checklist
- Open-source and low-resource guardrail confirmations

### Review Process

1. A maintainer will review your PR within 5 business days (larger PRs may take longer).
2. Reviewers may request changes. This is normal — it is how we make the code better together.
3. Once approved and CI passes, a maintainer will merge.
4. Your contribution will appear in the next release's changelog.

### CI Requirements

Initial GitHub Actions CI runs on `ubuntu-latest` for pull requests and pushes
to `main`. It is a verification workflow only; release installers and
cross-platform packaging are not built by CI yet.

See [Release Readiness](docs/RELEASE.md) for the current release status, manual
verification checklist, and future packaging artifacts.

All PRs must pass:
- `npm run lint` — ESLint for JavaScript repo tooling/config files
- `npm run check` — TypeScript and Svelte diagnostics
- `npm run test` — Frontend unit tests
- `npm run build` — Frontend production build
- `npm run test:e2e` — Browser smoke tests for the Vite-rendered app shell and hash routes
- `cargo fmt --all -- --check` — Rust formatting check
- `cargo clippy --workspace -- -D warnings` — Rust linting
- `cargo check --workspace` — Rust workspace compile check
- `cargo test --workspace` — Rust unit tests

---

## Types of Contributions

### Bug Reports

If you find a bug, [open an issue](https://github.com/900Labs/900CRM/issues/new?template=bug_report.md) using the bug report template. Include:

- What you expected to happen
- What actually happened
- Steps to reproduce (be as specific as possible)
- Your operating system and version
- Any relevant data (e.g., a CSV file that fails to import — remove any sensitive data)

### Feature Requests

Have an idea for something 900CRM should do? [Open a feature request](https://github.com/900Labs/900CRM/issues/new?template=feature_request.md). Describe the problem you are solving, how you imagine the feature working, and why it would be useful — especially in low-connectivity contexts.

### Code Contributions

For small fixes (typos, obvious bugs), open a PR directly. For larger features or significant refactors, open an issue first to discuss the approach. This prevents you from spending time on work that may not align with the project direction.

### Translations

Translating 900CRM into a new language is one of the most impactful contributions you can make. See the [Translation Guide](#translation-guide) below. Languages we especially need: Yoruba, Tagalog, and Amharic.

### Documentation Improvements

Found something confusing? A command that doesn't work? A concept that needs a better explanation? Open a PR — documentation improvements are always welcome and are an excellent first contribution.

### Testing

We need more test coverage, particularly for:
- CSV import edge cases (malformed files, encoding issues, large files)
- Sync conflict resolution logic
- Search ranking and relevance
- Database migration correctness

---

## Translation Guide

All user-facing strings live in JSON files under `apps/desktop/src/lib/i18n/`. Adding a new language requires only five steps.

### Adding a New Language

**Step 1 — Copy the English base file**

```bash
cp apps/desktop/src/lib/i18n/en.json apps/desktop/src/lib/i18n/LANGUAGE_CODE.json
```

Replace `LANGUAGE_CODE` with an [IETF language tag](https://www.iana.org/assignments/language-subtag-registry) (e.g., `pt` for Portuguese, `vi` for Vietnamese, `ha` for Hausa).

**Step 2 — Translate the values**

Open the new file and translate all the string values. Do not change the keys — only the values.

```json
{
  "nav.contacts": "Contacts",       ← English key (do not change)
  "nav.contacts": "Contacts"        ← Translate the VALUE, not the key
}
```

For example, in French:
```json
{
  "nav.contacts": "Contacts",
  "nav.pipeline": "Pipeline",
  "nav.activities": "Activités",
  "nav.dashboard": "Tableau de bord"
}
```

**Step 3 — Register the language**

In `apps/desktop/src/lib/i18n/index.ts`, add your language metadata to `availableLocales`:

```typescript
export const availableLocales: LocaleInfo[] = [
  { code: 'en', name: 'English', nativeName: 'English', direction: 'ltr' },
  // ...
  { code: 'pt', name: 'Portuguese (Brazil)', nativeName: 'Português (Brasil)', direction: 'ltr' },
];
```

Then register lazy loading in the `loadLocale` switch:

```typescript
switch (code) {
  // ...
  case 'pt':
    module = await import('./pt.json');
    break;
}
```

**Step 4 — RTL languages (Arabic, Hebrew, Urdu, etc.)**

Set `direction: 'rtl'` in your locale metadata entry in `availableLocales`. The layout will automatically mirror:

```typescript
{ code: 'ar', name: 'Arabic', nativeName: 'العربية', direction: 'rtl' }
```

**Step 5 — Submit a pull request**

Open a PR with your new translation file. In the PR description, indicate what percentage of strings are translated and your native language proficiency. Partial translations are welcome — another community member can complete them later.

### Updating an Existing Translation

If you find a translation that is incorrect, unclear, or outdated:
1. Edit the relevant `apps/desktop/src/lib/i18n/LANGUAGE_CODE.json` file
2. Submit a PR with a brief explanation of the change

---

## Module Ownership

| Module | Path | Primary Maintainer | Description |
|---|---|---|---|
| CRM Engine — Contacts | `crates/crm-core/src/crm_engine/contacts.rs` | Community | Contact domain logic |
| CRM Engine — Deals | `crates/crm-core/src/crm_engine/deals.rs` | Community | Pipeline and deal logic |
| CRM Engine — Activities | `crates/crm-core/src/crm_engine/activities.rs` | Community | Activity scheduling |
| CRM Engine — Search | `crates/crm-core/src/crm_engine/search.rs` | Community | FTS5 index management |
| Storage Layer | `crates/crm-core/src/storage/` | 900 Labs Core | SQLite schema and queries |
| Sync Engine | `crates/crm-core/src/storage/sync.rs` | 900 Labs Core | Changelog-based sync |
| Tauri Commands | `apps/desktop/src-tauri/src/commands/` | Community | IPC command handlers |
| Frontend — Pipeline | `apps/desktop/src/lib/components/KanbanBoard.svelte` | Community | Kanban UI components |
| Frontend — Contacts | `apps/desktop/src/lib/components/ContactCard.svelte` | Community | Contact UI components |
| Frontend — Activities | `apps/desktop/src/lib/components/ActivityFeed.svelte` | Community | Activity UI components |
| Frontend — Dashboard | `apps/desktop/src/routes/Dashboard.svelte` | Community | Dashboard metrics UI |
| Internationalization | `apps/desktop/src/lib/i18n/` | Community | Translation files |
| CI workflow | `.github/workflows/` | 900 Labs Core | Pull request and main-branch verification |

"Community" means the module is open for community contributions. "900 Labs Core" means changes should be discussed in an issue first.

---

## Plugin Development Guide

> **Status: Coming in v2.0**
>
> The plugin API is planned for the v2.0 release. The architecture is defined but not yet implemented.

See [plugins/README.md](plugins/README.md) for the full specification of the planned plugin system, including the manifest format, available extension points, and an example plugin skeleton.

If you are interested in shaping the plugin API design before it is implemented, open a [discussion](https://github.com/900Labs/900CRM/discussions) tagged `plugin-system`. Community input during the design phase is especially valuable.

---

## Issue Reporting Guide

### Bug Reports

Use the [bug report template](https://github.com/900Labs/900CRM/issues/new?template=bug_report.md). A good bug report includes:

1. **Clear title** — describe the problem in one sentence (e.g., "App crashes when importing CSV with Arabic characters")
2. **Steps to reproduce** — numbered steps, as specific as possible
3. **Expected behavior** — what should have happened
4. **Actual behavior** — what actually happened, including any error messages
5. **Environment** — operating system, OS version, 900CRM version
6. **Additional context** — screenshots, sample data (remove any sensitive information)

The more specific your report, the faster we can diagnose and fix the issue.

### Feature Requests

Use the [feature request template](https://github.com/900Labs/900CRM/issues/new?template=feature_request.md). A good feature request explains:

1. **The problem** — what are you currently unable to do?
2. **Your proposed solution** — how you imagine the feature working
3. **Alternatives considered** — other approaches you have tried
4. **User impact** — who else would benefit, especially in low-connectivity contexts

Feature requests are not guaranteed to be implemented, but they all inform the roadmap. The most impactful requests for users in developing nations are prioritized.

### Claiming an Issue

Leave a comment on the issue saying you would like to work on it. A maintainer will assign it to you. This prevents two contributors from working on the same thing simultaneously.

---

## Architecture Decision Records

There is no `docs/adr/` directory and no ADR process in this repository.
Design rationale for the current stack lives in
[ARCHITECTURE.md](ARCHITECTURE.md#key-design-decisions).

If you propose a significant technical change (new dependency, architectural
shift, or data-model change), explain the options and the tradeoff in the PR
so reviewers can see the reasoning.

---

## Good First Issues

**What makes a good first issue?**
- Clearly described with enough context to understand the problem
- Limited scope — fixable by changing a small area of the codebase
- Does not require deep knowledge of the whole system

**Where to find them:**
Look for the [`good first issue`](https://github.com/900Labs/900CRM/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) label on the issue tracker.

**Claiming an issue:**
Comment on the issue that you would like to work on it. A maintainer will assign it.

**Ask if unsure:**
Leave a comment in the issue asking for clarification. That is exactly what issue comments are for.

---

Thank you for being here. Whatever you contribute, you are helping build tools that matter.

— The 900CRM maintainers
