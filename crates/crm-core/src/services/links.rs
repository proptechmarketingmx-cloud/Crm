use rusqlite::Connection;

use crate::audit::ACTOR_DESKTOP_APP;
use crate::result::CrmResult;
use crate::storage::{self, links::EntityLink};
use crate::utils::errors::CrmError;

use super::{record_audit_json, CrmCore};

impl CrmCore {
    pub fn create_entity_link(
        &mut self,
        entity_type: String,
        entity_id: String,
        title: Option<String>,
        kind: String,
        target: String,
    ) -> CrmResult<EntityLink> {
        let entity_type = normalize_link_entity_type(&entity_type)?;
        let entity_id = normalize_required(&entity_id, "entity_id")?;
        ensure_parent_exists(&self.db.conn, entity_type, &entity_id)?;
        let (kind, target) = normalize_link_target(&kind, &target)?;
        let title = normalize_title(title.as_deref(), &kind, &target);

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let link = storage::links::create_link(
            &tx,
            entity_type,
            &entity_id,
            &title,
            &kind,
            &target,
            &device_id,
        )?;
        storage::sync::record_change(
            &tx,
            "entity_link",
            &link.id,
            "__create__",
            None,
            Some(&link.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "create",
            Some("entity_link"),
            Some(&link.id),
            None::<&()>,
            Some(&link),
            &device_id,
        )?;
        tx.commit()?;
        Ok(link)
    }

    pub fn list_entity_links(
        &self,
        entity_type: String,
        entity_id: String,
    ) -> CrmResult<Vec<EntityLink>> {
        let entity_type = normalize_link_entity_type(&entity_type)?;
        let entity_id = normalize_required(&entity_id, "entity_id")?;
        ensure_parent_exists(&self.db.conn, entity_type, &entity_id)?;
        storage::links::list_links_for_entity(&self.db.conn, entity_type, &entity_id)
    }

    pub fn update_entity_link(
        &mut self,
        id: &str,
        title: Option<String>,
        kind: String,
        target: String,
    ) -> CrmResult<EntityLink> {
        let before = storage::links::get_link(&self.db.conn, id)?;
        let (kind, target) = normalize_link_target(&kind, &target)?;
        let title = normalize_title(title.as_deref(), &kind, &target);
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let link = storage::links::update_link(&tx, id, &title, &kind, &target)?;
        storage::sync::record_change(
            &tx,
            "entity_link",
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
            Some("entity_link"),
            Some(id),
            Some(&before),
            Some(&link),
            &device_id,
        )?;
        tx.commit()?;
        Ok(link)
    }

    pub fn delete_entity_link(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::links::get_link(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::links::soft_delete_link(&tx, id)?;
        storage::sync::record_change(
            &tx,
            "entity_link",
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
            Some("entity_link"),
            Some(id),
            Some(&before),
            Option::<&EntityLink>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }
}

fn normalize_link_entity_type(entity_type: &str) -> CrmResult<&'static str> {
    match entity_type.trim().to_ascii_lowercase().as_str() {
        "contact" => Ok("contact"),
        "organization" => Ok("organization"),
        "deal" => Ok("deal"),
        other => Err(CrmError::InvalidInput(format!(
            "Unsupported entity_type '{}'. Links attach to contact, organization, or deal",
            other
        ))),
    }
}

fn normalize_required(value: &str, field: &str) -> CrmResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CrmError::InvalidInput(format!("{field} is required")));
    }
    Ok(trimmed.to_string())
}

fn ensure_parent_exists(conn: &Connection, entity_type: &str, entity_id: &str) -> CrmResult<()> {
    match entity_type {
        "contact" => storage::contacts::get_contact(conn, entity_id).map(|_| ()),
        "organization" => storage::organizations::get_organization(conn, entity_id).map(|_| ()),
        "deal" => storage::deals::get_deal(conn, entity_id).map(|_| ()),
        _ => Err(CrmError::InvalidInput(format!(
            "Unsupported entity_type '{entity_type}'"
        ))),
    }
}

pub fn normalize_link_target(kind: &str, target: &str) -> CrmResult<(String, String)> {
    let kind = match kind.trim().to_ascii_lowercase().as_str() {
        "url" => "url",
        "path" => "path",
        other => {
            return Err(CrmError::InvalidInput(format!(
                "Invalid link kind '{other}'. Must be 'url' or 'path'"
            )))
        }
    };
    let target = target.trim();
    if target.is_empty() {
        return Err(CrmError::InvalidInput(
            "Link target is required".to_string(),
        ));
    }
    if target.chars().any(|ch| ch.is_control()) {
        return Err(CrmError::InvalidInput(
            "Link target cannot contain control characters".to_string(),
        ));
    }

    if kind == "url" {
        let lower = target.to_ascii_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return Err(CrmError::InvalidInput(
                "Website links must start with http:// or https://".to_string(),
            ));
        }
        if lower.starts_with("javascript:") || lower.starts_with("data:") {
            return Err(CrmError::InvalidInput(
                "This URL scheme is not allowed".to_string(),
            ));
        }
        Ok(("url".to_string(), target.to_string()))
    } else {
        let lower = target.to_ascii_lowercase();
        if lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("javascript:")
            || lower.starts_with("data:")
            || lower.starts_with("file:")
        {
            return Err(CrmError::InvalidInput(
                "File links must be a local path, not a URL".to_string(),
            ));
        }
        Ok(("path".to_string(), target.to_string()))
    }
}

fn normalize_title(title: Option<&str>, kind: &str, target: &str) -> String {
    if let Some(title) = title.map(str::trim).filter(|value| !value.is_empty()) {
        return title.to_string();
    }

    if kind == "url" {
        target
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or(target)
            .to_string()
    } else {
        target
            .rsplit(['/', '\\'])
            .find(|part| !part.is_empty())
            .unwrap_or(target)
            .to_string()
    }
}
