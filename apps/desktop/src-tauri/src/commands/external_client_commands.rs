use crm_core::{
    permissions::ToolPermissionEvaluation,
    storage::{
        external_client_permissions::ExternalClientPermission, external_clients::ExternalClient,
    },
};
use tauri::State;

use crate::{commands::lock_core, AppState};

#[tauri::command(rename_all = "snake_case")]
pub async fn list_external_clients(
    state: State<'_, AppState>,
) -> Result<Vec<ExternalClient>, String> {
    let core = lock_core(&state)?;
    core.list_external_clients().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn create_external_client_placeholder(
    state: State<'_, AppState>,
    name: String,
    client_type: String,
) -> Result<ExternalClient, String> {
    let mut core = lock_core(&state)?;
    core.create_external_client_placeholder(&name, &client_type)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_external_client_activation(
    state: State<'_, AppState>,
    client_id: String,
    enabled: bool,
    permission_mode: String,
) -> Result<ExternalClient, String> {
    let mut core = lock_core(&state)?;
    core.update_external_client_activation(&client_id, enabled, &permission_mode)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_external_client_permissions(
    state: State<'_, AppState>,
    client_id: String,
) -> Result<Vec<ExternalClientPermission>, String> {
    let core = lock_core(&state)?;
    core.list_external_client_permissions(&client_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn upsert_external_client_tool_permission(
    state: State<'_, AppState>,
    client_id: String,
    tool_name: String,
    can_read: bool,
    can_write: bool,
    requires_confirmation: bool,
) -> Result<ExternalClientPermission, String> {
    let mut core = lock_core(&state)?;
    core.upsert_external_client_tool_permission(
        &client_id,
        &tool_name,
        can_read,
        can_write,
        requires_confirmation,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn evaluate_external_client_tool_read_permission(
    state: State<'_, AppState>,
    client_id: String,
    tool_name: String,
) -> Result<ToolPermissionEvaluation, String> {
    let core = lock_core(&state)?;
    core.evaluate_external_client_tool_read_permission(&client_id, &tool_name)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn evaluate_external_client_draft_permission(
    state: State<'_, AppState>,
    client_id: String,
    tool_name: String,
) -> Result<ToolPermissionEvaluation, String> {
    let core = lock_core(&state)?;
    core.evaluate_external_client_draft_permission(&client_id, &tool_name)
        .map_err(|e| e.to_string())
}
