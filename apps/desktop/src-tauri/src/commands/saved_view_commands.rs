use crm_core::storage::saved_views::SavedView;
use tauri::State;

use crate::{commands::lock_core, AppState};

#[tauri::command(rename_all = "snake_case")]
pub async fn create_saved_view(
    state: State<'_, AppState>,
    entity_type: String,
    name: String,
    filters_json: String,
) -> Result<SavedView, String> {
    let mut core = lock_core(&state)?;
    core.create_saved_view(entity_type, name, filters_json)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_saved_views(
    state: State<'_, AppState>,
    entity_type: String,
) -> Result<Vec<SavedView>, String> {
    let core = lock_core(&state)?;
    core.list_saved_views(entity_type)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_saved_view(
    state: State<'_, AppState>,
    id: String,
    name: String,
    filters_json: String,
) -> Result<SavedView, String> {
    let mut core = lock_core(&state)?;
    core.update_saved_view(&id, name, filters_json)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_saved_view(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut core = lock_core(&state)?;
    core.delete_saved_view(&id).map_err(|e| e.to_string())
}
