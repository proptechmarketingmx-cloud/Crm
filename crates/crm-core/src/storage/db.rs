//! SQLite database initialization and schema management for 900CRM.
//!
//! This module wraps a `rusqlite` connection and provides the [`Database`]
//! struct stored in [`crate::AppState`]. All other storage modules operate on
//! the `pub(crate) conn` field directly for maximum composability.
//!
//! # Schema Overview
//!
//! The complete CRM schema is created by [`Database::run_migrations`]:
//!
//! | Table                | Purpose |
//! |----------------------|---------|
//! | `contacts`           | People and legacy organization contacts |
//! | `organizations`      | First-class organization records |
//! | `deals`              | Sales opportunities |
//! | `activities`         | Tasks, calls, meetings, emails |
//! | `notes`              | Free-text notes attached to any entity |
//! | `tags`               | Taxonomy labels |
//! | `entity_tags`        | Many-to-many entity ↔ tag join |
//! | `custom_field_defs`  | User-defined field definitions |
//! | `custom_field_values`| User-defined field values per entity |
//! | `settings`           | Key-value application preferences |
//! | `sync_changelog`     | Offline-first mutation log for sync |
//! | `contacts_fts`       | FTS5 virtual table for contact search |
//! | `*_fts`              | FTS5 virtual tables for cross-entity search |
//!
//! # Migration Strategy
//!
//! [`Database::run_migrations`] is idempotent: `CREATE TABLE IF NOT EXISTS` and
//! `CREATE INDEX IF NOT EXISTS` are safe to call on every startup. Incremental
//! upgrades compare `PRAGMA user_version` to [`CURRENT_SCHEMA_VERSION`].
//!
//! # Thread Safety
//!
//! `rusqlite::Connection` is `!Send`. The `Database` struct is wrapped in a
//! `Mutex<Database>` inside `AppState`, ensuring single-threaded access.
//! WAL mode is enabled so readers never block writers.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

use crate::utils::errors::{CrmError, CrmResult};

/// The current schema version. Increment whenever a new migration is added.
const CURRENT_SCHEMA_VERSION: u32 = 15;
const DATABASE_FILENAME: &str = "900crm.db";

// ─────────────────────────────────────────────────────────────────────────────
// Database struct
// ─────────────────────────────────────────────────────────────────────────────

/// A wrapper around a `rusqlite::Connection` with a fully initialized CRM schema.
///
/// Obtain an instance via [`Database::new`]. All storage sub-modules access the
/// underlying connection through `db.conn`.
///
/// # Example
///
/// ```rust,ignore
/// use std::path::Path;
/// use crate::storage::Database;
///
/// let db = Database::new(Path::new("/tmp/crm-test")).unwrap();
/// ```
pub struct Database {
    /// The underlying SQLite connection.
    ///
    /// Exposed as `pub(crate)` so sibling storage modules can use `prepare`,
    /// `execute`, `query_row`, etc. directly.
    pub(crate) conn: Connection,

    /// Absolute or caller-provided path to the SQLite database file.
    db_path: PathBuf,
}

impl Database {
    /// Opens (or creates) the SQLite database at `<app_data_dir>/900crm.db`
    /// and runs all pending migrations.
    ///
    /// # Parameters
    ///
    /// - `app_data_dir` — Directory returned by Tauri's `app_data_dir()` API.
    ///   The database file is placed at `<app_data_dir>/900crm.db`.
    ///
    /// # Errors
    ///
    /// - [`CrmError::Io`] — `app_data_dir` could not be created.
    /// - [`CrmError::Database`] — Connection could not be opened or migrations failed.
    pub fn new(app_data_dir: &Path) -> CrmResult<Self> {
        // Ensure the application data directory exists.
        std::fs::create_dir_all(app_data_dir).map_err(|e| {
            CrmError::Io(format!(
                "Failed to create app data directory '{}': {}",
                app_data_dir.display(),
                e
            ))
        })?;

        let db_path = app_data_dir.join(DATABASE_FILENAME);
        log::info!("Opening SQLite database at {}", db_path.display());

        let conn = Connection::open(&db_path).map_err(|e| {
            CrmError::Database(format!(
                "Failed to open database at '{}': {}",
                db_path.display(),
                e
            ))
        })?;

        let mut db = Self { conn, db_path };
        db.configure()?;
        db.run_migrations()?;

        log::info!("Database initialized successfully");
        Ok(db)
    }

