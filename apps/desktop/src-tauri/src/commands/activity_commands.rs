use crm_core::storage::activities::{Activity, ActivityLink};
use tauri::State;

use crate::AppState;

const DEFAULT_UPCOMING_ACTIVITIES_LIMIT: u32 = 10;
const MAX_UPCOMING_ACTIVITIES_LIMIT: u32 = 200;
const DEFAULT_LIST_ACTIVITIES_LIMIT: u32 = 200;
const MAX_LIST_ACTIVITIES_LIMIT: u32 = 500;

#[tauri::command(rename_all = "snake_case")]
pub async fn create_activity(
    state: State<'_, AppState>,
    activity_type: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
    contact_id: Option<String>,
    deal_id: Option<String>,
) -> Result<Activity, String> {
    let mut core = super::lock_core(&state)?;
    core.create_activity(
        activity_type,
        title,
        description,
        due_date,
        contact_id,
        deal_id,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_activity(state: State<'_, AppState>, id: String) -> Result<Activity, String> {
    let core = super::lock_core(&state)?;
    core.get_activity(&id).map_err(|e| e.to_string())
}

/// List activities. The IPC payload is always windowed. When `limit` is `None`,
/// a default page of `DEFAULT_LIST_ACTIVITIES_LIMIT` is used.
#[tauri::command(rename_all = "snake_case")]
pub async fn list_activities(
    state: State<'_, AppState>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Activity>, String> {
    let core = super::lock_core(&state)?;
    let window = list_activities_limit(limit);
    core.list_activities_windowed(window, offset.unwrap_or(0))
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_activities_for_deals(
    state: State<'_, AppState>,
    deal_ids: Vec<String>,
) -> Result<Vec<Activity>, String> {
    let core = super::lock_core(&state)?;
    core.list_activities_for_deal_ids(deal_ids)
        .map_err(|e| e.to_string())
}

fn list_activities_limit(limit: Option<u32>) -> Option<u32> {
    Some(
        limit
            .unwrap_or(DEFAULT_LIST_ACTIVITIES_LIMIT)
            .clamp(1, MAX_LIST_ACTIVITIES_LIMIT),
    )
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_activities_for_contact(
    state: State<'_, AppState>,
    contact_id: String,
) -> Result<Vec<Activity>, String> {
    let core = super::lock_core(&state)?;
    core.list_activities_for_contact(&contact_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_activities_for_deal(
    state: State<'_, AppState>,
    deal_id: String,
) -> Result<Vec<Activity>, String> {
    let core = super::lock_core(&state)?;
    core.list_activities_for_deal(&deal_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_upcoming_activities(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<Activity>, String> {
    let core = super::lock_core(&state)?;
    core.list_upcoming_activities(upcoming_activities_limit(limit))
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mark_activity_complete(
    state: State<'_, AppState>,
    id: String,
) -> Result<Activity, String> {
    let mut core = super::lock_core(&state)?;
    core.mark_activity_complete(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn mark_activity_incomplete(
    state: State<'_, AppState>,
    id: String,
) -> Result<Activity, String> {
    let mut core = super::lock_core(&state)?;
    core.mark_activity_incomplete(&id)
        .map_err(|e| e.to_string())
}

// Preserve the existing field-level IPC command shape for frontend callers.
#[allow(clippy::too_many_arguments)]
#[tauri::command(rename_all = "snake_case")]
pub async fn update_activity(
    state: State<'_, AppState>,
    id: String,
    activity_type: Option<String>,
    title: Option<String>,
    description: Option<String>,
    due_date: Option<String>,
    reset_due_date: Option<bool>,
    completed: Option<bool>,
    contact_id: Option<String>,
    reset_contact_id: Option<bool>,
    deal_id: Option<String>,
    reset_deal_id: Option<bool>,
) -> Result<Activity, String> {
    let mut core = super::lock_core(&state)?;
    core.update_activity(
        &id,
        activity_type,
        title,
        description,
        nullable_update_from_args(due_date, reset_due_date),
        completed,
        nullable_update_from_args(contact_id, reset_contact_id),
        nullable_update_from_args(deal_id, reset_deal_id),
    )
    .map_err(|e| e.to_string())
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

fn upcoming_activities_limit(limit: Option<u32>) -> u32 {
    limit
        .unwrap_or(DEFAULT_UPCOMING_ACTIVITIES_LIMIT)
        .clamp(1, MAX_UPCOMING_ACTIVITIES_LIMIT)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_activity(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut core = super::lock_core(&state)?;
    core.delete_activity(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_activity_links(
    state: State<'_, AppState>,
    activity_id: String,
) -> Result<Vec<ActivityLink>, String> {
    let core = super::lock_core(&state)?;
    core.list_activity_links(&activity_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_activity_links_for_activities(
    state: State<'_, AppState>,
    activity_ids: Vec<String>,
) -> Result<Vec<ActivityLink>, String> {
    let core = super::lock_core(&state)?;
    core.list_activity_links_for_activities(activity_ids)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn add_activity_link(
    state: State<'_, AppState>,
    activity_id: String,
    entity_type: String,
    entity_id: String,
) -> Result<ActivityLink, String> {
    let mut core = super::lock_core(&state)?;
    core.add_activity_link(&activity_id, &entity_type, &entity_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn remove_activity_link(
    state: State<'_, AppState>,
    activity_id: String,
    entity_type: String,
    entity_id: String,
) -> Result<ActivityLink, String> {
    let mut core = super::lock_core(&state)?;
    core.remove_activity_link(&activity_id, &entity_type, &entity_id)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        list_activities_limit, nullable_update_from_args, upcoming_activities_limit,
        DEFAULT_LIST_ACTIVITIES_LIMIT, DEFAULT_UPCOMING_ACTIVITIES_LIMIT,
        MAX_LIST_ACTIVITIES_LIMIT, MAX_UPCOMING_ACTIVITIES_LIMIT,
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
            nullable_update_from_args(Some("deal-1".to_string()), Some(true)),
            Some(None)
        );
    }

    #[test]
    fn upcoming_activities_limit_defaults_and_clamps_to_storage_bounds() {
        assert_eq!(
            upcoming_activities_limit(None),
            DEFAULT_UPCOMING_ACTIVITIES_LIMIT
        );
        assert_eq!(upcoming_activities_limit(Some(0)), 1);
        assert_eq!(upcoming_activities_limit(Some(25)), 25);
        assert_eq!(
            upcoming_activities_limit(Some(5_000)),
            MAX_UPCOMING_ACTIVITIES_LIMIT
        );
    }

    #[test]
    fn list_activities_limit_defaults_none_and_clamps_some_to_max() {
        assert_eq!(
            list_activities_limit(None),
            Some(DEFAULT_LIST_ACTIVITIES_LIMIT)
        );
        assert_eq!(list_activities_limit(Some(0)), Some(1));
        assert_eq!(list_activities_limit(Some(25)), Some(25));
        assert_eq!(
            list_activities_limit(Some(5_000)),
            Some(MAX_LIST_ACTIVITIES_LIMIT)
        );
    }
}
