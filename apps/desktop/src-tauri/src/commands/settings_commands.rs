use std::collections::HashMap;

use crm_core::storage::settings::Setting;
use tauri::State;

use crate::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let core = super::lock_core(&state)?;
    core.get_settings().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_setting(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<Setting>, String> {
    let core = super::lock_core(&state)?;
    core.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<Setting, String> {
    let mut core = super::lock_core(&state)?;
    core.update_setting(key, value).map_err(|e| e.to_string())
}
