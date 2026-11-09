use crm_core::storage::links::EntityLink;
use tauri::State;

use crate::{commands::lock_core, AppState};

#[tauri::command(rename_all = "snake_case")]
pub async fn create_entity_link(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
    title: Option<String>,
    kind: String,
    target: String,
) -> Result<EntityLink, String> {
    let mut core = lock_core(&state)?;
    core.create_entity_link(entity_type, entity_id, title, kind, target)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_entity_links(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<EntityLink>, String> {
    let core = lock_core(&state)?;
    core.list_entity_links(entity_type, entity_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_entity_link(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    kind: String,
    target: String,
) -> Result<EntityLink, String> {
    let mut core = lock_core(&state)?;
    core.update_entity_link(&id, title, kind, target)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_entity_link(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut core = lock_core(&state)?;
    core.delete_entity_link(&id).map_err(|e| e.to_string())
}
