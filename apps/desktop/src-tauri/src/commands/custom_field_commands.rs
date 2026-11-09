use crm_core::storage::custom_fields::{
    CustomFieldDefinition, CustomFieldValue, EntityCustomFieldValue, EntityTypeCustomFieldValue,
};
use tauri::State;

use crate::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn list_custom_field_defs(
    state: State<'_, AppState>,
    entity_type: Option<String>,
) -> Result<Vec<CustomFieldDefinition>, String> {
    let core = super::lock_core(&state)?;
    core.list_custom_field_defs(entity_type)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_custom_field_def(
    state: State<'_, AppState>,
    entity_type: String,
    field_name: String,
    field_type: String,
    field_options: Option<String>,
    sort_order: Option<i32>,
) -> Result<CustomFieldDefinition, String> {
    let mut core = super::lock_core(&state)?;
    core.create_custom_field_def(
        entity_type,
        field_name,
        field_type,
        field_options,
        sort_order,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_custom_field_def(
    state: State<'_, AppState>,
    id: String,
    field_name: Option<String>,
    field_type: Option<String>,
    field_options: Option<String>,
    sort_order: Option<i32>,
) -> Result<CustomFieldDefinition, String> {
    let mut core = super::lock_core(&state)?;
    core.update_custom_field_def(&id, field_name, field_type, field_options, sort_order)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_custom_field_def(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut core = super::lock_core(&state)?;
    core.delete_custom_field_def(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_custom_field_value(
    state: State<'_, AppState>,
    field_def_id: String,
    entity_id: String,
    value: String,
) -> Result<CustomFieldValue, String> {
    let mut core = super::lock_core(&state)?;
    core.set_custom_field_value(field_def_id, entity_id, value)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_custom_field_values(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<EntityCustomFieldValue>, String> {
    let core = super::lock_core(&state)?;
    core.list_custom_field_values(&entity_type, &entity_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_custom_field_values_for_type(
    state: State<'_, AppState>,
    entity_type: String,
) -> Result<Vec<EntityTypeCustomFieldValue>, String> {
    let core = super::lock_core(&state)?;
    core.list_custom_field_values_for_type(&entity_type)
        .map_err(|e| e.to_string())
}
