use crm_core::storage::notes::Note;
use tauri::State;

use crate::{commands::lock_core, AppState};

#[tauri::command(rename_all = "snake_case")]
pub async fn create_note(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
    content: String,
) -> Result<Note, String> {
    let mut core = lock_core(&state)?;
    core.create_note(entity_type, entity_id, content)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_note(state: State<'_, AppState>, id: String) -> Result<Note, String> {
    let core = lock_core(&state)?;
    core.get_note(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_notes_for_entity(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<Note>, String> {
    let core = lock_core(&state)?;
    core.list_notes_for_entity(entity_type, entity_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_note(
    state: State<'_, AppState>,
    id: String,
    content: String,
) -> Result<Note, String> {
    let mut core = lock_core(&state)?;
    core.update_note(&id, content).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut core = lock_core(&state)?;
    core.delete_note(&id).map_err(|e| e.to_string())
}
