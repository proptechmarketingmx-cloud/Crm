use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crm_core::{
    services::{LocalBackup, LocalBackupValidation, LocalRestoreResult},
    CrmCore,
};
use tauri::State;

use crate::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn create_local_backup(
    state: State<'_, AppState>,
    backup_dir: String,
) -> Result<LocalBackup, String> {
    let backup_dir = match super::path_guard::validate_export_path(&backup_dir) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.create_local_backup(Path::new(&backup_dir))
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn validate_local_backup(
    state: State<'_, AppState>,
    backup_dir: String,
) -> Result<LocalBackupValidation, String> {
    let backup_dir = match super::path_guard::validate_import_path(&backup_dir) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let core = super::lock_core(&state)?;
    core.validate_local_backup(Path::new(&backup_dir))
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn restore_local_backup_to_app_data(
    state: State<'_, AppState>,
    backup_dir: String,
    confirm_destructive_restore: bool,
) -> Result<LocalRestoreResult, String> {
    let backup_dir = match super::path_guard::validate_import_path(&backup_dir) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(msg) => return Err(msg),
    };
    let app_data_dir = state.data_dir.clone();
    close_active_core(&state)?;

    let restore_result = CrmCore::restore_local_backup_to_app_data(
        &app_data_dir,
        Path::new(&backup_dir),
        confirm_destructive_restore,
    )
    .map_err(|e| e.to_string());

    let reopen_result = reopen_core(&state, &app_data_dir);
    match (restore_result, reopen_result) {
        (Ok(result), Ok(())) => Ok(result),
        (Err(restore_error), Ok(())) => Err(restore_error),
        (Ok(_), Err(reopen_error)) => Err(format!(
            "Local restore completed but CrmCore failed to reopen: {}",
            reopen_error
        )),
        (Err(restore_error), Err(reopen_error)) => Err(format!(
            "Local restore failed: {}; CrmCore also failed to reopen: {}",
            restore_error, reopen_error
        )),
    }
}

fn close_active_core(state: &State<'_, AppState>) -> Result<(), String> {
    let mut core_slot = state
        .core
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    drop(core_slot.take());
    state.needs_reopen.store(false, Ordering::SeqCst);
    Ok(())
}

fn reopen_core(state: &State<'_, AppState>, app_data_dir: &Path) -> Result<(), String> {
    let mut last_error: Option<String> = None;
    for attempt in 0..3 {
        match CrmCore::open(app_data_dir) {
            Ok(reopened_core) => {
                let mut core_slot = state
                    .core
                    .lock()
                    .map_err(|e| format!("Lock error: {}", e))?;
                *core_slot = Some(reopened_core);
                state.needs_reopen.store(false, Ordering::SeqCst);
                return Ok(());
            }
            Err(e) => {
                last_error = Some(e.to_string());
                if attempt + 1 < 3 {
                    std::thread::sleep(Duration::from_millis(250));
                }
            }
        }
    }
    state.needs_reopen.store(true, Ordering::SeqCst);
    Err(last_error.unwrap_or_else(|| "Unknown reopen error".to_string()))
}
