use crm_core::{services::TagColorUpdate, storage::tags::Tag};
use tauri::State;

use crate::{commands::lock_core, AppState};

#[tauri::command(rename_all = "snake_case")]
pub async fn create_tag(
    state: State<'_, AppState>,
    name: String,
    color: Option<String>,
) -> Result<Tag, String> {
    let mut core = lock_core(&state)?;
    core.create_tag(name, color).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_tag(state: State<'_, AppState>, id: String) -> Result<Tag, String> {
    let core = lock_core(&state)?;
    core.get_tag(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let core = lock_core(&state)?;
    core.list_tags().map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_tag(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    color: Option<String>,
    reset_color: Option<bool>,
) -> Result<Tag, String> {
    let mut core = lock_core(&state)?;
    core.update_tag(&id, name, color_update_from_args(color, reset_color))
        .map_err(|e| e.to_string())
}

pub(crate) fn color_update_from_args(
    color: Option<String>,
    reset_color: Option<bool>,
) -> Option<TagColorUpdate> {
    if reset_color.unwrap_or(false) {
        return Some(TagColorUpdate::Reset);
    }

    color.map(|value| {
        if value.trim().is_empty() {
            TagColorUpdate::Reset
        } else {
            TagColorUpdate::Set(value)
        }
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_tag(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut core = lock_core(&state)?;
    core.delete_tag(&id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn apply_tag_to_entity(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
    tag_id: String,
) -> Result<(), String> {
    let mut core = lock_core(&state)?;
    core.apply_tag_to_entity(entity_type, entity_id, tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn remove_tag_from_entity(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
    tag_id: String,
) -> Result<(), String> {
    let mut core = lock_core(&state)?;
    core.remove_tag_from_entity(entity_type, entity_id, tag_id)
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_tags_for_entity(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<Vec<Tag>, String> {
    let core = lock_core(&state)?;
    core.list_tags_for_entity(entity_type, entity_id)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{color_update_from_args, TagColorUpdate};

    #[test]
    fn color_update_from_args_distinguishes_no_change_reset_and_set() {
        assert_eq!(color_update_from_args(None, None), None);
        assert_eq!(
            color_update_from_args(None, Some(true)),
            Some(TagColorUpdate::Reset)
        );
        assert_eq!(
            color_update_from_args(Some("   ".to_string()), None),
            Some(TagColorUpdate::Reset)
        );
        assert_eq!(
            color_update_from_args(Some("#0f766e".to_string()), None),
            Some(TagColorUpdate::Set("#0f766e".to_string()))
        );
        assert_eq!(
            color_update_from_args(Some("#0f766e".to_string()), Some(true)),
            Some(TagColorUpdate::Reset)
        );
    }
}
