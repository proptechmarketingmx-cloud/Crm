use crm_core::storage::deals::{Deal, DealContact, PipelineSummary};
use tauri::State;

use crate::AppState;

// Preserve the existing field-level IPC command shape for frontend callers.
#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub async fn create_deal(
    state: State<'_, AppState>,
    title: String,
    value: Option<f64>,
    currency: Option<String>,
    stage: Option<String>,
    probability: Option<i32>,
    expected_close: Option<String>,
    contact_id: Option<String>,
    organization_id: Option<String>,
    notes: Option<String>,
    owner: Option<String>,
) -> Result<Deal, String> {
    let mut core = super::lock_core(&state)?;
    let deal = core
        .create_deal(
            title,
            value,
            currency,
            stage,
            probability,
            expected_close,
            contact_id,
            organization_id,
            notes,
        )
        .map_err(|e| e.to_string())?;
    if owner
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return core
            .set_deal_owner(&deal.id, owner.as_deref())
            .map_err(|e| e.to_string());
    }
    Ok(deal)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_deal(state: State<'_, AppState>, id: String) -> Result<Deal, String> {
    let core = super::lock_core(&state)?;
    core.get_deal(&id).map_err(|e| e.to_string())
}

/// List deals. The IPC payload is always windowed. When `limit` is `None`,
/// a default page of `DEFAULT_LIST_DEALS_LIMIT` is used. `limit` is clamped
/// to `MAX_LIST_DEALS_LIMIT`.
#[tauri::command(rename_all = "snake_case")]
pub async fn list_deals(
    state: State<'_, AppState>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Deal>, String> {
    let core = super::lock_core(&state)?;
    let window = list_deals_limit(limit);
    core.list_deals_windowed(window, offset.unwrap_or(0))
        .map_err(|e| e.to_string())
}

const DEFAULT_LIST_DEALS_LIMIT: u32 = 200;
const MAX_LIST_DEALS_LIMIT: u32 = 500;

fn list_deals_limit(limit: Option<u32>) -> Option<u32> {
    Some(
        limit
            .unwrap_or(DEFAULT_LIST_DEALS_LIMIT)
            .clamp(1, MAX_LIST_DEALS_LIMIT),
    )
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_deals_by_stage(
    state: State<'_, AppState>,
    stage: String,
) -> Result<Vec<Deal>, String> {
    let core = super::lock_core(&state)?;
    core.list_deals_by_stage(&stage).map_err(|e| e.to_string())
}

// Preserve the existing field-level IPC command shape for frontend callers.
#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub async fn update_deal(
    state: State<'_, AppState>,
    id: String,
    title: Option<String>,
    value: Option<f64>,
    currency: Option<String>,
    stage: Option<String>,
    probability: Option<i32>,
    expected_close: Option<String>,
    reset_expected_close: Option<bool>,
    contact_id: Option<String>,
    reset_contact_id: Option<bool>,
    organization_id: Option<String>,
    reset_organization_id: Option<bool>,
    notes: Option<String>,
    owner: Option<String>,
    reset_owner: Option<bool>,
) -> Result<Deal, String> {
    let mut core = super::lock_core(&state)?;
    let deal = core
        .update_deal(
            &id,
            title,
            value,
            currency,
            stage,
            probability,
            nullable_update_from_args(expected_close, reset_expected_close),
            nullable_update_from_args(contact_id, reset_contact_id),
            nullable_update_from_args(organization_id, reset_organization_id),
            notes,
        )
        .map_err(|e| e.to_string())?;
    if reset_owner.unwrap_or(false) {
        return core
            .set_deal_owner(&deal.id, None)
            .map_err(|e| e.to_string());
    }
    if let Some(owner) = owner {
        return core
            .set_deal_owner(&deal.id, Some(owner.as_str()))
            .map_err(|e| e.to_string());
    }
    Ok(deal)
}

pub(crate) fn nullable_update_from_args(
    value: Option<String>,
    reset: Option<bool>,
) -> Option<Option<String>> {
    if reset.unwrap_or(false) {
        return Some(None);
    }

    value.map(|value| {
        if value.trim().is_empty() {
            None
        } else {
            Some(value)
        }
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn move_deal_stage(
    state: State<'_, AppState>,
    id: String,
    stage: String,
    probability: Option<i32>,
) -> Result<Deal, String> {
    let mut core = super::lock_core(&state)?;
    core.move_deal_stage(&id, &stage, probability)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_deal(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut core = super::lock_core(&state)?;
    core.delete_deal(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn link_deal_to_organization(
    state: State<'_, AppState>,
    deal_id: String,
    organization_id: Option<String>,
) -> Result<Deal, String> {
    let mut core = super::lock_core(&state)?;
    core.link_deal_to_organization(&deal_id, organization_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn add_deal_contact(
    state: State<'_, AppState>,
    deal_id: String,
    contact_id: String,
    role: Option<String>,
    is_primary: bool,
) -> Result<DealContact, String> {
    let mut core = super::lock_core(&state)?;
    core.add_deal_contact(&deal_id, &contact_id, role, is_primary)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn remove_deal_contact(
    state: State<'_, AppState>,
    deal_id: String,
    contact_id: String,
) -> Result<DealContact, String> {
    let mut core = super::lock_core(&state)?;
    core.remove_deal_contact(&deal_id, &contact_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_deal_contacts(
    state: State<'_, AppState>,
    deal_id: String,
) -> Result<Vec<DealContact>, String> {
    let core = super::lock_core(&state)?;
    core.list_deal_contacts(&deal_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_pipeline_summary(
    state: State<'_, AppState>,
) -> Result<Vec<PipelineSummary>, String> {
    let core = super::lock_core(&state)?;
    core.get_pipeline_summary().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        list_deals_limit, nullable_update_from_args, DEFAULT_LIST_DEALS_LIMIT, MAX_LIST_DEALS_LIMIT,
    };

    #[test]
    fn nullable_update_from_args_distinguishes_no_change_reset_blank_and_set() {
        assert_eq!(nullable_update_from_args(None, None), None);
        assert_eq!(nullable_update_from_args(None, Some(false)), None);
        assert_eq!(nullable_update_from_args(None, Some(true)), Some(None));
        assert_eq!(
            nullable_update_from_args(Some("   ".to_string()), None),
            Some(None)
        );
        assert_eq!(
            nullable_update_from_args(Some("2026-07-15".to_string()), None),
            Some(Some("2026-07-15".to_string()))
        );
        assert_eq!(
            nullable_update_from_args(Some("contact-1".to_string()), None),
            Some(Some("contact-1".to_string()))
        );
        assert_eq!(
            nullable_update_from_args(Some("org-1".to_string()), None),
            Some(Some("org-1".to_string()))
        );
        assert_eq!(
            nullable_update_from_args(Some("org-1".to_string()), Some(true)),
            Some(None)
        );
    }

    #[test]
    fn list_deals_limit_defaults_none_and_clamps_some_to_max() {
        assert_eq!(list_deals_limit(None), Some(DEFAULT_LIST_DEALS_LIMIT));
        assert_eq!(list_deals_limit(Some(0)), Some(1));
        assert_eq!(list_deals_limit(Some(25)), Some(25));
        assert_eq!(list_deals_limit(Some(5_000)), Some(MAX_LIST_DEALS_LIMIT));
    }
}
