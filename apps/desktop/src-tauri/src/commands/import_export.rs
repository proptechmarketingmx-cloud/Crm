use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crm_core::utils::csv::ImportColumnMapping;
use crm_core::{
    services::{
        ImportOptions, ImportPreflightReport, ImportResult, ImportRollbackPlan,
        ImportRollbackResult, LocalBackup,
    },
    utils::csv::JsonImportPreview,
    CrmCore,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

static PRE_IMPORT_BACKUP_COUNTER: AtomicU64 = AtomicU64::new(0);

const MAX_IMPORT_FILE_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWithBackupResult {
    pub import: ImportResult,
    pub backup: LocalBackup,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn read_import_text(file_path: String) -> Result<String, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p,
        Err(msg) => return Err(msg),
    };
    let path = file_path.to_string_lossy().to_string();
    enforce_size_limit(&path)?;
    std::fs::read_to_string(&file_path).map_err(|err| format!("Unable to read import file: {err}"))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn validate_open_path(file_path: String) -> Result<String, String> {
    super::path_guard::validate_import_path(&file_path)
        .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn write_export_text(file_path: String, contents: String) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(path) => path,
        Err(msg) => return Err(msg),
    };
    if contents.len() > MAX_IMPORT_FILE_BYTES as usize {
        return Err("Export is too large to write".to_string());
    }
    if let Some(parent) = file_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("Unable to create export folder: {err}"))?;
        }
    }
    std::fs::write(&file_path, contents.as_bytes())
        .map_err(|err| format!("Unable to write export file: {err}"))?;
    Ok(1)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn rollback_completed_import(
    state: State<'_, AppState>,
    rollback_plan: ImportRollbackPlan,
) -> Result<ImportRollbackResult, String> {
    let mut core = super::lock_core(&state)?;
    core.rollback_completed_import(&rollback_plan)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_contacts_csv(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_contacts_csv_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_contacts_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_contacts_csv_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_contacts_json(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_contacts_json_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_contacts_json_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_contacts_json_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preview_contacts_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<JsonImportPreview, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preview_contacts_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_contacts_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_contacts_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_contacts_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_contacts_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_contacts_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_contacts_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_contacts_json_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_contacts_json_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_contacts_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_contacts_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_contacts_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_contacts_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_audit_log_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_audit_log_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_audit_log_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_audit_log_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_proposed_actions_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_proposed_actions_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_proposed_actions_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_proposed_actions_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_external_clients_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_external_clients_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_external_clients_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_external_clients_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_external_client_permissions_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_external_client_permissions_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_external_client_permissions_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_external_client_permissions_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_deals_csv(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_deals_csv_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_deals_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_deals_csv_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_deals_json(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_deals_json_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_deals_json_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_deals_json_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preview_deals_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<JsonImportPreview, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preview_deals_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_deals_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_deals_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_deals_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_deals_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_deals_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_deals_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_deals_json_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_deals_json_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_deals_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_deals_csv(&file_path).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_deals_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_deals_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_activities_csv(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_activities_csv_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_activities_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_activities_csv_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_activities_json(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_activities_json_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_activities_json_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_activities_json_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preview_activities_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<JsonImportPreview, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preview_activities_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_activities_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_activities_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_activities_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_activities_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_activities_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_activities_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_activities_json_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_activities_json_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_activities_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_activities_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_activities_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_activities_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_notes_csv(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_notes_csv_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_notes_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_notes_csv_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_notes_json(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_notes_json_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_notes_json_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_notes_json_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preview_notes_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<JsonImportPreview, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preview_notes_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_notes_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_notes_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_notes_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_notes_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_notes_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_notes_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_notes_json_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_notes_json_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_notes_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_notes_csv(&file_path).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_notes_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_notes_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_tag_definitions_csv(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_tag_definitions_csv_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_tag_definitions_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_tag_definitions_csv_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_tag_definitions_json(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_tag_definitions_json_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_tag_definitions_json_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_tag_definitions_json_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preview_tag_definitions_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<JsonImportPreview, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preview_tag_definitions_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_tag_definitions_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_tag_definitions_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_tag_definitions_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_tag_definitions_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_tag_definitions_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_tag_definitions_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_tag_definitions_json_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_tag_definitions_json_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_tag_definitions_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_tag_definitions_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_tag_definitions_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_tag_definitions_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_custom_field_definitions_csv(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_custom_field_definitions_csv_with_options(
            &file_path,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_custom_field_definitions_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_custom_field_definitions_csv_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_custom_field_definitions_json(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_custom_field_definitions_json_with_options(
            &file_path,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_custom_field_definitions_json_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_custom_field_definitions_json_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preview_custom_field_definitions_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<JsonImportPreview, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preview_custom_field_definitions_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_custom_field_definitions_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_custom_field_definitions_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_custom_field_definitions_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_custom_field_definitions_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_custom_field_definitions_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_custom_field_definitions_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_custom_field_definitions_json_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_custom_field_definitions_json_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_custom_field_definitions_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_custom_field_definitions_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_custom_field_definitions_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_custom_field_definitions_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_tag_links_csv(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_tag_links_csv_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_tag_links_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_tag_links_csv_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_tag_links_json(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_tag_links_json_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_tag_links_json_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_tag_links_json_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preview_tag_links_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<JsonImportPreview, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preview_tag_links_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_tag_links_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_tag_links_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_tag_links_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_tag_links_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_tag_links_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_tag_links_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_tag_links_json_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_tag_links_json_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_tag_links_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_tag_links_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_tag_links_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_tag_links_json(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_organizations_csv(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_organizations_csv_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_organizations_csv_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_organizations_csv_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_organizations_json(
    state: State<'_, AppState>,
    file_path: String,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_organizations_json_with_options(&file_path, import_options(merge_duplicates))
            .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn import_organizations_json_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
    merge_duplicates: Option<bool>,
) -> Result<ImportWithBackupResult, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    import_with_pre_import_backup(&state, |core| {
        core.import_organizations_json_with_mapping_and_options(
            &file_path,
            mapping,
            import_options(merge_duplicates),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preview_organizations_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<JsonImportPreview, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preview_organizations_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_organizations_csv_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_organizations_csv_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_organizations_csv_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_organizations_csv_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_organizations_json_import(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_organizations_json_import(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn preflight_organizations_json_import_with_mapping(
    state: State<'_, AppState>,
    file_path: String,
    mapping: ImportColumnMapping,
) -> Result<ImportPreflightReport, String> {
    let file_path = match super::path_guard::validate_import_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    enforce_size_limit(&file_path)?;
    let core = super::lock_core(&state)?;
    core.preflight_organizations_json_import_with_mapping(&file_path, mapping)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_organizations_csv(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_organizations_csv(&file_path)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_organizations_json(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<u32, String> {
    let file_path = match super::path_guard::validate_export_path(&file_path) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.export_organizations_json(&file_path)
        .map_err(|e| e.to_string())
}

fn import_with_pre_import_backup<F>(
    state: &State<'_, AppState>,
    import: F,
) -> Result<ImportWithBackupResult, String>
where
    F: FnOnce(&mut CrmCore) -> Result<ImportResult, String>,
{
    let mut core = super::lock_core(state)?;
    let backup_dir = next_pre_import_backup_dir(&state.data_dir)?;
    create_backup_then_import(&mut core, &backup_dir, import)
}

/// Rejects import paths that are not regular files or that exceed the size cap.
fn enforce_size_limit(path: &str) -> Result<(), String> {
    use std::fs;
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };
    if !meta.is_file() {
        return Err("Import path is not a regular file.".to_string());
    }
    if meta.len() > MAX_IMPORT_FILE_BYTES {
        return Err(format!(
            "Import file is too large ({} bytes); the maximum is {} bytes.",
            meta.len(),
            MAX_IMPORT_FILE_BYTES
        ));
    }
    Ok(())
}

fn import_options(merge_duplicates: Option<bool>) -> ImportOptions {
    ImportOptions {
        merge_duplicates: merge_duplicates.unwrap_or(false),
    }
}

fn create_backup_then_import<F>(
    core: &mut CrmCore,
    backup_dir: &Path,
    import: F,
) -> Result<ImportWithBackupResult, String>
where
    F: FnOnce(&mut CrmCore) -> Result<ImportResult, String>,
{
    let backup = core
        .create_local_backup(backup_dir)
        .map_err(|e| e.to_string())?;
    let import = import(core)?;
    Ok(ImportWithBackupResult { import, backup })
}

fn next_pre_import_backup_dir(app_data_dir: &Path) -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System clock is before UNIX_EPOCH: {}", e))?
        .as_millis();
    let sequence = PRE_IMPORT_BACKUP_COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(app_data_dir
        .join("pre-import-backups")
        .join(format!("{}-{}", timestamp, sequence)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, panic, path::PathBuf};

    fn unique_test_dir(name: &str) -> PathBuf {
        let sequence = PRE_IMPORT_BACKUP_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "900crm-pre-import-backup-{}-{}-{}",
            name,
            std::process::id(),
            sequence
        ))
    }

    fn write_csv(path: &Path, contents: &str) {
        fs::write(path, contents).expect("test CSV should be writable");
    }

    #[test]
    fn write_export_text_writes_a_validated_file() {
        let dir = unique_test_dir("write-export-text");
        fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("snapshot.csv");
        let contents = "section,name,field,value\nmeta,snapshot,note,ok\n";

        let written = tauri::async_runtime::block_on(write_export_text(
            path.to_string_lossy().to_string(),
            contents.to_string(),
        ))
        .expect("write export text");

        assert_eq!(written, 1);
        assert_eq!(fs::read_to_string(&path).expect("read export"), contents);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn pre_import_backup_path_is_unique_under_app_data() {
        let app_data_dir = PathBuf::from("/tmp/900crm-app-data");
        let first = next_pre_import_backup_dir(&app_data_dir).expect("first path");
        let second = next_pre_import_backup_dir(&app_data_dir).expect("second path");

        assert_ne!(first, second);
        assert_eq!(
            first.parent().unwrap(),
            app_data_dir.join("pre-import-backups")
        );
        assert_eq!(
            second.parent().unwrap(),
            app_data_dir.join("pre-import-backups")
        );
    }

    #[test]
    fn backup_is_created_before_import_writes() {
        let app_data_dir = unique_test_dir("creates-before-write");
        let csv_path = app_data_dir.join("contacts.csv");
        fs::create_dir_all(&app_data_dir).expect("app data dir");
        write_csv(
            &csv_path,
            "first_name,last_name,email\nAda,Lovelace,ada@example.com\n",
        );

        let mut core = CrmCore::open(&app_data_dir).expect("core opens");
        let backup_dir = app_data_dir.join("pre-import-backups").join("before");
        let result = create_backup_then_import(&mut core, &backup_dir, |core| {
            core.import_contacts_csv(csv_path.to_str().expect("utf8 path"))
                .map_err(|e| e.to_string())
        })
        .expect("backup and import should succeed");

        assert_eq!(result.import.created, 1);
        assert!(backup_dir.join("900crm.db").is_file());
        assert_eq!(result.backup.backup_dir, backup_dir.to_string_lossy());

        let backup_core = CrmCore::open(&backup_dir).expect("backup core opens");
        let backup_contacts = backup_core
            .list_contacts(None)
            .expect("backup contacts list");
        assert_eq!(backup_contacts.contacts.len(), 0);

        let active_contacts = core.list_contacts(None).expect("active contacts list");
        assert_eq!(active_contacts.contacts.len(), 1);

        fs::remove_dir_all(&app_data_dir).ok();
    }

    #[test]
    fn backup_is_created_before_auto_merge_import_writes() {
        let app_data_dir = unique_test_dir("creates-before-auto-merge");
        let csv_path = app_data_dir.join("contacts.csv");
        fs::create_dir_all(&app_data_dir).expect("app data dir");

        let mut core = CrmCore::open(&app_data_dir).expect("core opens");
        let existing = core
            .create_contact(
                Some("person".to_string()),
                Some("Ada".to_string()),
                Some("Lovelace".to_string()),
                None,
                Some("ada@example.com".to_string()),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .expect("existing contact should be created");
        write_csv(
            &csv_path,
            "first_name,last_name,email,phone\nImported,Duplicate,ADA@example.com,+15550123\n",
        );

        let backup_dir = app_data_dir.join("pre-import-backups").join("before-merge");
        let result = create_backup_then_import(&mut core, &backup_dir, |core| {
            core.import_contacts_csv_with_options(
                csv_path.to_str().expect("utf8 path"),
                ImportOptions {
                    merge_duplicates: true,
                },
            )
            .map_err(|e| e.to_string())
        })
        .expect("backup and auto-merge import should succeed");

        assert_eq!(result.import.created, 0);
        assert_eq!(result.import.merged, 1);
        assert!(backup_dir.join("900crm.db").is_file());

        let backup_core = CrmCore::open(&backup_dir).expect("backup core opens");
        let backup_contact = backup_core
            .get_contact(&existing.id)
            .expect("backup should contain pre-merge contact");
        assert_eq!(backup_contact.phone, "");

        let active_contact = core
            .get_contact(&existing.id)
            .expect("active contact should remain after merge");
        assert_eq!(active_contact.phone, "+15550123");

        fs::remove_dir_all(&app_data_dir).ok();
    }

    #[test]
    fn backup_is_created_before_deal_auto_merge_import_writes() {
        let app_data_dir = unique_test_dir("creates-before-deal-auto-merge");
        let csv_path = app_data_dir.join("deals.csv");
        fs::create_dir_all(&app_data_dir).expect("app data dir");

        let mut core = CrmCore::open(&app_data_dir).expect("core opens");
        let existing = core
            .create_deal(
                "Acme Renewal".to_string(),
                Some(0.0),
                Some("USD".to_string()),
                Some("Lead".to_string()),
                Some(10),
                None,
                None,
                None,
                None,
            )
            .expect("existing deal should be created");
        write_csv(
            &csv_path,
            "title,value,currency,stage,expected_close,notes\n\
             acme renewal,7500,EUR,Negotiation,2026-10-15,Imported note\n",
        );

        let backup_dir = app_data_dir
            .join("pre-import-backups")
            .join("before-deal-merge");
        let result = create_backup_then_import(&mut core, &backup_dir, |core| {
            core.import_deals_csv_with_options(
                csv_path.to_str().expect("utf8 path"),
                ImportOptions {
                    merge_duplicates: true,
                },
            )
            .map_err(|e| e.to_string())
        })
        .expect("backup and deal auto-merge import should succeed");

        assert_eq!(result.import.created, 0);
        assert_eq!(result.import.merged, 1);
        assert!(backup_dir.join("900crm.db").is_file());

        let backup_core = CrmCore::open(&backup_dir).expect("backup core opens");
        let backup_deal = backup_core
            .get_deal(&existing.id)
            .expect("backup should contain pre-merge deal");
        assert_eq!(backup_deal.value, 0.0);
        assert_eq!(backup_deal.notes, "");

        let active_deal = core
            .get_deal(&existing.id)
            .expect("active deal should remain after merge");
        assert_eq!(active_deal.value, 7500.0);
        assert_eq!(active_deal.notes, "Imported note");

        fs::remove_dir_all(&app_data_dir).ok();
    }

    #[test]
    fn backup_is_created_before_activity_import_writes() {
        let app_data_dir = unique_test_dir("creates-before-activity-write");
        let csv_path = app_data_dir.join("activities.csv");
        fs::create_dir_all(&app_data_dir).expect("app data dir");
        write_csv(
            &csv_path,
            "activity_type,title,description,completed\ncall,Intro call,Imported activity,true\n",
        );

        let mut core = CrmCore::open(&app_data_dir).expect("core opens");
        let backup_dir = app_data_dir
            .join("pre-import-backups")
            .join("before-activity");
        let result = create_backup_then_import(&mut core, &backup_dir, |core| {
            core.import_activities_csv(csv_path.to_str().expect("utf8 path"))
                .map_err(|e| e.to_string())
        })
        .expect("backup and activity import should succeed");

        assert_eq!(result.import.created, 1);
        assert!(backup_dir.join("900crm.db").is_file());

        let backup_core = CrmCore::open(&backup_dir).expect("backup core opens");
        let backup_activities = backup_core
            .list_activities()
            .expect("backup activities list");
        assert_eq!(backup_activities.len(), 0);

        let active_activities = core.list_activities().expect("active activities list");
        assert_eq!(active_activities.len(), 1);
        assert_eq!(active_activities[0].title, "Intro call");
        assert!(active_activities[0].completed);

        fs::remove_dir_all(&app_data_dir).ok();
    }

    #[test]
    fn backup_is_created_before_tag_definition_import_writes() {
        let app_data_dir = unique_test_dir("creates-before-tag-write");
        let csv_path = app_data_dir.join("tag-definitions.csv");
        fs::create_dir_all(&app_data_dir).expect("app data dir");
        write_csv(&csv_path, "name,color\nVIP,#ef4444\n");

        let mut core = CrmCore::open(&app_data_dir).expect("core opens");
        let backup_dir = app_data_dir.join("pre-import-backups").join("before-tag");
        let result = create_backup_then_import(&mut core, &backup_dir, |core| {
            core.import_tag_definitions_csv(csv_path.to_str().expect("utf8 path"))
                .map_err(|e| e.to_string())
        })
        .expect("backup and tag definition import should succeed");

        assert_eq!(result.import.created, 1);
        assert!(backup_dir.join("900crm.db").is_file());

        let backup_core = CrmCore::open(&backup_dir).expect("backup core opens");
        let backup_tags = backup_core.list_tags().expect("backup tags list");
        assert_eq!(backup_tags.len(), 0);

        let active_tags = core.list_tags().expect("active tags list");
        assert_eq!(active_tags.len(), 1);
        assert_eq!(active_tags[0].name, "VIP");

        fs::remove_dir_all(&app_data_dir).ok();
    }

    #[test]
    fn backup_is_created_before_custom_field_definition_import_writes() {
        let app_data_dir = unique_test_dir("creates-before-custom-field-definition-write");
        let csv_path = app_data_dir.join("custom-field-definitions.csv");
        fs::create_dir_all(&app_data_dir).expect("app data dir");
        write_csv(
            &csv_path,
            "entity_type,field_name,field_type,field_options,sort_order\ncontact,Tier,text,,1\n",
        );

        let mut core = CrmCore::open(&app_data_dir).expect("core opens");
        let backup_dir = app_data_dir
            .join("pre-import-backups")
            .join("before-custom-field-definition");
        let result = create_backup_then_import(&mut core, &backup_dir, |core| {
            core.import_custom_field_definitions_csv(csv_path.to_str().expect("utf8 path"))
                .map_err(|e| e.to_string())
        })
        .expect("backup and custom field definition import should succeed");

        assert_eq!(result.import.created, 1);
        assert!(backup_dir.join("900crm.db").is_file());

        let backup_core = CrmCore::open(&backup_dir).expect("backup core opens");
        let backup_definitions = backup_core
            .list_custom_field_defs(None)
            .expect("backup custom field definitions list");
        assert_eq!(backup_definitions.len(), 0);

        let active_definitions = core
            .list_custom_field_defs(None)
            .expect("active custom field definitions list");
        assert_eq!(active_definitions.len(), 1);
        assert_eq!(active_definitions[0].field_name, "Tier");

        fs::remove_dir_all(&app_data_dir).ok();
    }

    #[test]
    fn failed_backup_prevents_import() {
        let app_data_dir = unique_test_dir("failed-backup");
        let backup_dir = app_data_dir.join("pre-import-backups").join("blocked");
        fs::create_dir_all(&backup_dir).expect("backup dir");
        fs::write(backup_dir.join("900crm.db"), b"existing").expect("existing db marker");

        let mut core = CrmCore::open(&app_data_dir).expect("core opens");
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            create_backup_then_import(&mut core, &backup_dir, |_core| {
                panic!("import must not run when backup fails")
            })
        }))
        .expect("helper should return an error, not panic");

        assert!(result
            .expect_err("backup should fail")
            .contains("already exists"));

        fs::remove_dir_all(&app_data_dir).ok();
    }
}
