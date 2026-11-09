# 900CRM Architecture Guide

This document describes the technical architecture of 900CRM for contributors. It covers the high-level system design, module structure, data flow, offline-first principles, sync protocol, database schema, and key design decisions.

If you are new to the codebase, read this document before diving into the source. It will help you understand *why* things are built the way they are.

---

## Table of Contents

- [High-Level Overview](#high-level-overview)
- [Module Map](#module-map)
- [Frontend Architecture](#frontend-architecture)
- [Backend Architecture](#backend-architecture)
- [Data Flow](#data-flow)
- [Offline-First Design](#offline-first-design)
- [Sync Protocol](#sync-protocol)
- [Database Schema](#database-schema)
- [Public Baseline Docs](#public-baseline-docs)
- [Extension Points](#extension-points)
- [MCP Readiness](#mcp-readiness)
- [Key Design Decisions](#key-design-decisions)

---

## High-Level Overview

900CRM is a two-process desktop application built on [Tauri v2](https://tauri.app/). The two processes are:

1. **Rust backend** — a Tauri shell in `apps/desktop/src-tauri` plus shared CRM logic in `crates/crm-core`. Handles all data operations, business logic, file I/O, and import/export.
2. **WebView frontend** — a Svelte 5 application rendered in the system WebView. Handles all UI rendering and user interaction. Cannot access the OS directly.

The two processes communicate via **Tauri IPC**: the frontend calls named commands, the backend executes them, and returns results as serialized JSON.

```
┌──────────────────────────────────────────────────────────────────┐
│                         Operating System                          │
│                                                                  │
│  ┌──────────────────────────┐    ┌────────────────────────────┐  │
│  │      Rust Backend         │    │    WebView (Frontend)       │  │
│  │ apps/desktop/src-tauri    │◄──►│    apps/desktop/src         │  │
│  │ crates/crm-core           │    │    Svelte 5 / TypeScript   │  │
│  │                          │    │                            │  │
│  │  ┌────────────────────┐  │    │  ┌──────────────────────┐  │  │
│  │  │   Tauri Commands   │  │    │  │   Routes / Pages     │  │  │
│  │  │  (IPC handlers)    │  │    │  │  /dashboard          │  │  │
│  │  └────────┬───────────┘  │    │  │  /contacts           │  │  │
│  │           │              │    │  │  /pipeline           │  │  │
│  │  ┌────────▼───────────┐  │    │  │  /activities         │  │  │
│  │  │    CRM Engine       │  │    │  └──────────────────────┘  │  │
│  │  │  (business logic)   │  │    │                            │  │
│  │  └────────┬───────────┘  │    │  ┌──────────────────────┐  │  │
│  │           │              │    │  │   Components          │  │  │
│  │  ┌────────▼───────────┐  │    │  │  contacts/ pipeline/ │  │  │
│  │  │   Storage Layer     │  │    │  │  activities/dashboard│  │  │
│  │  │ (rusqlite + sync)   │  │    │  └──────────────────────┘  │  │
│  │  └────────┬───────────┘  │    │                            │  │
│  └───────────┼──────────────┘    │  ┌──────────────────────┐  │  │
│              │                   │  │   Stores (rune state) │  │  │
│              │                   │  └──────────────────────┘  │  │
│    ┌─────────▼──────────┐        │                            │  │
│    │   SQLite Database   │        │  ┌──────────────────────┐  │  │
│    │  (~/.local/share/   │        │  │   API layer           │  │  │
│    │   900CRM/900crm.db) │        │  │  (invoke wrappers)   │  │  │
│    └────────────────────┘        └──┴──────────────────────┴──┘  │
│                                                                  │
│                    Tauri IPC bridge (JSON over stdin/stdout)      │
└──────────────────────────────────────────────────────────────────┘
```

---

## Module Map

### Tauri Shell (`apps/desktop/src-tauri/src/`)

```
apps/desktop/src-tauri/src/
├── main.rs                    Native entry point.
├── lib.rs                     Configures and launches the Tauri app.
├── state.rs                   Managed application state shared by commands.
│
├── commands/                  Tauri IPC command handlers.
│   │                          These are thin wrappers: validate input, call
│   │                          crm-core services, return serialized result or error.
│   │                          No business logic lives here.
│   │
│   ├── contact_commands.rs    create_contact, get_contact, list_contacts,
│   │                          update_contact, delete_contact, restore_contact,
│   │                          search_contacts, list_contact_duplicate_candidates,
│   │                          merge_contacts
│   │
│   ├── organization_commands.rs
│   │                          create_organization, get_organization,
│   │                          list_organizations, update_organization,
│   │                          delete_organization, link_contact_to_organization
│   │
│   ├── note_commands.rs       create_note, get_note, list_notes_for_entity,
│   │                          update_note, delete_note
│   │
│   ├── tag_commands.rs        create_tag, get_tag, list_tags, update_tag,
│   │                          delete_tag, apply_tag_to_entity,
│   │                          remove_tag_from_entity, list_tags_for_entity
│   │
│   ├── deal_commands.rs       create_deal, get_deal, list_deals, update_deal,
│   │                          list_deals_by_stage, move_deal_stage, delete_deal,
│   │                          link_deal_to_organization, add_deal_contact,
│   │                          remove_deal_contact, list_deal_contacts,
│   │                          get_pipeline_summary
│   │
│   ├── activity_commands.rs   create_activity, get_activity, list_activities,
│   │                          list_activities_for_contact,
│   │                          list_activities_for_deal, list_upcoming_activities,
│   │                          mark_activity_complete, mark_activity_incomplete,
│   │                          update_activity, delete_activity,
│   │                          list_activity_links, add_activity_link,
│   │                          remove_activity_link
│   │
│   ├── import_export.rs       import_contacts_csv, export_contacts_csv,
│   │                          import_contacts_csv_with_mapping,
│   │                          preflight_contacts_csv_import,
│   │                          preflight_contacts_csv_import_with_mapping,
│   │                          import_deals_csv, export_deals_csv,
│   │                          import_organizations_csv,
│   │                          import_organizations_csv_with_mapping,
│   │                          preflight_organizations_csv_import,
│   │                          preflight_organizations_csv_import_with_mapping,
│   │                          export_organizations_csv
│   │
│   ├── dashboard_commands.rs  get_dashboard_stats
│   ├── report_commands.rs     get_pipeline_conversion_report,
│   │                          get_activity_funnel_report
│   ├── search_commands.rs     global_search
│   ├── audit_pending_commands.rs
│   │                          list_recent_audit_log,
│   │                          list_pending_proposed_actions,
│   │                          approve_proposed_action,
│   │                          reject_proposed_action
│   │
│   ├── external_client_commands.rs
│   │                          list_external_clients,
│   │                          create_external_client_placeholder,
│   │                          list_external_client_permissions,
│   │                          upsert_external_client_tool_permission,
│   │                          evaluate_external_client_tool_read_permission,
│   │                          evaluate_external_client_draft_permission
│   │
│   ├── custom_field_commands.rs
│   │                          list_custom_field_defs,
│   │                          create_custom_field_def,
│   │                          update_custom_field_def,
│   │                          delete_custom_field_def,
│   │                          set_custom_field_value,
│   │                          list_custom_field_values,
│   │                          list_custom_field_values_for_type
│   │
│   ├── email_commands.rs      test_email_server_connection
│   ├── sync_commands.rs       get_sync_status, trigger_sync
│   ├── backup_commands.rs     create_local_backup, validate_local_backup,
│   │                          restore_local_backup_to_app_data
│   └── settings_commands.rs   get_settings, get_setting, update_setting
```

### CRM Core (`crates/crm-core/src/`)

```
crates/crm-core/src/
├── crm_engine/                Business rules, fully decoupled from Tauri.
│   ├── contacts.rs            Contact validation and relationship logic
│   ├── deals.rs               Deal rules and pipeline calculations
│   ├── activities.rs          Activity scheduling and completion tracking
│   ├── pipeline.rs            Pipeline stage rules
│   └── search.rs              Search query building and ranking
│
├── storage/                   SQLite access layer. All SQL lives here.
│   ├── db.rs                  Database opening and migration setup
│   ├── contacts.rs            Contact persistence
│   ├── deals.rs               Deal persistence
│   ├── activities.rs          Activity persistence
│   ├── sync.rs                Changelog and sync persistence
│   ├── settings.rs            Settings persistence
│   └── search.rs              FTS persistence
│
├── domain/                    Domain models shared by services and storage.
├── services/                  Application services used by Tauri commands.
├── search/                    Search support modules.
├── import_export/             CSV import/export support.
├── audit/                     Audit support.
├── permissions/               Permission helpers.
└── utils/                     Shared utility modules.
```

### Rust Placeholders (`crates/crm-mcp/`, `crates/crm-sdk/`)

```
crates/crm-mcp/src/main.rs     Optional local MCP stdio boundary.
crates/crm-sdk/src/lib.rs      Read-only local SDK facade over crm-core.
```

The current MCP package is optional and is not started by the desktop app or `crm-core`.

### Frontend (`apps/desktop/src/`)

```
apps/desktop/src/
├── routes/                    SvelteKit routes and route-level components
│   ├── +layout.svelte         App shell: persistent sidebar navigation
│   ├── +page.svelte           Default route
│   ├── Dashboard.svelte       Dashboard metrics and recent activity feed
│   ├── Contacts.svelte        Contacts list with search and filter
│   ├── ContactDetail.svelte   Contact detail panel with activity timeline
│   ├── Pipeline.svelte        Kanban board — deals grouped by stage
│   ├── Activities.svelte      Activity feed with filters
│   └── Settings.svelte        Language, theme, date format, data location
│
├── lib/
│   ├── components/            Reusable Svelte components
│   ├── stores/                Svelte 5 rune-based reactive state
│   ├── api/                   Typed wrappers around Tauri invoke calls
│   ├── i18n/                  Translation files and i18n loader
│   ├── services/              Frontend service helpers
│   └── utils/                 Shared helper functions
│
├── app.css                    Global styles and design tokens
├── app.d.ts
└── app.html
```

---

## Frontend Architecture

### Route Structure

900CRM uses SvelteKit's file-system routing. Each route corresponds to a top-level navigation item. The persistent layout (`+layout.svelte`) wraps all routes and provides the sidebar navigation and top search bar.

### Component Patterns

All components follow consistent patterns:

1. **Documentation header** — every `.svelte` file starts with a `@component` JSDoc block describing its purpose and props.
2. **Typed props** — props are declared with a TypeScript `interface` and `$props()`.
3. **Rune-based state** — internal state uses `$state()`. Derived values use `$derived()`. Side effects use `$effect()`.
4. **No direct API calls** — components never call `invoke()` directly. They call store methods or API functions, which call `invoke()`.

### State Management

State is managed in store modules (`apps/desktop/src/lib/stores/`). Each store exports reactive `$state` variables and mutating functions. Components subscribe by importing the store.

```typescript
// apps/desktop/src/lib/stores/contacts.ts
let contacts = $state<Contact[]>([]);
let loading = $state(false);

export async function loadContacts() {
  loading = true;
  contacts = await api.listContacts();
  loading = false;
}

export { contacts, loading };
```

### API Layer

The `apps/desktop/src/lib/api/` layer provides typed wrappers around Tauri's `invoke()` function. This separation means:
- Components never depend on the string names of Tauri commands
- TypeScript enforces the correct argument and return types
- The API layer can be mocked in unit tests without mocking Tauri

```typescript
// apps/desktop/src/lib/api/contacts.ts
import { invoke } from '@tauri-apps/api/core';
import type { Contact, NewContact, UpdateContact } from '$lib/types';

export async function createContact(data: NewContact): Promise<Contact> {
  return await invoke<Contact>('create_contact', { data });
}

export async function listContacts(filter?: ContactFilter): Promise<Contact[]> {
  return await invoke<Contact[]>('list_contacts', { filter });
}
```

### i18n

All user-facing strings use the `t()` function from the i18n module:

```svelte
<script lang="ts">
  import { t } from '$lib/i18n';
</script>

<h1>{t('dashboard.title')}</h1>
<button>{t('contacts.add')}</button>
```

Language switching is reactive and takes effect immediately without a page reload.

---

## Backend Architecture

### Command Handlers

Commands (`apps/desktop/src-tauri/src/commands/`) are the entry points for all frontend requests. They follow a strict pattern:

1. Deserialize input (Tauri handles JSON deserialization automatically)
2. Validate input (call `crm-core` validation functions)
3. Acquire a database connection from the managed connection pool
4. Call the appropriate `crm-core` service or engine function
5. Return the result (Tauri serializes to JSON automatically)

Commands contain no business logic. They are glue between the IPC layer and the engine.

### CRM Engine

The engine (`crates/crm-core/src/crm_engine/`) contains business logic. It is part of a pure Rust crate with no Tauri dependencies, which makes it unit-testable without spinning up a Tauri application.

The engine functions take a `&Connection` parameter (the SQLite connection) and the domain input, and return `Result<T, CrmError>`. They call storage queries to read and write data, apply business rules, and return domain objects.

### Storage Layer

The storage layer (`crates/crm-core/src/storage/`) owns SQL persistence. Every query is a Rust function with typed parameters and return types. This keeps SQL auditable in one place and isolates schema changes to the shared core crate.

---

## Data Flow

The complete path of a user action through the system:

```
User clicks "Add Contact"
         │
         ▼
[Svelte component]
ContactEditor.svelte → validates form fields locally
         │
         ▼
[API layer]
apps/desktop/src/lib/api/contacts.ts → createContact(formData)
         │
         ▼
[Tauri IPC bridge]
invoke('create_contact', { data: formData })
         │ (JSON serialized)
         ▼
[Rust command handler]
apps/desktop/src-tauri/src/commands/contact_commands.rs → create_contact(data, state)
  • Input deserialized from JSON
  • State (DB connection pool) acquired
         │
         ▼
[CRM Engine]
crates/crm-core/src/crm_engine/contacts.rs → create_contact(&conn, new_contact)
  • Validates required fields
  • Normalizes tags (lowercase, trim)
  • Checks for duplicate email
  • Generates UUID for new contact
         │
         ▼
[Storage layer]
crates/crm-core/src/storage/contacts.rs → insert_contact(&conn, contact)
  • Executes INSERT INTO contacts ...
  • Inserts associated tags into contact_tags
  • Inserts changelog entry (for sync)
         │
         ▼
[SQLite database]
900crm.db → record persisted
         │
         ▼
[Result bubbles back up]
Contact struct → serialized to JSON → returned to frontend
         │
         ▼
[API layer]
createContact() resolves with Contact
         │
         ▼
[Store]
contacts store → appends new contact to local $state
         │
         ▼
[UI]
ContactList re-renders reactively with new contact visible
```

---

## Offline-First Design

Offline-first means the application works correctly when there is no internet connection, and that network connectivity is **never a prerequisite** for any feature. This is the default operating mode, not a fallback.

### Core Principle: Local SQLite is the Source of Truth

All reads and writes go to the local SQLite database. There is no "optimistic UI" backed by a remote store. Every operation is committed to local storage before it is reflected in the UI.

### No Network Dependency in the Core

The Rust backend does not make outbound network calls as part of normal CRM
operation. Optional email endpoint tests are user-initiated and can open TCP
connections to the SMTP or IMAP host entered by the user. The WebView is
restricted by the Tauri capability model and a local-first content security
policy; CRM data access still goes through Tauri IPC and `crm-core`.

### Sync Readiness

Current sync support is a local readiness foundation, not a real multi-device
sync transport. Mutating service paths write local `sync_changelog` entries, and
Settings stores `sync_enabled` plus `sync_url` values. The current desktop sync
commands report local status and trigger-state results, but they do not push to
or pull from a sync server.

See the [Sync Protocol](#sync-protocol) section for the implemented boundary and
future design direction.

### File Size Rationale

The installer is under 8 MB because 900CRM uses the system's built-in WebView (Edge on Windows, WebKit on macOS, WebKitGTK on Linux) rather than bundling Chromium. This matters for users downloading over slow or metered mobile connections — common in the regions 900CRM is built for.

---

## Sync Protocol

Real multi-device sync is not implemented in the current app. The implemented
piece is the local `sync_changelog` table and the service calls that append
entries after successful mutations.

The current changelog stores:

- entity type and entity ID;
- field name, including sentinel values such as `__create__` and `__delete__`;
- old and new values when available;
- timestamp;
- originating device ID.

Future sync work should keep the current offline-first guarantee: all writes go
to local SQLite first, and network transport must be optional. A future sync
server, conflict-resolution policy, tombstone model, and credential story need
separate implementation, tests, and public documentation before they are treated
as supported behavior.

---

## Database Schema

The current source-derived data model baseline is documented in
[Data Model](docs/DATA_MODEL.md). That document is the preferred reference for
schema version, core entities, migration history, audit/sync readiness, and
legacy compatibility caveats.

At a high level, current local storage includes contacts, organizations, deals,
activities, notes, tags, custom fields, settings, sync changelog entries, audit
entries, external-client readiness records, external-client permissions, and
proposed actions. Schema versioning uses SQLite `PRAGMA user_version`, currently
version `14`.

## Public Baseline Docs

Required public baselines live under `docs/`:

- [Data Model](docs/DATA_MODEL.md) documents the current schema/model state,
  migration versions, audit/sync/proposed-action readiness, and compatibility
  caveats.
- [Import and Export](docs/IMPORT_EXPORT.md) documents current CSV import/export
  behavior, mapped import flow, duplicate preflight, backup relationship, and
  unimplemented surfaces.
- [Privacy](docs/PRIVACY.md) documents offline-first behavior, local storage,
  no telemetry/cloud requirement, optional network touchpoints, MCP/AI
  non-behavior, and security caveats.
- [MCP Readiness Baseline](docs/MCP_READINESS.md) documents the current optional
  MCP boundary and future acceptance checklist.

---

## Extension Points

### Custom Fields

Custom fields are implemented. Use [Data Model](docs/DATA_MODEL.md) for the
current table names and columns. The original sketch used an EAV pair:

```sql
CREATE TABLE custom_field_definitions (
    id          TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,  -- 'contact', 'deal', 'activity'
    name        TEXT NOT NULL,
    field_type  TEXT NOT NULL,  -- 'text', 'number', 'date', 'boolean', 'select'
    options     TEXT,           -- JSON array for 'select' type
    position    INTEGER NOT NULL
);

CREATE TABLE custom_field_values (
    entity_id    TEXT NOT NULL,
    field_id     TEXT NOT NULL REFERENCES custom_field_definitions(id),
    value        TEXT,
    PRIMARY KEY (entity_id, field_id)
);
```

### Plugin SDK (v2.0+)

The plugin system is planned for v2.0. The plugin architecture follows the same two-layer model as the core:

- **Plugin backend** (Rust or WASM): registers additional Tauri commands via the plugin API
- **Plugin frontend** (JavaScript/TypeScript): adds UI panels, toolbar items, and menu entries via the Plugin Host API

Plugins are sandboxed: they can only call commands they explicitly declare in their `manifest.json`, and they cannot access the SQLite database directly — they must go through the plugin API.

See [plugins/README.md](plugins/README.md) for the current specification.

---

## MCP Readiness

MCP is an optional package boundary, not a desktop or core runtime dependency. The desktop app and `crm-core` do not include a built-in AI agent, do not start an MCP server, do not bind a localhost listener, and do not require internet access, cloud services, or model providers.

The current codebase has external-client records, per-tool permission rows, proposed actions, audit logging, Pending Actions/Audit Log UI surfaces, a disabled-by-default local MCP stdio boundary for reviewed read tools plus `create_activity_draft` pending-action creation, and one narrow core execution path for approved `create_activity_draft` proposed actions. The active permission modes are limited to `disabled`, `read_only`, and `draft_only`; broader schema-reserved modes are inactive. Approval does not run MCP/client code, and unsupported proposed-action tool/action types remain pending with explicit errors.

See [MCP Readiness Baseline](docs/MCP_READINESS.md) for the current status, non-goals, security gates, and future implementation acceptance checklist.

---

## Key Design Decisions

### Why Tauri Instead of Electron?

Electron bundles Chromium (~120 MB) into every installer. Tauri uses the system's built-in WebView, resulting in a ~3 MB binary. For users downloading over slow or metered connections — the primary audience for 900CRM — this difference is significant. Electron also uses 150–300 MB RAM at idle vs. 30–40 MB for Tauri.

### Why Not a Web App or PWA?

PWAs have unreliable offline write capabilities, especially on iOS. Service Workers do not handle complex relational data operations (joins, aggregates, full-text search) well. A native desktop app gives full filesystem access for CSV import/export, works on older systems without browser configuration, and avoids the PWA fragmentation across browsers.

### Why SQLite Instead of a Managed Database?

SQLite is zero-configuration, embedded, and produces a single portable file. There is no server to run, no connection string to configure, no port to open. For a single-user desktop app in an offline context, it is the obvious choice. It handles millions of CRM records comfortably.

### Why Custom Changelog Sync Instead of CRDTs?

CRDTs (Conflict-free Replicated Data Types) are elegant for collaborative documents but add significant complexity for structured relational data. CRM records have clear ownership semantics (one user updates one record), making last-write-wins at the field level correct in practice. The custom changelog approach gives full control with no external dependencies.

### Why Svelte 5 Instead of React or Vue?

Svelte 5 with runes has the smallest runtime of any major frontend framework. It compiles to vanilla JavaScript with no virtual DOM overhead. This directly translates to faster startup and lower memory usage on constrained hardware. The 900 Labs team also uses Svelte in 900PDF, giving consistency across the ecosystem.

### Why Rust for the Backend?

Rust provides memory safety without a garbage collector, native performance, and excellent SQLite bindings (`rusqlite`). It compiles to native code on all platforms, ensuring consistent performance regardless of OS. The Tauri ecosystem is Rust-native, making integration straightforward.

### Why No Encryption by Default?

Adding encryption (e.g., SQLCipher) requires a key management strategy. For a single-user local app, the question becomes: where does the key live? Storing it on the same disk as the data provides limited security. Proper key management (OS keychain, user-provided passphrase) adds UX complexity that is a barrier for our target users. The recommendation is to use OS-level disk encryption (BitLocker, FileVault, LUKS), which is more effective and already available. Opt-in encryption may be added in a future release.

---

For questions about the architecture, open a [discussion](https://github.com/900Labs/900CRM/discussions) or comment on the relevant issue. Architecture discussions are some of the most valuable contributions you can make.