    pub fn database_filename() -> &'static str {
        DATABASE_FILENAME
    }

    pub fn current_schema_version() -> u32 {
        CURRENT_SCHEMA_VERSION
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn schema_version(&self) -> CrmResult<u32> {
        self.conn
            .query_row("PRAGMA user_version;", [], |row| row.get(0))
            .map_err(CrmError::from)
    }

    /// Writes a transactionally consistent SQLite snapshot to `destination`.
    ///
    /// The live database uses WAL mode, so callers should not copy the database
    /// file directly. `VACUUM INTO` asks SQLite to produce a complete standalone
    /// database image that includes committed WAL contents.
    pub fn write_snapshot(&self, destination: &Path) -> CrmResult<()> {
        if destination.exists() {
            return Err(CrmError::InvalidInput(format!(
                "Backup database '{}' already exists",
                destination.display()
            )));
        }

        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }

        self.conn.execute(
            "VACUUM main INTO ?1",
            params![destination.to_string_lossy().as_ref()],
        )?;
        Ok(())
    }

    // ─── Configuration ────────────────────────────────────────────────────────

    /// Applies performance and reliability PRAGMAs to the connection.
    ///
    /// Called once immediately after opening, before any user code runs.
    ///
    /// PRAGMAs applied:
    /// - `journal_mode = WAL` — Readers don't block writers.
    /// - `synchronous = NORMAL` — Good balance between safety and performance.
    /// - `foreign_keys = ON` — Enforce referential integrity.
    /// - `cache_size = -8192` — 8 MB page cache.
    /// - `journal_size_limit = 67108864` — Cap WAL file at 64 MB.
    fn configure(&mut self) -> CrmResult<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA journal_mode      = WAL;
            PRAGMA synchronous       = NORMAL;
            PRAGMA foreign_keys      = ON;
            PRAGMA cache_size        = -8192;
            PRAGMA journal_size_limit = 67108864;
            PRAGMA temp_store        = MEMORY;
            "#,
        )?;
        log::debug!("SQLite PRAGMAs applied");
        Ok(())
    }

    // ─── Migrations ───────────────────────────────────────────────────────────

    /// Creates all application tables, indices, and virtual tables if they do
    /// not already exist, then applies incremental schema upgrades.
    ///
    /// This function is idempotent: calling it multiple times is safe.
    ///
    /// # Errors
    ///
    /// Returns [`CrmError::Database`] if any DDL statement fails.
    pub fn run_migrations(&mut self) -> CrmResult<()> {
        let current_version: u32 = self
            .conn
            .query_row("PRAGMA user_version;", [], |row| row.get(0))
            .unwrap_or(0);

        log::debug!(
            "Database schema: current={}, target={}",
            current_version,
            CURRENT_SCHEMA_VERSION
        );

        if current_version < 1 {
            self.migrate_v1()?;
        }

        if current_version < 2 {
            self.migrate_v2_reporting_indexes()?;
        }

        if current_version < 3 {
            self.migrate_v3_notification_defaults()?;
        }

        if current_version < 4 {
            self.migrate_v4_email_defaults()?;
        }

        if current_version < 5 {
            self.migrate_v5_foundation_realignment()?;
        }

        if current_version < 6 {
            self.migrate_v6_schema_normalization_bridge()?;
        }

        if current_version < 7 {
            self.migrate_v7_deal_relationships_core_surface()?;
        }

        if current_version < 8 {
            self.migrate_v8_activity_relationships_core_surface()?;
        }

        if current_version < 9 {
            self.migrate_v9_external_client_permission_uniqueness()?;
        }

        if current_version < 10 {
            self.migrate_v10_global_search_fts_parity()?;
        }

        if current_version < 11 {
            self.migrate_v11_contact_lifecycle()?;
        }

        if current_version < 12 {
            self.migrate_v12_entity_links()?;
        }

        if current_version < 13 {
            self.migrate_v13_saved_views()?;
        }

        if current_version < 14 {
            self.migrate_v14_record_owner()?;
        }

        if current_version < 15 {
            self.migrate_v15_activity_due_time()?;
        }

        self.conn.execute_batch(&format!(
            "PRAGMA user_version = {};",
            CURRENT_SCHEMA_VERSION
        ))?;

        log::info!("Migrations complete (schema v{})", CURRENT_SCHEMA_VERSION);
        Ok(())
    }

    /// Initial schema migration — creates all tables for schema version 1.
    fn migrate_v1(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v1");

        self.conn.execute_batch(
            r#"
            -- ──────────────────────────────────────────────────────────────────
            -- contacts
            -- Stores individual people and organizations.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS contacts (
                id           TEXT PRIMARY KEY NOT NULL,
                contact_type TEXT NOT NULL DEFAULT 'person',
                first_name   TEXT NOT NULL DEFAULT '',
                last_name    TEXT NOT NULL DEFAULT '',
                org_name     TEXT NOT NULL DEFAULT '',
                email        TEXT NOT NULL DEFAULT '',
                phone        TEXT NOT NULL DEFAULT '',
                address      TEXT NOT NULL DEFAULT '',
                city         TEXT NOT NULL DEFAULT '',
                country      TEXT NOT NULL DEFAULT '',
                org_id       TEXT REFERENCES contacts(id) ON DELETE SET NULL,
                notes        TEXT NOT NULL DEFAULT '',
                created_at   TEXT NOT NULL,
                updated_at   TEXT NOT NULL,
                deleted_at   TEXT,
                device_id    TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_contacts_email
                ON contacts (email);
            CREATE INDEX IF NOT EXISTS idx_contacts_org_id
                ON contacts (org_id);
            CREATE INDEX IF NOT EXISTS idx_contacts_deleted_at
                ON contacts (deleted_at);
            CREATE INDEX IF NOT EXISTS idx_contacts_contact_type
                ON contacts (contact_type);

            -- ──────────────────────────────────────────────────────────────────
            -- deals
            -- Sales opportunities moving through the pipeline.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS deals (
                id             TEXT PRIMARY KEY NOT NULL,
                title          TEXT NOT NULL DEFAULT '',
                value          REAL NOT NULL DEFAULT 0.0,
                currency       TEXT NOT NULL DEFAULT 'USD',
                stage          TEXT NOT NULL DEFAULT 'Lead',
                probability    INTEGER NOT NULL DEFAULT 0,
                expected_close TEXT,
                contact_id     TEXT REFERENCES contacts(id) ON DELETE SET NULL,
                notes          TEXT NOT NULL DEFAULT '',
                created_at     TEXT NOT NULL,
                updated_at     TEXT NOT NULL,
                deleted_at     TEXT,
                device_id      TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_deals_stage
                ON deals (stage);
            CREATE INDEX IF NOT EXISTS idx_deals_contact_id
                ON deals (contact_id);
            CREATE INDEX IF NOT EXISTS idx_deals_deleted_at
                ON deals (deleted_at);

            -- ──────────────────────────────────────────────────────────────────
            -- activities
            -- Tasks, calls, meetings, emails attached to contacts or deals.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS activities (
                id            TEXT PRIMARY KEY NOT NULL,
                activity_type TEXT NOT NULL DEFAULT 'task',
                title         TEXT NOT NULL DEFAULT '',
                description   TEXT NOT NULL DEFAULT '',
                due_date      TEXT,
                completed     INTEGER NOT NULL DEFAULT 0,
                contact_id    TEXT REFERENCES contacts(id) ON DELETE SET NULL,
                deal_id       TEXT REFERENCES deals(id) ON DELETE SET NULL,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                deleted_at    TEXT,
                device_id     TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_activities_contact_id
                ON activities (contact_id);
            CREATE INDEX IF NOT EXISTS idx_activities_deal_id
                ON activities (deal_id);
            CREATE INDEX IF NOT EXISTS idx_activities_due_date
                ON activities (due_date);
            CREATE INDEX IF NOT EXISTS idx_activities_completed
                ON activities (completed);
            CREATE INDEX IF NOT EXISTS idx_activities_deleted_at
                ON activities (deleted_at);

            -- ──────────────────────────────────────────────────────────────────
            -- notes
            -- Freeform text notes attached to any entity.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS notes (
                id          TEXT PRIMARY KEY NOT NULL,
                content     TEXT NOT NULL DEFAULT '',
                entity_type TEXT NOT NULL,
                entity_id   TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_notes_entity
                ON notes (entity_type, entity_id);
            CREATE INDEX IF NOT EXISTS idx_notes_deleted_at
                ON notes (deleted_at);

            -- ──────────────────────────────────────────────────────────────────
            -- tags
            -- Taxonomy labels that can be applied to any entity.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS tags (
                id         TEXT PRIMARY KEY NOT NULL,
                name       TEXT NOT NULL UNIQUE,
                color      TEXT NOT NULL DEFAULT '#6366f1',
                created_at TEXT NOT NULL
            );

            -- ──────────────────────────────────────────────────────────────────
            -- entity_tags
            -- Many-to-many join: entity ↔ tag.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS entity_tags (
                entity_type TEXT NOT NULL,
                entity_id   TEXT NOT NULL,
                tag_id      TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                PRIMARY KEY (entity_type, entity_id, tag_id)
            );

            CREATE INDEX IF NOT EXISTS idx_entity_tags_entity
                ON entity_tags (entity_type, entity_id);
            CREATE INDEX IF NOT EXISTS idx_entity_tags_tag_id
                ON entity_tags (tag_id);

            -- ──────────────────────────────────────────────────────────────────
            -- custom_field_defs
            -- User-defined extra fields per entity type.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS custom_field_defs (
                id            TEXT PRIMARY KEY NOT NULL,
                entity_type   TEXT NOT NULL,
                field_name    TEXT NOT NULL,
                field_type    TEXT NOT NULL DEFAULT 'text',
                field_options TEXT,
                sort_order    INTEGER NOT NULL DEFAULT 0,
                created_at    TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_custom_field_defs_entity_type
                ON custom_field_defs (entity_type);

            -- ──────────────────────────────────────────────────────────────────
            -- custom_field_values
            -- Values for user-defined fields, per entity instance.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS custom_field_values (
                id           TEXT PRIMARY KEY NOT NULL,
                field_def_id TEXT NOT NULL REFERENCES custom_field_defs(id) ON DELETE CASCADE,
                entity_id    TEXT NOT NULL,
                value        TEXT NOT NULL DEFAULT '',
                created_at   TEXT NOT NULL,
                updated_at   TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_custom_field_values_entity
                ON custom_field_values (entity_id);
            CREATE INDEX IF NOT EXISTS idx_custom_field_values_field_def
                ON custom_field_values (field_def_id);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_custom_field_values_field_entity
                ON custom_field_values (field_def_id, entity_id);

            -- ──────────────────────────────────────────────────────────────────
            -- settings
            -- Application-wide key-value preferences store.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL DEFAULT ''
            );

            -- Seed default settings. INSERT OR IGNORE preserves existing values.
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('language',     'en',         '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('currency',     'USD',        '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('theme',        'light',      '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('date_format',  'YYYY-MM-DD', '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('sync_enabled', 'false',      '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('sync_url',     '',           '');

            -- ──────────────────────────────────────────────────────────────────
            -- sync_changelog
            -- Append-only log of every mutation for offline-first sync.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS sync_changelog (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                entity_id   TEXT NOT NULL,
                field_name  TEXT NOT NULL,
                old_value   TEXT,
                new_value   TEXT,
                timestamp   TEXT NOT NULL,
                device_id   TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_sync_changelog_timestamp
                ON sync_changelog (timestamp);
            CREATE INDEX IF NOT EXISTS idx_sync_changelog_entity
                ON sync_changelog (entity_type, entity_id);
            CREATE INDEX IF NOT EXISTS idx_sync_changelog_device_id
                ON sync_changelog (device_id);

            -- ──────────────────────────────────────────────────────────────────
            -- contacts_fts
            -- FTS5 virtual table for full-text search over contacts.
            -- ──────────────────────────────────────────────────────────────────
            CREATE VIRTUAL TABLE IF NOT EXISTS contacts_fts USING fts5(
                first_name,
                last_name,
                org_name,
                email,
                phone,
                content='contacts',
                content_rowid='rowid'
            );
            "#,
        )?;

        log::info!("Migration v1 complete");
        Ok(())
    }

    /// Schema v2 migration - reporting/perf indexes for analytics endpoints.
    ///
    /// Adds composite and time-oriented indexes used by reporting queries to
    /// keep aggregation paths responsive on low-resource hardware.
    fn migrate_v2_reporting_indexes(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v2 reporting indexes");

        self.conn.execute_batch(
            r#"
            CREATE INDEX IF NOT EXISTS idx_deals_created_at
                ON deals (created_at);
            CREATE INDEX IF NOT EXISTS idx_deals_deleted_stage
                ON deals (deleted_at, stage);

            CREATE INDEX IF NOT EXISTS idx_activities_created_at
                ON activities (created_at);
            CREATE INDEX IF NOT EXISTS idx_activities_type
                ON activities (activity_type);
            CREATE INDEX IF NOT EXISTS idx_activities_completed_due
                ON activities (completed, due_date);
            "#,
        )?;

        log::info!("Migration v2 reporting indexes complete");
        Ok(())
    }

    /// Schema v3 migration - notification/reminder settings defaults.
    ///
    /// Ensures existing installations receive the desktop-reminder setting
    /// keys without requiring a full schema rebuild.
    fn migrate_v3_notification_defaults(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v3 notification defaults");

        self.conn.execute_batch(
            r#"
            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('notifications_enabled', 'true', '');

            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('reminder_lead_minutes', '30', '');
            "#,
        )?;

        log::info!("Migration v3 notification defaults complete");
        Ok(())
    }

    /// Schema v4 migration - optional email integration defaults.
    ///
    /// Seeds IMAP/SMTP configuration keys for existing installs.
    fn migrate_v4_email_defaults(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v4 email defaults");

        self.conn.execute_batch(
            r#"
            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('email_integration_enabled', 'false', '');

            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('smtp_host', '', '');
            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('smtp_port', '587', '');
            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('smtp_username', '', '');
            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('smtp_password', '', '');
            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('smtp_from', '', '');

            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('imap_host', '', '');
            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('imap_port', '993', '');
            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('imap_username', '', '');
            INSERT OR IGNORE INTO settings (key, value, updated_at)
            VALUES ('imap_password', '', '');
            "#,
        )?;

        log::info!("Migration v4 email defaults complete");
        Ok(())
    }

    /// Foundation realignment schema.
    ///
    /// Schema v5 migration - foundation realignment.
    ///
    /// This migration starts the target build-spec schema without rewriting the
    /// existing v1 contact/deal/activity tables. Follow-up migrations can move
    /// data into the final normalized shape after the desktop UI is updated.
    fn migrate_v5_foundation_realignment(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v5 foundation realignment");

        self.conn.execute_batch(
            r#"
            -- ──────────────────────────────────────────────────────────────────
            -- organizations
            -- New normalized organization table from the build spec.
            -- ──────────────────────────────────────────────────────────────────
            CREATE TABLE IF NOT EXISTS organizations (
                id            TEXT PRIMARY KEY,
                name          TEXT NOT NULL,
                email         TEXT,
                phone         TEXT,
                website       TEXT,
                address_line1 TEXT,
                address_line2 TEXT,
                city          TEXT,
                region        TEXT,
                country       TEXT,
                postal_code   TEXT,
                source        TEXT,
                description   TEXT,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                deleted_at    TEXT,
                device_id     TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_organizations_name
                ON organizations (name);
            CREATE INDEX IF NOT EXISTS idx_organizations_deleted_at
                ON organizations (deleted_at);

            -- Join table matching the target tag_links name. The existing v1
            -- entity_tags table remains for compatibility during transition.
            CREATE TABLE IF NOT EXISTS tag_links (
                id          TEXT PRIMARY KEY,
                tag_id      TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id   TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL,
                FOREIGN KEY (tag_id) REFERENCES tags(id)
            );

            CREATE INDEX IF NOT EXISTS idx_tag_links_entity
                ON tag_links (entity_type, entity_id);
            CREATE INDEX IF NOT EXISTS idx_tag_links_tag_id
                ON tag_links (tag_id);

            -- User-visible accountability log. Separate from sync_changelog.
            CREATE TABLE IF NOT EXISTS audit_log (
                id          TEXT PRIMARY KEY,
                actor_type  TEXT NOT NULL,
                actor_id    TEXT,
                action      TEXT NOT NULL,
                entity_type TEXT,
                entity_id   TEXT,
                before_json TEXT,
                after_json  TEXT,
                created_at  TEXT NOT NULL,
                device_id   TEXT NOT NULL,
                CHECK (actor_type IN ('user', 'desktop_app', 'mcp_client', 'import', 'system'))
            );

            CREATE INDEX IF NOT EXISTS idx_audit_log_created_at
                ON audit_log (created_at);
            CREATE INDEX IF NOT EXISTS idx_audit_log_entity
                ON audit_log (entity_type, entity_id);

            -- Optional external clients, including future MCP servers.
            CREATE TABLE IF NOT EXISTS external_clients (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                client_type     TEXT NOT NULL,
                permission_mode TEXT NOT NULL DEFAULT 'disabled',
                enabled         INTEGER NOT NULL DEFAULT 0,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL,
                deleted_at      TEXT,
                device_id       TEXT NOT NULL,
                CHECK (permission_mode IN (
                    'disabled',
                    'read_only',
                    'draft_only',
                    'write_with_confirmation',
                    'write_allowed'
                ))
            );

            CREATE INDEX IF NOT EXISTS idx_external_clients_enabled
                ON external_clients (enabled);

            CREATE TABLE IF NOT EXISTS external_client_permissions (
                id                    TEXT PRIMARY KEY,
                client_id             TEXT NOT NULL,
                tool_name             TEXT NOT NULL,
                can_read              INTEGER NOT NULL DEFAULT 0,
                can_write             INTEGER NOT NULL DEFAULT 0,
                requires_confirmation INTEGER NOT NULL DEFAULT 1,
                created_at            TEXT NOT NULL,
                updated_at            TEXT NOT NULL,
                FOREIGN KEY (client_id) REFERENCES external_clients(id)
            );

            CREATE INDEX IF NOT EXISTS idx_external_client_permissions_client
                ON external_client_permissions (client_id);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_external_client_permissions_client_tool
                ON external_client_permissions (client_id, tool_name);

            CREATE TABLE IF NOT EXISTS proposed_actions (
                id                   TEXT PRIMARY KEY,
                client_id            TEXT,
                action_type          TEXT NOT NULL,
                tool_name            TEXT NOT NULL,
                entity_type          TEXT,
                entity_id            TEXT,
                input_json           TEXT NOT NULL,
                proposed_output_json TEXT,
                status               TEXT NOT NULL DEFAULT 'pending',
                created_at           TEXT NOT NULL,
                approved_at          TEXT,
                rejected_at          TEXT,
                executed_at          TEXT,
                device_id            TEXT NOT NULL,
                FOREIGN KEY (client_id) REFERENCES external_clients(id),
                CHECK (status IN (
                    'pending',
                    'approved',
                    'rejected',
                    'executed',
                    'failed',
                    'cancelled'
                ))
            );

            CREATE INDEX IF NOT EXISTS idx_proposed_actions_status
                ON proposed_actions (status);
            CREATE INDEX IF NOT EXISTS idx_proposed_actions_entity
                ON proposed_actions (entity_type, entity_id);

            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('mcp_status',          'not_installed', '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('mcp_enabled',         'false',         '');
            INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES ('mcp_permission_mode', 'disabled',      '');
            "#,
        )?;

        self.add_column_if_missing("contacts", "organization_id", "organization_id TEXT")?;
        self.add_column_if_missing("contacts", "title", "title TEXT")?;
        self.add_column_if_missing("contacts", "whatsapp", "whatsapp TEXT")?;
        self.add_column_if_missing("contacts", "address_line1", "address_line1 TEXT")?;
        self.add_column_if_missing("contacts", "address_line2", "address_line2 TEXT")?;
        self.add_column_if_missing("contacts", "region", "region TEXT")?;
        self.add_column_if_missing("contacts", "postal_code", "postal_code TEXT")?;
        self.add_column_if_missing("contacts", "source", "source TEXT")?;
        self.add_column_if_missing("contacts", "description", "description TEXT")?;

        self.add_column_if_missing("notes", "body", "body TEXT")?;
        self.add_column_if_missing("tags", "updated_at", "updated_at TEXT")?;
        self.add_column_if_missing("tags", "deleted_at", "deleted_at TEXT")?;
        self.add_column_if_missing("tags", "device_id", "device_id TEXT NOT NULL DEFAULT ''")?;

        self.add_column_if_missing(
            "sync_changelog",
            "operation",
            "operation TEXT NOT NULL DEFAULT 'update'",
        )?;
        self.add_column_if_missing("sync_changelog", "synced_at", "synced_at TEXT")?;

        self.conn.execute_batch(
            r#"
            CREATE INDEX IF NOT EXISTS idx_contacts_organization_id
                ON contacts (organization_id);
            "#,
        )?;

        let sync_changelog_exists: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'sync_changelog'",
            [],
            |row| row.get(0),
        )?;
        if sync_changelog_exists != 0 {
            self.conn.execute_batch(
                r#"
                CREATE INDEX IF NOT EXISTS idx_sync_changelog_synced_at
                    ON sync_changelog (synced_at);
                "#,
            )?;
        }

        log::info!("Migration v5 foundation realignment complete");
        Ok(())
    }

    /// Schema v6 migration - bridges legacy organization-as-contact data into the normalized table.
    ///
    /// This is intentionally non-destructive: old `contacts` rows stay in
    /// place, and `INSERT OR IGNORE` avoids overwriting organization records
    /// users may already have edited in the normalized table.
    fn migrate_v6_schema_normalization_bridge(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v6 organization bridge");

        self.add_column_if_missing("contacts", "organization_id", "organization_id TEXT")?;

        self.conn.execute_batch(
            r#"
            INSERT OR IGNORE INTO organizations
                (id, name, email, phone, website, address_line1, address_line2,
                 city, region, country, postal_code, source, description,
                 created_at, updated_at, deleted_at, device_id)
            SELECT
                id,
                COALESCE(
                    NULLIF(TRIM(org_name), ''),
                    NULLIF(TRIM(first_name || ' ' || last_name), ''),
                    NULLIF(TRIM(email), ''),
                    id
                ) AS name,
                NULLIF(TRIM(email), ''),
                NULLIF(TRIM(phone), ''),
                NULL,
                NULLIF(TRIM(address), ''),
                NULL,
                NULLIF(TRIM(city), ''),
                NULL,
                NULLIF(TRIM(country), ''),
                NULL,
                'legacy_contact',
                NULLIF(TRIM(notes), ''),
                created_at,
                updated_at,
                deleted_at,
                COALESCE(NULLIF(TRIM(device_id), ''), 'migration')
            FROM contacts
            WHERE contact_type = 'organization';

            UPDATE contacts
            SET organization_id = org_id
            WHERE (organization_id IS NULL OR organization_id = '')
              AND org_id IS NOT NULL
              AND EXISTS (
                  SELECT 1 FROM organizations WHERE organizations.id = contacts.org_id
              );

            UPDATE contacts
            SET organization_id = id
            WHERE contact_type = 'organization'
              AND (organization_id IS NULL OR organization_id = '')
              AND EXISTS (
                  SELECT 1 FROM organizations WHERE organizations.id = contacts.id
              );
            "#,
        )?;

        log::info!("Migration v6 organization bridge complete");
        Ok(())
    }

    /// Schema v7 migration - additive deal relationship foundation.
    ///
    /// This keeps the legacy `deals.contact_id` primary-contact mirror while
    /// adding first-class deal-to-organization and deal-to-contact surfaces.
    fn migrate_v7_deal_relationships_core_surface(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v7 deal relationships");

        if !self.table_exists("deals")? {
            log::warn!(
                "Skipping deal relationship migration because legacy table 'deals' is missing"
            );
            return Ok(());
        }

        self.add_column_if_missing("deals", "organization_id", "organization_id TEXT")?;

        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS deal_contacts (
                id         TEXT PRIMARY KEY,
                deal_id    TEXT NOT NULL,
                contact_id TEXT NOT NULL,
                role       TEXT,
                is_primary INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                deleted_at TEXT,
                device_id  TEXT NOT NULL,
                FOREIGN KEY (deal_id) REFERENCES deals(id),
                FOREIGN KEY (contact_id) REFERENCES contacts(id),
                CHECK (is_primary IN (0, 1))
            );

            CREATE INDEX IF NOT EXISTS idx_deals_organization_id
                ON deals (organization_id);
            CREATE INDEX IF NOT EXISTS idx_deal_contacts_deal_id
                ON deal_contacts (deal_id);
            CREATE INDEX IF NOT EXISTS idx_deal_contacts_contact_id
                ON deal_contacts (contact_id);
            CREATE INDEX IF NOT EXISTS idx_deal_contacts_active_deal
                ON deal_contacts (deal_id)
                WHERE deleted_at IS NULL;
            CREATE INDEX IF NOT EXISTS idx_deal_contacts_active_contact
                ON deal_contacts (contact_id)
                WHERE deleted_at IS NULL;
            CREATE INDEX IF NOT EXISTS idx_deal_contacts_active_primary
                ON deal_contacts (deal_id, contact_id)
                WHERE deleted_at IS NULL AND is_primary = 1;
            "#,
        )?;

        if self.table_exists("contacts")? {
            self.conn.execute_batch(
                r#"

            INSERT INTO deal_contacts
                (id, deal_id, contact_id, role, is_primary, created_at, deleted_at, device_id)
            SELECT
                'legacy-primary:' || d.id || ':' || d.contact_id,
                d.id,
                d.contact_id,
                NULL,
                1,
                COALESCE(NULLIF(TRIM(d.created_at), ''), datetime('now')),
                NULL,
                COALESCE(NULLIF(TRIM(d.device_id), ''), 'migration')
            FROM deals d
            WHERE d.deleted_at IS NULL
              AND TRIM(COALESCE(d.contact_id, '')) <> ''
              AND EXISTS (
                  SELECT 1
                  FROM contacts c
                  WHERE c.id = d.contact_id
                    AND c.deleted_at IS NULL
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM deal_contacts dc
                  WHERE dc.deal_id = d.id
                    AND dc.contact_id = d.contact_id
                    AND dc.deleted_at IS NULL
              );

            "#,
            )?;

            if self.table_exists("organizations")? {
                self.conn.execute_batch(
                    r#"
                UPDATE deals
                SET organization_id = (
                    SELECT c.organization_id
                    FROM contacts c
                    JOIN organizations o
                      ON o.id = c.organization_id
                     AND o.deleted_at IS NULL
                    WHERE c.id = deals.contact_id
                      AND c.deleted_at IS NULL
                      AND TRIM(COALESCE(c.organization_id, '')) <> ''
                    LIMIT 1
                )
                WHERE deleted_at IS NULL
                  AND TRIM(COALESCE(organization_id, '')) = ''
                  AND TRIM(COALESCE(contact_id, '')) <> ''
                  AND EXISTS (
                      SELECT 1
                      FROM contacts c
                      JOIN organizations o
                        ON o.id = c.organization_id
                       AND o.deleted_at IS NULL
                      WHERE c.id = deals.contact_id
                        AND c.deleted_at IS NULL
                        AND TRIM(COALESCE(c.organization_id, '')) <> ''
                  );
                "#,
                )?;
            } else {
                log::warn!(
                    "Skipping deal organization backfill because table 'organizations' is missing"
                );
            }
        } else {
            log::warn!(
                "Skipping deal relationship backfill because legacy table 'contacts' is missing"
            );
        }

        log::info!("Migration v7 deal relationships complete");
        Ok(())
    }

    /// Schema v8 migration - additive activity relationship foundation.
    ///
    /// This keeps `activities.contact_id` and `activities.deal_id` as legacy
    /// compatibility mirrors while adding a first-class activity link table.
    fn migrate_v8_activity_relationships_core_surface(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v8 activity relationships");

        if !self.table_exists("activities")? {
            log::warn!(
                "Skipping activity relationship migration because table 'activities' is missing"
            );
            return Ok(());
        }

        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS activity_links (
                id          TEXT PRIMARY KEY NOT NULL,
                activity_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id   TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL,
                FOREIGN KEY (activity_id) REFERENCES activities(id),
                CHECK (entity_type IN ('contact', 'organization', 'deal'))
            );

            CREATE INDEX IF NOT EXISTS idx_activity_links_activity_id
                ON activity_links (activity_id);
            CREATE INDEX IF NOT EXISTS idx_activity_links_entity
                ON activity_links (entity_type, entity_id);
            CREATE INDEX IF NOT EXISTS idx_activity_links_active_activity
                ON activity_links (activity_id)
                WHERE deleted_at IS NULL;
            CREATE INDEX IF NOT EXISTS idx_activity_links_active_entity
                ON activity_links (entity_type, entity_id)
                WHERE deleted_at IS NULL;
            CREATE UNIQUE INDEX IF NOT EXISTS idx_activity_links_active_unique
                ON activity_links (activity_id, entity_type, entity_id)
                WHERE deleted_at IS NULL;
            "#,
        )?;

        if self.table_exists("contacts")? {
            self.conn.execute_batch(
                r#"
            INSERT INTO activity_links
                (id, activity_id, entity_type, entity_id, created_at, deleted_at, device_id)
            SELECT
                'legacy-contact:' || a.id || ':' || a.contact_id,
                a.id,
                'contact',
                a.contact_id,
                COALESCE(NULLIF(TRIM(a.created_at), ''), datetime('now')),
                NULL,
                COALESCE(NULLIF(TRIM(a.device_id), ''), 'migration')
            FROM activities a
            WHERE a.deleted_at IS NULL
              AND TRIM(COALESCE(a.contact_id, '')) <> ''
              AND EXISTS (
                  SELECT 1
                  FROM contacts c
                  WHERE c.id = a.contact_id
                    AND c.deleted_at IS NULL
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM activity_links al
                  WHERE al.activity_id = a.id
                    AND al.entity_type = 'contact'
                    AND al.entity_id = a.contact_id
                    AND al.deleted_at IS NULL
              );
            "#,
            )?;
        }

        if self.table_exists("deals")? {
            self.conn.execute_batch(
                r#"
            INSERT INTO activity_links
                (id, activity_id, entity_type, entity_id, created_at, deleted_at, device_id)
            SELECT
                'legacy-deal:' || a.id || ':' || a.deal_id,
                a.id,
                'deal',
                a.deal_id,
                COALESCE(NULLIF(TRIM(a.created_at), ''), datetime('now')),
                NULL,
                COALESCE(NULLIF(TRIM(a.device_id), ''), 'migration')
            FROM activities a
            WHERE a.deleted_at IS NULL
              AND TRIM(COALESCE(a.deal_id, '')) <> ''
              AND EXISTS (
                  SELECT 1
                  FROM deals d
                  WHERE d.id = a.deal_id
                    AND d.deleted_at IS NULL
              )
              AND NOT EXISTS (
                  SELECT 1
                  FROM activity_links al
                  WHERE al.activity_id = a.id
                    AND al.entity_type = 'deal'
                    AND al.entity_id = a.deal_id
                    AND al.deleted_at IS NULL
              );
            "#,
            )?;
        }

        log::info!("Migration v8 activity relationships complete");
        Ok(())
    }

    fn migrate_v9_external_client_permission_uniqueness(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v9 external client permission uniqueness");

        if !self.table_exists("external_client_permissions")? {
            log::warn!(
                "Skipping external client permission uniqueness migration because table 'external_client_permissions' is missing"
            );
            return Ok(());
        }

        self.conn.execute_batch(
            r#"
            DELETE FROM external_client_permissions
            WHERE id NOT IN (
                SELECT id
                FROM (
                    SELECT
                        id,
                        ROW_NUMBER() OVER (
                            PARTITION BY client_id, tool_name
                            ORDER BY updated_at DESC, created_at DESC, id DESC
                        ) AS row_rank
                    FROM external_client_permissions
                )
                WHERE row_rank = 1
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_external_client_permissions_client_tool
                ON external_client_permissions (client_id, tool_name);
            "#,
        )?;

        log::info!("Migration v9 external client permission uniqueness complete");
        Ok(())
    }

    fn migrate_v10_global_search_fts_parity(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v10 global search FTS parity");

        if self.table_has_columns(
            "organizations",
            &[
                "name",
                "email",
                "phone",
                "website",
                "description",
                "city",
                "region",
                "country",
                "source",
                "deleted_at",
            ],
        )? {
            self.conn.execute_batch(
                r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS organizations_fts USING fts5(
                name,
                email,
                phone,
                website,
                description,
                city,
                region,
                country,
                source,
                content='organizations',
                content_rowid='rowid'
            );

            INSERT INTO organizations_fts(organizations_fts) VALUES('delete-all');
            INSERT INTO organizations_fts(rowid, name, email, phone, website, description, city, region, country, source)
            SELECT rowid, name, email, phone, website, description, city, region, country, source
            FROM organizations
            WHERE deleted_at IS NULL;

            CREATE TRIGGER IF NOT EXISTS organizations_fts_ai
            AFTER INSERT ON organizations
            BEGIN
                INSERT INTO organizations_fts(rowid, name, email, phone, website, description, city, region, country, source)
                SELECT new.rowid, new.name, new.email, new.phone, new.website, new.description, new.city, new.region, new.country, new.source
                WHERE new.deleted_at IS NULL;
            END;

            CREATE TRIGGER IF NOT EXISTS organizations_fts_au
            AFTER UPDATE ON organizations
            BEGIN
                INSERT INTO organizations_fts(organizations_fts, rowid, name, email, phone, website, description, city, region, country, source)
                SELECT 'delete', old.rowid, old.name, old.email, old.phone, old.website, old.description, old.city, old.region, old.country, old.source
                WHERE old.deleted_at IS NULL;
                INSERT INTO organizations_fts(rowid, name, email, phone, website, description, city, region, country, source)
                SELECT new.rowid, new.name, new.email, new.phone, new.website, new.description, new.city, new.region, new.country, new.source
                WHERE new.deleted_at IS NULL;
            END;

            CREATE TRIGGER IF NOT EXISTS organizations_fts_ad
            AFTER DELETE ON organizations
            BEGIN
                INSERT INTO organizations_fts(organizations_fts, rowid, name, email, phone, website, description, city, region, country, source)
                SELECT 'delete', old.rowid, old.name, old.email, old.phone, old.website, old.description, old.city, old.region, old.country, old.source
                WHERE old.deleted_at IS NULL;
            END;
            "#,
            )?;
        } else {
            log::warn!(
                "Skipping organizations FTS migration because required source columns are missing"
            );
        }

        if self.table_has_columns(
            "deals",
            &["title", "stage", "notes", "currency", "deleted_at"],
        )? {
            self.conn.execute_batch(
                r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS deals_fts USING fts5(
                title,
                stage,
                notes,
                currency,
                content='deals',
                content_rowid='rowid'
            );

            INSERT INTO deals_fts(deals_fts) VALUES('delete-all');
            INSERT INTO deals_fts(rowid, title, stage, notes, currency)
            SELECT rowid, title, stage, notes, currency
            FROM deals
            WHERE deleted_at IS NULL;

            CREATE TRIGGER IF NOT EXISTS deals_fts_ai
            AFTER INSERT ON deals
            BEGIN
                INSERT INTO deals_fts(rowid, title, stage, notes, currency)
                SELECT new.rowid, new.title, new.stage, new.notes, new.currency
                WHERE new.deleted_at IS NULL;
            END;

            CREATE TRIGGER IF NOT EXISTS deals_fts_au
            AFTER UPDATE ON deals
            BEGIN
                INSERT INTO deals_fts(deals_fts, rowid, title, stage, notes, currency)
                SELECT 'delete', old.rowid, old.title, old.stage, old.notes, old.currency
                WHERE old.deleted_at IS NULL;
                INSERT INTO deals_fts(rowid, title, stage, notes, currency)
                SELECT new.rowid, new.title, new.stage, new.notes, new.currency
                WHERE new.deleted_at IS NULL;
            END;

            CREATE TRIGGER IF NOT EXISTS deals_fts_ad
            AFTER DELETE ON deals
            BEGIN
                INSERT INTO deals_fts(deals_fts, rowid, title, stage, notes, currency)
                SELECT 'delete', old.rowid, old.title, old.stage, old.notes, old.currency
                WHERE old.deleted_at IS NULL;
            END;
            "#,
            )?;
        } else {
            log::warn!("Skipping deals FTS migration because required source columns are missing");
        }

        if self.table_has_columns(
            "activities",
            &["title", "activity_type", "description", "deleted_at"],
        )? {
            self.conn.execute_batch(
                r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS activities_fts USING fts5(
                title,
                activity_type,
                description,
                content='activities',
                content_rowid='rowid'
            );

            INSERT INTO activities_fts(activities_fts) VALUES('delete-all');
            INSERT INTO activities_fts(rowid, title, activity_type, description)
            SELECT rowid, title, activity_type, description
            FROM activities
            WHERE deleted_at IS NULL;

            CREATE TRIGGER IF NOT EXISTS activities_fts_ai
            AFTER INSERT ON activities
            BEGIN
                INSERT INTO activities_fts(rowid, title, activity_type, description)
                SELECT new.rowid, new.title, new.activity_type, new.description
                WHERE new.deleted_at IS NULL;
            END;

            CREATE TRIGGER IF NOT EXISTS activities_fts_au
            AFTER UPDATE ON activities
            BEGIN
                INSERT INTO activities_fts(activities_fts, rowid, title, activity_type, description)
                SELECT 'delete', old.rowid, old.title, old.activity_type, old.description
                WHERE old.deleted_at IS NULL;
                INSERT INTO activities_fts(rowid, title, activity_type, description)
                SELECT new.rowid, new.title, new.activity_type, new.description
                WHERE new.deleted_at IS NULL;
            END;

            CREATE TRIGGER IF NOT EXISTS activities_fts_ad
            AFTER DELETE ON activities
            BEGIN
                INSERT INTO activities_fts(activities_fts, rowid, title, activity_type, description)
                SELECT 'delete', old.rowid, old.title, old.activity_type, old.description
                WHERE old.deleted_at IS NULL;
            END;
            "#,
            )?;
        } else {
            log::warn!(
                "Skipping activities FTS migration because required source columns are missing"
            );
        }

        if self.table_has_columns("notes", &["content", "body", "entity_type", "deleted_at"])? {
            self.conn.execute_batch(
                r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
                content,
                body,
                entity_type,
                content='notes',
                content_rowid='rowid'
            );

            INSERT INTO notes_fts(notes_fts) VALUES('delete-all');
            INSERT INTO notes_fts(rowid, content, body, entity_type)
            SELECT rowid, content, body, entity_type
            FROM notes
            WHERE deleted_at IS NULL;

            CREATE TRIGGER IF NOT EXISTS notes_fts_ai
            AFTER INSERT ON notes
            BEGIN
                INSERT INTO notes_fts(rowid, content, body, entity_type)
                SELECT new.rowid, new.content, new.body, new.entity_type
                WHERE new.deleted_at IS NULL;
            END;

            CREATE TRIGGER IF NOT EXISTS notes_fts_au
            AFTER UPDATE ON notes
            BEGIN
                INSERT INTO notes_fts(notes_fts, rowid, content, body, entity_type)
                SELECT 'delete', old.rowid, old.content, old.body, old.entity_type
                WHERE old.deleted_at IS NULL;
                INSERT INTO notes_fts(rowid, content, body, entity_type)
                SELECT new.rowid, new.content, new.body, new.entity_type
                WHERE new.deleted_at IS NULL;
            END;

            CREATE TRIGGER IF NOT EXISTS notes_fts_ad
            AFTER DELETE ON notes
            BEGIN
                INSERT INTO notes_fts(notes_fts, rowid, content, body, entity_type)
                SELECT 'delete', old.rowid, old.content, old.body, old.entity_type
                WHERE old.deleted_at IS NULL;
            END;
            "#,
            )?;
        } else {
            log::warn!("Skipping notes FTS migration because required source columns are missing");
        }

        if self.table_has_columns("tags", &["name", "color", "deleted_at"])? {
            self.conn.execute_batch(
                r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS tags_fts USING fts5(
                name,
                color,
                content='tags',
                content_rowid='rowid'
            );

            INSERT INTO tags_fts(tags_fts) VALUES('delete-all');
            INSERT INTO tags_fts(rowid, name, color)
            SELECT rowid, name, color
            FROM tags
            WHERE deleted_at IS NULL;

            CREATE TRIGGER IF NOT EXISTS tags_fts_ai
            AFTER INSERT ON tags
            BEGIN
                INSERT INTO tags_fts(rowid, name, color)
                SELECT new.rowid, new.name, new.color
                WHERE new.deleted_at IS NULL;
            END;

            CREATE TRIGGER IF NOT EXISTS tags_fts_au
            AFTER UPDATE ON tags
            BEGIN
                INSERT INTO tags_fts(tags_fts, rowid, name, color)
                SELECT 'delete', old.rowid, old.name, old.color
                WHERE old.deleted_at IS NULL;
                INSERT INTO tags_fts(rowid, name, color)
                SELECT new.rowid, new.name, new.color
                WHERE new.deleted_at IS NULL;
            END;

            CREATE TRIGGER IF NOT EXISTS tags_fts_ad
            AFTER DELETE ON tags
            BEGIN
                INSERT INTO tags_fts(tags_fts, rowid, name, color)
                SELECT 'delete', old.rowid, old.name, old.color
                WHERE old.deleted_at IS NULL;
            END;
            "#,
            )?;
        } else {
            log::warn!("Skipping tags FTS migration because required source columns are missing");
        }

        log::info!("Migration v10 global search FTS parity complete");
        Ok(())
    }

    fn migrate_v11_contact_lifecycle(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v11 contact lifecycle");

        self.add_column_if_missing(
            "contacts",
            "lifecycle",
            "lifecycle TEXT NOT NULL DEFAULT 'customer'",
        )?;

        if self.table_exists("contacts")? {
            self.conn.execute_batch(
                r#"
                UPDATE contacts
                SET lifecycle = 'customer'
                WHERE TRIM(COALESCE(lifecycle, '')) = ''
                   OR lifecycle NOT IN ('lead', 'customer');

                CREATE INDEX IF NOT EXISTS idx_contacts_lifecycle
                    ON contacts (lifecycle)
                    WHERE deleted_at IS NULL;
                "#,
            )?;
        }

        log::info!("Migration v11 contact lifecycle complete");
        Ok(())
    }

    fn migrate_v12_entity_links(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v12 entity links");

        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS entity_links (
                id           TEXT PRIMARY KEY NOT NULL,
                entity_type  TEXT NOT NULL,
                entity_id    TEXT NOT NULL,
                title        TEXT NOT NULL DEFAULT '',
                kind         TEXT NOT NULL,
                target       TEXT NOT NULL,
                created_at   TEXT NOT NULL,
                updated_at   TEXT NOT NULL,
                deleted_at   TEXT,
                device_id    TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_entity_links_parent
                ON entity_links (entity_type, entity_id)
                WHERE deleted_at IS NULL;
            "#,
        )?;

        log::info!("Migration v12 entity links complete");
        Ok(())
    }

    fn migrate_v13_saved_views(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v13 saved views");

        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS saved_views (
                id           TEXT PRIMARY KEY NOT NULL,
                entity_type  TEXT NOT NULL,
                name         TEXT NOT NULL,
                filters_json TEXT NOT NULL,
                created_at   TEXT NOT NULL,
                updated_at   TEXT NOT NULL,
                deleted_at   TEXT,
                device_id    TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_saved_views_entity
                ON saved_views (entity_type, name)
                WHERE deleted_at IS NULL;
            "#,
        )?;

        log::info!("Migration v13 saved views complete");
        Ok(())
    }

    fn migrate_v14_record_owner(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v14 record owner");

        self.add_column_if_missing("contacts", "owner", "owner TEXT")?;
        self.add_column_if_missing("deals", "owner", "owner TEXT")?;
        self.add_column_if_missing("organizations", "owner", "owner TEXT")?;

        if self.table_exists("contacts")? {
            self.conn.execute_batch(
                r#"
                CREATE INDEX IF NOT EXISTS idx_contacts_owner
                    ON contacts (owner)
                    WHERE deleted_at IS NULL AND owner IS NOT NULL AND TRIM(owner) <> '';
                "#,
            )?;
        }

        if self.table_exists("deals")? {
            self.conn.execute_batch(
                r#"
                CREATE INDEX IF NOT EXISTS idx_deals_owner
                    ON deals (owner)
                    WHERE deleted_at IS NULL AND owner IS NOT NULL AND TRIM(owner) <> '';
                "#,
            )?;
        }

        if self.table_exists("organizations")? {
            self.conn.execute_batch(
                r#"
                CREATE INDEX IF NOT EXISTS idx_organizations_owner
                    ON organizations (owner)
                    WHERE deleted_at IS NULL AND owner IS NOT NULL AND TRIM(owner) <> '';
                "#,
            )?;
        }

        if self.table_exists("settings")? {
            self.conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO settings (key, value, updated_at)
                VALUES ('default_owner', '', '');
                "#,
            )?;
        }

        log::info!("Migration v14 record owner complete");
        Ok(())
    }

    fn migrate_v15_activity_due_time(&mut self) -> CrmResult<()> {
        log::info!("Running database migration v15 activity due time");

        if self.table_exists("settings")? {
            self.conn.execute_batch(
                r#"
                INSERT OR IGNORE INTO settings (key, value, updated_at)
                VALUES ('activity_due_time', '1', '');
                "#,
            )?;
        }

        log::info!("Migration v15 activity due time complete");
        Ok(())
    }

    fn table_exists(&self, table_name: &str) -> CrmResult<bool> {
        let table_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            params![table_name],
            |row| row.get(0),
        )?;

        Ok(table_count != 0)
    }

    fn table_has_columns(&self, table_name: &str, required_columns: &[&str]) -> CrmResult<bool> {
        if !self.table_exists(table_name)? {
            return Ok(false);
        }

        let mut stmt = self
            .conn
            .prepare(&format!("PRAGMA table_info({})", table_name))?;
        let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
        let mut existing_columns = Vec::new();

        for column in columns {
            existing_columns.push(column?);
        }

        Ok(required_columns
            .iter()
            .all(|required| existing_columns.iter().any(|column| column == required)))
    }

    fn add_column_if_missing(
        &mut self,
        table_name: &str,
        column_name: &str,
        column_definition: &str,
    ) -> CrmResult<()> {
        if !self.table_exists(table_name)? {
            log::warn!(
                "Skipping column migration for missing legacy table '{}'",
                table_name
            );
            return Ok(());
        }

        let exists = {
            let mut stmt = self
                .conn
                .prepare(&format!("PRAGMA table_info({})", table_name))?;
            let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;

            let mut exists = false;
            for column in columns {
                if column? == column_name {
                    exists = true;
                    break;
                }
            }
            exists
        };

        if exists {
            return Ok(());
        }

        self.conn.execute_batch(&format!(
            "ALTER TABLE {} ADD COLUMN {};",
            table_name, column_definition
        ))?;
        Ok(())
    }

    /// Returns a reference to the underlying `rusqlite::Connection`.
    ///
    /// Prefer using the typed storage functions in the sibling modules. This
    /// accessor is provided for advanced queries not covered by the storage API.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database").finish_non_exhaustive()
    }
}
