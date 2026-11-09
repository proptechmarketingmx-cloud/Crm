use serde_json::{Map, Value};

use crate::audit::ACTOR_DESKTOP_APP;
use crate::result::CrmResult;
use crate::storage::{self, saved_views::SavedView};
use crate::utils::errors::CrmError;

use super::{record_audit_json, CrmCore};

impl CrmCore {
    pub fn create_saved_view(
        &mut self,
        entity_type: String,
        name: String,
        filters_json: String,
    ) -> CrmResult<SavedView> {
        let entity_type = normalize_view_entity_type(&entity_type)?;
        let name = normalize_view_name(&name)?;
        let filters_json = canonicalize_filters_json(&filters_json)?;
        ensure_name_available(&self.db.conn, entity_type, &name, None)?;

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let view = storage::saved_views::create_saved_view(
            &tx,
            entity_type,
            &name,
            &filters_json,
            &device_id,
        )?;
        storage::sync::record_change(
            &tx,
            "saved_view",
            &view.id,
            "__create__",
            None,
            Some(&view.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "create",
            Some("saved_view"),
            Some(&view.id),
            None::<&()>,
            Some(&view),
            &device_id,
        )?;
        tx.commit()?;
        Ok(view)
    }

    pub fn list_saved_views(&self, entity_type: String) -> CrmResult<Vec<SavedView>> {
        let entity_type = normalize_view_entity_type(&entity_type)?;
        storage::saved_views::list_saved_views(&self.db.conn, entity_type)
    }

    pub fn update_saved_view(
        &mut self,
        id: &str,
        name: String,
        filters_json: String,
    ) -> CrmResult<SavedView> {
        let before = storage::saved_views::get_saved_view(&self.db.conn, id)?;
        let name = normalize_view_name(&name)?;
        let filters_json = canonicalize_filters_json(&filters_json)?;
        ensure_name_available(&self.db.conn, &before.entity_type, &name, Some(id))?;

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let view = storage::saved_views::update_saved_view(&tx, id, &name, &filters_json)?;
        storage::sync::record_change(
            &tx,
            "saved_view",
            id,
            "__update__",
            None,
            Some(id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "update",
            Some("saved_view"),
            Some(id),
            Some(&before),
            Some(&view),
            &device_id,
        )?;
        tx.commit()?;
        Ok(view)
    }

    pub fn delete_saved_view(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::saved_views::get_saved_view(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::saved_views::soft_delete_saved_view(&tx, id)?;
        storage::sync::record_change(
            &tx,
            "saved_view",
            id,
            "__delete__",
            Some(id),
            None,
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "delete",
            Some("saved_view"),
            Some(id),
            Some(&before),
            Option::<&SavedView>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }
}

fn normalize_view_entity_type(entity_type: &str) -> CrmResult<&'static str> {
    match entity_type.trim().to_ascii_lowercase().as_str() {
        "contact" => Ok("contact"),
        "organization" => Ok("organization"),
        "deal" => Ok("deal"),
        "activity" => Ok("activity"),
        "report" => Ok("report"),
        other => Err(CrmError::InvalidInput(format!(
            "Unsupported saved-view entity_type '{other}'. Must be contact, organization, deal, activity, or report"
        ))),
    }
}

fn normalize_view_name(name: &str) -> CrmResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(CrmError::InvalidInput(
            "Saved view name is required".to_string(),
        ));
    }
    if trimmed.chars().count() > 80 {
        return Err(CrmError::InvalidInput(
            "Saved view name must be 80 characters or fewer".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

fn ensure_name_available(
    conn: &rusqlite::Connection,
    entity_type: &str,
    name: &str,
    current_id: Option<&str>,
) -> CrmResult<()> {
    if let Some(existing) =
        storage::saved_views::find_active_saved_view_by_name(conn, entity_type, name)?
    {
        if current_id != Some(existing.id.as_str()) {
            return Err(CrmError::InvalidInput(format!(
                "A saved view named '{name}' already exists"
            )));
        }
    }
    Ok(())
}

fn canonicalize_filters_json(raw: &str) -> CrmResult<String> {
    let value: Value = serde_json::from_str(raw).map_err(|_| {
        CrmError::InvalidInput("Saved view filters must be a JSON object".to_string())
    })?;
    let object = value.as_object().ok_or_else(|| {
        CrmError::InvalidInput("Saved view filters must be a JSON object".to_string())
    })?;

    let mut cleaned = Map::new();
    for (key, entry) in object {
        match key.as_str() {
            "search"
            | "type"
            | "lifecycle"
            | "country"
            | "custom_field_def_id"
            | "custom_field_query"
            | "sort_by"
            | "sort_dir"
            | "attention"
            | "status"
            | "bucket"
            | "focus" => {}
            other => {
                return Err(CrmError::InvalidInput(format!(
                    "Unsupported saved-view filter '{other}'"
                )));
            }
        }

        let Some(text) = entry
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };

        match key.as_str() {
            "type"
                if !matches!(
                    text,
                    "person" | "organization" | "task" | "call" | "meeting" | "email"
                ) =>
            {
                return Err(CrmError::InvalidInput(
                    "Saved view type must be person, organization, task, call, meeting, or email"
                        .to_string(),
                ));
            }
            "lifecycle" if text != "lead" && text != "customer" => {
                return Err(CrmError::InvalidInput(
                    "Saved view lifecycle must be lead or customer".to_string(),
                ));
            }
            "status" if !matches!(text, "pending" | "completed" | "overdue") => {
                return Err(CrmError::InvalidInput(
                    "Saved view status must be pending, completed, or overdue".to_string(),
                ));
            }
            "bucket"
                if !matches!(
                    text,
                    "overdue" | "today" | "thisWeek" | "later" | "unscheduled" | "completed"
                ) =>
            {
                return Err(CrmError::InvalidInput(
                    "Saved view bucket must be overdue, today, thisWeek, later, unscheduled, or completed"
                        .to_string(),
                ));
            }
            "sort_dir" if text != "asc" && text != "desc" => {
                return Err(CrmError::InvalidInput(
                    "Saved view sort_dir must be asc or desc".to_string(),
                ));
            }
            "attention" if text != "needsFollowUp" && text != "stale" && text != "overdue" => {
                return Err(CrmError::InvalidInput(
                    "Saved view attention must be needsFollowUp, stale, or overdue".to_string(),
                ));
            }
            "focus" if !matches!(text, "pipeline" | "activity" | "stale") => {
                return Err(CrmError::InvalidInput(
                    "Saved view focus must be pipeline, activity, or stale".to_string(),
                ));
            }
            "sort_by"
                if !matches!(
                    text,
                    "name" | "first_name" | "created_at" | "updated_at" | "createdAt" | "updatedAt"
                ) =>
            {
                return Err(CrmError::InvalidInput(
                    "Saved view sort_by is not supported".to_string(),
                ));
            }
            _ => {}
        }

        cleaned.insert(key.clone(), Value::String(text.to_string()));
    }

    serde_json::to_string(&Value::Object(cleaned)).map_err(|error| {
        CrmError::InvalidInput(format!("Saved view filters could not be stored: {error}"))
    })
}
