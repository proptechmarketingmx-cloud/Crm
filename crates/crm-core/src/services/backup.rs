use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};

use crate::result::CrmResult;
use crate::storage;
use crate::utils::{
    datetime::now_iso8601,
    errors::{CrmError, CrmResult as InternalCrmResult},
    uuid::new_uuid,
};

use super::CrmCore;

const BACKUP_FORMAT_VERSION: u32 = 1;
const BACKUP_METADATA_FILE: &str = "metadata.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalBackupMetadata {
    pub backup_format_version: u32,
    pub created_at: String,
    pub app_version: String,
    pub schema_version: u32,
    pub device_id: String,
    pub database_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalBackup {
    pub backup_dir: String,
    pub database_path: String,
    pub metadata_path: String,
    pub metadata: LocalBackupMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalBackupValidation {
    pub backup_dir: String,
    pub database_path: String,
    pub metadata_path: String,
    pub metadata: LocalBackupMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalRestoreResult {
    pub restored_at: String,
    pub database_path: String,
    pub metadata: LocalBackupMetadata,
}

impl CrmCore {
    /// Creates a full local SQLite backup directory.
    ///
    /// The backup is a standalone SQLite database snapshot plus metadata. The
    /// method refuses to overwrite an existing backup database or metadata file.
    pub fn create_local_backup(&self, backup_dir: &Path) -> CrmResult<LocalBackup> {
        let database_path = backup_dir.join(storage::Database::database_filename());
        let metadata_path = backup_dir.join(BACKUP_METADATA_FILE);

        ensure_backup_targets_available(&database_path, &metadata_path)?;

        let schema_version = self.db.schema_version()?;
        let metadata = LocalBackupMetadata {
            backup_format_version: BACKUP_FORMAT_VERSION,
            created_at: now_iso8601(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            schema_version,
            device_id: self.device_id.clone(),
            database_file: storage::Database::database_filename().to_string(),
        };

        self.db.write_snapshot(&database_path)?;
        write_metadata(&metadata_path, &metadata)?;

        Ok(LocalBackup {
            backup_dir: path_to_string(backup_dir),
            database_path: path_to_string(&database_path),
            metadata_path: path_to_string(&metadata_path),
            metadata,
        })
    }

    /// Validates backup metadata and database compatibility without applying it.
    pub fn validate_local_backup(&self, backup_dir: &Path) -> CrmResult<LocalBackupValidation> {
        validate_backup_dir(backup_dir)
    }

    /// Restores a validated local backup into an app data directory.
    ///
    /// This associated function does not use an open `CrmCore` because restore
    /// replaces the SQLite database file. Callers must close the active core
    /// first, validate the backup, and pass `confirm_destructive_restore = true`.
    pub fn restore_local_backup_to_app_data(
        app_data_dir: &Path,
        backup_dir: &Path,
        confirm_destructive_restore: bool,
    ) -> CrmResult<LocalRestoreResult> {
        let validation = validate_backup_dir(backup_dir)?;

        if !confirm_destructive_restore {
            return Err(CrmError::InvalidInput(
                "Local restore is destructive and requires explicit confirmation".to_string(),
            ));
        }

        fs::create_dir_all(app_data_dir)?;
        let target_database_path = app_data_dir.join(storage::Database::database_filename());
        reject_self_restore(&validation.database_path, &target_database_path)?;

        let temporary_restore_path = app_data_dir.join("900crm.db.restore_tmp");
        if temporary_restore_path.exists() {
            fs::remove_file(&temporary_restore_path)?;
        }

        let mut restore_guard = TempFileGuard::prepare(&temporary_restore_path)?;
        fs::copy(&validation.database_path, restore_guard.path())?;
        replace_database_file(restore_guard.path(), &target_database_path)?;
        restore_guard.disarm();
        remove_sidecar_if_exists(&target_database_path, "wal")?;
        remove_sidecar_if_exists(&target_database_path, "shm")?;

        Ok(LocalRestoreResult {
            restored_at: now_iso8601(),
            database_path: path_to_string(&target_database_path),
            metadata: validation.metadata,
        })
    }
}

fn ensure_backup_targets_available(database_path: &Path, metadata_path: &Path) -> CrmResult<()> {
    if database_path.exists() {
        return Err(CrmError::InvalidInput(format!(
            "Backup database '{}' already exists",
            database_path.display()
        )));
    }
    if metadata_path.exists() {
        return Err(CrmError::InvalidInput(format!(
            "Backup metadata '{}' already exists",
            metadata_path.display()
        )));
    }
    Ok(())
}

fn validate_backup_dir(backup_dir: &Path) -> CrmResult<LocalBackupValidation> {
    let metadata_path = backup_dir.join(BACKUP_METADATA_FILE);
    let metadata = read_metadata(&metadata_path)?;
    validate_metadata(&metadata)?;

    let database_path = backup_dir.join(&metadata.database_file);
    validate_backup_database(&database_path, metadata.schema_version)?;

    Ok(LocalBackupValidation {
        backup_dir: path_to_string(backup_dir),
        database_path: path_to_string(&database_path),
        metadata_path: path_to_string(&metadata_path),
        metadata,
    })
}

fn read_metadata(metadata_path: &Path) -> CrmResult<LocalBackupMetadata> {
    let bytes = fs::read(metadata_path).map_err(|err| {
        CrmError::InvalidInput(format!(
            "Backup metadata '{}' is not readable: {}",
            metadata_path.display(),
            err
        ))
    })?;
    serde_json::from_slice(&bytes).map_err(|err| {
        CrmError::InvalidInput(format!(
            "Backup metadata '{}' is invalid JSON: {}",
            metadata_path.display(),
            err
        ))
    })
}

fn write_metadata(metadata_path: &Path, metadata: &LocalBackupMetadata) -> CrmResult<()> {
    if let Some(parent) = metadata_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = serde_json::to_vec_pretty(metadata)?;
    fs::write(metadata_path, bytes)?;
    Ok(())
}

fn validate_metadata(metadata: &LocalBackupMetadata) -> CrmResult<()> {
    if metadata.backup_format_version != BACKUP_FORMAT_VERSION {
        return Err(CrmError::InvalidInput(format!(
            "Unsupported backup format version {}",
            metadata.backup_format_version
        )));
    }

    if metadata.app_version != env!("CARGO_PKG_VERSION") {
        return Err(CrmError::InvalidInput(format!(
            "Backup app version '{}' is not compatible with '{}'",
            metadata.app_version,
            env!("CARGO_PKG_VERSION")
        )));
    }

    if metadata.schema_version > storage::Database::current_schema_version() {
        return Err(CrmError::InvalidInput(format!(
            "Backup schema version {} is newer than supported schema version {}",
            metadata.schema_version,
            storage::Database::current_schema_version()
        )));
    }

    if metadata.device_id.trim().is_empty() {
        return Err(CrmError::InvalidInput(
            "Backup metadata is missing device_id".to_string(),
        ));
    }

    if metadata.database_file != storage::Database::database_filename() {
        return Err(CrmError::InvalidInput(format!(
            "Backup database file '{}' is not supported",
            metadata.database_file
        )));
    }

    Ok(())
}

fn validate_backup_database(database_path: &Path, expected_schema_version: u32) -> CrmResult<()> {
    if !database_path.is_file() {
        return Err(CrmError::InvalidInput(format!(
            "Backup database '{}' does not exist",
            database_path.display()
        )));
    }

    validate_sqlite_integrity(database_path)?;

    let conn = Connection::open_with_flags(
        database_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|err| invalid_backup_database(database_path, err))?;
    let actual_schema_version: u32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|err| invalid_backup_database(database_path, err))?;

    if actual_schema_version != expected_schema_version {
        return Err(CrmError::InvalidInput(format!(
            "Backup metadata schema version {} does not match database schema version {}",
            expected_schema_version, actual_schema_version
        )));
    }

    let required_table_count: i64 = conn
        .query_row(
            r#"
        SELECT COUNT(*)
        FROM sqlite_master
        WHERE type = 'table'
          AND name IN ('contacts', 'settings', 'sync_changelog')
        "#,
            [],
            |row| row.get(0),
        )
        .map_err(|err| invalid_backup_database(database_path, err))?;

    if required_table_count != 3 {
        return Err(CrmError::InvalidInput(format!(
            "Backup database is missing required core tables: found {} of 3",
            required_table_count
        )));
    }

    Ok(())
}

fn validate_sqlite_integrity(database_path: &Path) -> CrmResult<()> {
    let validation_copy_path = database_path.with_file_name(format!(
        ".{}.integrity-check-{}",
        database_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("900crm.db"),
        new_uuid()
    ));
    let mut validation_guard = TempFileGuard::prepare(&validation_copy_path)?;
    fs::copy(database_path, validation_guard.path())?;

    let conn = Connection::open(validation_guard.path())
        .map_err(|err| invalid_backup_database(database_path, err))?;
    let check_result: String = conn
        .query_row("PRAGMA quick_check;", [], |row| row.get(0))
        .map_err(|err| invalid_backup_database(database_path, err))?;

    drop(conn);

    if check_result != "ok" {
        return Err(CrmError::InvalidInput(format!(
            "Backup database integrity check failed: {}",
            check_result
        )));
    }

    validation_guard.cleanup()?;
    Ok(())
}

fn reject_self_restore(source_database_path: &str, target_database_path: &Path) -> CrmResult<()> {
    let source = PathBuf::from(source_database_path);
    let same_path = source == target_database_path
        || canonicalize_if_exists(&source)? == canonicalize_if_exists(target_database_path)?;

    if same_path {
        return Err(CrmError::InvalidInput(
            "Backup source and restore target are the same database file".to_string(),
        ));
    }

    Ok(())
}

fn replace_database_file(source: &Path, target: &Path) -> CrmResult<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(source, target)?;
    Ok(())
}

struct TempFileGuard {
    path: PathBuf,
    armed: bool,
}

impl TempFileGuard {
    fn prepare(path: &Path) -> CrmResult<Self> {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(CrmError::Io(err.to_string())),
        }

        Ok(Self {
            path: path.to_path_buf(),
            armed: true,
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn disarm(&mut self) {
        self.armed = false;
    }

    fn cleanup(&mut self) -> CrmResult<()> {
        if !self.armed {
            return Ok(());
        }

        match fs::remove_file(&self.path) {
            Ok(()) => {
                self.armed = false;
                Ok(())
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                self.armed = false;
                Ok(())
            }
            Err(err) => Err(CrmError::Io(err.to_string())),
        }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn remove_sidecar_if_exists(database_path: &Path, extension: &str) -> CrmResult<()> {
    let sidecar = database_path.with_extension(format!(
        "{}-{}",
        database_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("db"),
        extension
    ));

    match fs::remove_file(&sidecar) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(CrmError::Io(err.to_string())),
    }
}

fn canonicalize_if_exists(path: &Path) -> InternalCrmResult<PathBuf> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(path.to_path_buf()),
        Err(err) => Err(CrmError::Io(err.to_string())),
    }
}

fn invalid_backup_database(path: &Path, err: rusqlite::Error) -> CrmError {
    CrmError::InvalidInput(format!(
        "Backup database '{}' is invalid: {}",
        path.display(),
        err
    ))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
