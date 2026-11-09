use crm_core::storage::contacts::{
    Contact, ContactDuplicateCandidate, ContactListParams, ContactListResult,
};
use tauri::State;

use crate::{commands::lock_core, AppState};

// Preserve the existing field-level IPC command shape for frontend callers.
#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub async fn create_contact(
    state: State<'_, AppState>,
    contact_type: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    org_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    address: Option<String>,
    city: Option<String>,
    country: Option<String>,
    org_id: Option<String>,
    notes: Option<String>,
    lifecycle: Option<String>,
    owner: Option<String>,
) -> Result<Contact, String> {
    let mut core = lock_core(&state)?;
    core.create_contact_with_lifecycle(
        contact_type,
        first_name,
        last_name,
        org_name,
        email,
        phone,
        address,
        city,
        country,
        org_id,
        notes,
        lifecycle,
        owner,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_contact(state: State<'_, AppState>, id: String) -> Result<Contact, String> {
    let core = lock_core(&state)?;
    core.get_contact(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_contacts(
    state: State<'_, AppState>,
    params: Option<ContactListParams>,
) -> Result<ContactListResult, String> {
    let core = lock_core(&state)?;
    core.list_contacts(params).map_err(|e| e.to_string())
}

// Preserve the existing field-level IPC command shape for frontend callers.
#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub async fn update_contact(
    state: State<'_, AppState>,
    id: String,
    contact_type: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    org_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    address: Option<String>,
    city: Option<String>,
    country: Option<String>,
    notes: Option<String>,
    owner: Option<String>,
    reset_owner: Option<bool>,
) -> Result<Contact, String> {
    let mut core = lock_core(&state)?;
    let owner = if reset_owner.unwrap_or(false) {
        Some(None)
    } else {
        owner.map(Some)
    };
    core.update_contact(
        &id,
        contact_type,
        first_name,
        last_name,
        org_name,
        email,
        phone,
        address,
        city,
        country,
        notes,
        owner,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_contact(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut core = lock_core(&state)?;
    core.delete_contact(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn restore_contact(state: State<'_, AppState>, id: String) -> Result<Contact, String> {
    let mut core = lock_core(&state)?;
    core.restore_contact(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search_contacts(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<Contact>, String> {
    let core = lock_core(&state)?;
    core.search_contacts(&query).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_contact_duplicate_candidates(
    state: State<'_, AppState>,
) -> Result<Vec<ContactDuplicateCandidate>, String> {
    let core = lock_core(&state)?;
    core.list_contact_duplicate_candidates()
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn merge_contacts(
    state: State<'_, AppState>,
    target_id: String,
    source_id: String,
) -> Result<Contact, String> {
    let mut core = lock_core(&state)?;
    core.merge_contacts(&target_id, &source_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_contact_lifecycle(
    state: State<'_, AppState>,
    id: String,
    lifecycle: String,
) -> Result<Contact, String> {
    let mut core = lock_core(&state)?;
    core.set_contact_lifecycle(&id, &lifecycle)
        .map_err(|e| e.to_string())
}
