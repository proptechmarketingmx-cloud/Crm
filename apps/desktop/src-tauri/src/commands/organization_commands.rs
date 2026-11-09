use crm_core::storage::{contacts::Contact, organizations::Organization};
use tauri::State;

use crate::{commands::lock_core, AppState};

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)]
pub async fn create_organization(
    state: State<'_, AppState>,
    name: String,
    email: Option<String>,
    phone: Option<String>,
    website: Option<String>,
    address_line1: Option<String>,
    address_line2: Option<String>,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    postal_code: Option<String>,
    description: Option<String>,
    owner: Option<String>,
) -> Result<Organization, String> {
    let mut core = lock_core(&state)?;
    let organization = core
        .create_organization(
            name,
            email,
            phone,
            website,
            address_line1,
            address_line2,
            city,
            region,
            country,
            postal_code,
            description,
        )
        .map_err(|e| e.to_string())?;
    if owner
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return core
            .set_organization_owner(&organization.id, owner.as_deref())
            .map_err(|e| e.to_string());
    }
    Ok(organization)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_organization(
    state: State<'_, AppState>,
    id: String,
) -> Result<Organization, String> {
    let core = lock_core(&state)?;
    core.get_organization(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_organizations(state: State<'_, AppState>) -> Result<Vec<Organization>, String> {
    let core = lock_core(&state)?;
    core.list_organizations().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)]
pub async fn update_organization(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    email: Option<Option<String>>,
    phone: Option<Option<String>>,
    website: Option<Option<String>>,
    address_line1: Option<Option<String>>,
    address_line2: Option<Option<String>>,
    city: Option<Option<String>>,
    region: Option<Option<String>>,
    country: Option<Option<String>>,
    postal_code: Option<Option<String>>,
    description: Option<Option<String>>,
    owner: Option<String>,
    reset_owner: Option<bool>,
) -> Result<Organization, String> {
    let mut core = lock_core(&state)?;
    let organization = core
        .update_organization(
            &id,
            name,
            email,
            phone,
            website,
            address_line1,
            address_line2,
            city,
            region,
            country,
            postal_code,
            description,
        )
        .map_err(|e| e.to_string())?;
    if reset_owner.unwrap_or(false) {
        return core
            .set_organization_owner(&organization.id, None)
            .map_err(|e| e.to_string());
    }
    if let Some(owner) = owner {
        return core
            .set_organization_owner(&organization.id, Some(owner.as_str()))
            .map_err(|e| e.to_string());
    }
    Ok(organization)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_organization(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut core = lock_core(&state)?;
    core.delete_organization(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn link_contact_to_organization(
    state: State<'_, AppState>,
    contact_id: String,
    organization_id: Option<String>,
) -> Result<Contact, String> {
    let mut core = lock_core(&state)?;
    core.link_contact_to_organization(&contact_id, organization_id)
        .map_err(|e| e.to_string())
}
