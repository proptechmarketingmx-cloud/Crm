use rusqlite::Connection;
use serde::Serialize;

use crate::audit::ACTOR_DESKTOP_APP;
use crate::result::CrmResult;
use crate::storage::{self, notes::Note, tags::Tag};
use crate::utils::errors::CrmError;

use super::{record_audit_json, CrmCore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagColorUpdate {
    Set(String),
    Reset,
}

impl CrmCore {
    pub fn create_note(
        &mut self,
        entity_type: String,
        entity_id: String,
        content: String,
    ) -> CrmResult<Note> {
        let entity_type = normalize_entity_type(&entity_type)?;
        let entity_id = normalize_id(&entity_id, "entity_id")?;
        let content = normalize_required(&content, "Note content")?;
        ensure_entity_exists(&self.db.conn, entity_type, &entity_id)?;

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let note = storage::notes::create_note(&tx, &content, entity_type, &entity_id, &device_id)?;
        storage::sync::record_change(
            &tx,
            "note",
            &note.id,
            "__create__",
            None,
            Some(&note.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "create",
            Some("note"),
            Some(&note.id),
            None::<&()>,
            Some(&note),
            &device_id,
        )?;
        tx.commit()?;
        Ok(note)
    }

    pub fn get_note(&self, id: &str) -> CrmResult<Note> {
        storage::notes::get_note(&self.db.conn, id)
    }

    pub fn list_notes_for_entity(
        &self,
        entity_type: String,
        entity_id: String,
    ) -> CrmResult<Vec<Note>> {
        let entity_type = normalize_entity_type(&entity_type)?;
        let entity_id = normalize_id(&entity_id, "entity_id")?;
        ensure_entity_exists(&self.db.conn, entity_type, &entity_id)?;
        storage::notes::get_notes_for_entity(&self.db.conn, entity_type, &entity_id)
    }

    pub fn list_notes(&self) -> CrmResult<Vec<Note>> {
        storage::notes::list_active_notes(&self.db.conn)
    }

    pub fn update_note(&mut self, id: &str, content: String) -> CrmResult<Note> {
        let before = storage::notes::get_note(&self.db.conn, id)?;
        let content = normalize_required(&content, "Note content")?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let note = storage::notes::update_note(&tx, id, &content)?;
        storage::sync::record_change(
            &tx,
            "note",
            id,
            "__update__",
            Some(&before.content),
            Some(&note.content),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "update",
            Some("note"),
            Some(id),
            Some(&before),
            Some(&note),
            &device_id,
        )?;
        tx.commit()?;
        Ok(note)
    }

    pub fn delete_note(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::notes::get_note(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::notes::soft_delete_note(&tx, id)?;
        storage::sync::record_change(&tx, "note", id, "__delete__", Some(id), None, &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "delete",
            Some("note"),
            Some(id),
            Some(&before),
            Option::<&Note>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn create_tag(&mut self, name: String, color: Option<String>) -> CrmResult<Tag> {
        let name = normalize_required(&name, "Tag name")?;
        let color = normalize_color(color);
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let tag = storage::tags::create_tag(&tx, &name, &color, &device_id)?;
        storage::sync::record_change(
            &tx,
            "tag",
            &tag.id,
            "__create__",
            None,
            Some(&tag.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "create",
            Some("tag"),
            Some(&tag.id),
            None::<&()>,
            Some(&tag),
            &device_id,
        )?;
        tx.commit()?;
        Ok(tag)
    }

    pub fn get_tag(&self, id: &str) -> CrmResult<Tag> {
        storage::tags::get_tag(&self.db.conn, id)
    }

    pub fn list_tags(&self) -> CrmResult<Vec<Tag>> {
        storage::tags::list_tags(&self.db.conn)
    }

    pub fn update_tag(
        &mut self,
        id: &str,
        name: Option<String>,
        color: Option<TagColorUpdate>,
    ) -> CrmResult<Tag> {
        let before = storage::tags::get_tag(&self.db.conn, id)?;
        let name = match name {
            Some(value) => Some(normalize_required(&value, "Tag name")?),
            None => None,
        };
        let color = normalize_color_update(color);

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let tag = storage::tags::update_tag(&tx, id, name.as_deref(), color.as_deref())?;
        storage::sync::record_change(&tx, "tag", id, "__update__", None, Some(id), &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "update",
            Some("tag"),
            Some(id),
            Some(&before),
            Some(&tag),
            &device_id,
        )?;
        tx.commit()?;
        Ok(tag)
    }

    pub fn delete_tag(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::tags::get_tag(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::tags::soft_delete_tag(&tx, id)?;
        storage::sync::record_change(&tx, "tag", id, "__delete__", Some(id), None, &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "delete",
            Some("tag"),
            Some(id),
            Some(&before),
            Option::<&Tag>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn apply_tag_to_entity(
        &mut self,
        entity_type: String,
        entity_id: String,
        tag_id: String,
    ) -> CrmResult<()> {
        let entity_type = normalize_entity_type(&entity_type)?;
        let entity_id = normalize_id(&entity_id, "entity_id")?;
        let tag_id = normalize_id(&tag_id, "tag_id")?;
        ensure_entity_exists(&self.db.conn, entity_type, &entity_id)?;
        let tag = storage::tags::get_tag(&self.db.conn, &tag_id)?;
        let change = TagEntityChange {
            entity_type,
            entity_id: entity_id.as_str(),
            tag_id: tag.id.as_str(),
        };

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let changed =
            storage::tags::add_tag_to_entity(&tx, entity_type, &entity_id, &tag_id, &device_id)?;
        if changed {
            storage::sync::record_change(
                &tx,
                entity_type,
                &entity_id,
                "tags",
                None,
                Some(&tag_id),
                &device_id,
            )?;
            record_audit_json(
                &tx,
                ACTOR_DESKTOP_APP,
                "apply_tag",
                Some(entity_type),
                Some(&entity_id),
                None::<&()>,
                Some(&change),
                &device_id,
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn remove_tag_from_entity(
        &mut self,
        entity_type: String,
        entity_id: String,
        tag_id: String,
    ) -> CrmResult<()> {
        let entity_type = normalize_entity_type(&entity_type)?;
        let entity_id = normalize_id(&entity_id, "entity_id")?;
        let tag_id = normalize_id(&tag_id, "tag_id")?;
        ensure_entity_exists(&self.db.conn, entity_type, &entity_id)?;
        let tag = storage::tags::get_tag(&self.db.conn, &tag_id)?;
        let change = TagEntityChange {
            entity_type,
            entity_id: entity_id.as_str(),
            tag_id: tag.id.as_str(),
        };

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let changed = storage::tags::remove_tag_from_entity(&tx, entity_type, &entity_id, &tag_id)?;
        if changed {
            storage::sync::record_change(
                &tx,
                entity_type,
                &entity_id,
                "tags",
                Some(&tag_id),
                None,
                &device_id,
            )?;
            record_audit_json(
                &tx,
                ACTOR_DESKTOP_APP,
                "remove_tag",
                Some(entity_type),
                Some(&entity_id),
                Some(&change),
                Option::<&TagEntityChange<'_>>::None,
                &device_id,
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn list_tags_for_entity(
        &self,
        entity_type: String,
        entity_id: String,
    ) -> CrmResult<Vec<Tag>> {
        let entity_type = normalize_entity_type(&entity_type)?;
        let entity_id = normalize_id(&entity_id, "entity_id")?;
        ensure_entity_exists(&self.db.conn, entity_type, &entity_id)?;
        storage::tags::get_tags_for_entity(&self.db.conn, entity_type, &entity_id)
    }
}

#[derive(Serialize)]
struct TagEntityChange<'a> {
    entity_type: &'a str,
    entity_id: &'a str,
    tag_id: &'a str,
}

fn normalize_entity_type(entity_type: &str) -> CrmResult<&'static str> {
    match entity_type.trim().to_ascii_lowercase().as_str() {
        "contact" => Ok("contact"),
        "organization" => Ok("organization"),
        "deal" => Ok("deal"),
        "activity" => Ok("activity"),
        other => Err(CrmError::InvalidInput(format!(
            "Unsupported entity_type '{}'",
            other
        ))),
    }
}

fn normalize_id(value: &str, field: &str) -> CrmResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CrmError::InvalidInput(format!("{} is required", field)));
    }
    Ok(trimmed.to_string())
}

fn normalize_required(value: &str, label: &str) -> CrmResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CrmError::InvalidInput(format!("{} is required", label)));
    }
    Ok(trimmed.to_string())
}

fn normalize_color(color: Option<String>) -> String {
    color
        .and_then(|value| {
            let trimmed = value.trim().to_string();
            (!trimmed.is_empty()).then_some(trimmed)
        })
        .unwrap_or_else(|| storage::tags::DEFAULT_TAG_COLOR.to_string())
}

fn normalize_color_update(color: Option<TagColorUpdate>) -> Option<String> {
    color.map(|update| match update {
        TagColorUpdate::Set(value) => normalize_color(Some(value)),
        TagColorUpdate::Reset => storage::tags::DEFAULT_TAG_COLOR.to_string(),
    })
}

fn ensure_entity_exists(conn: &Connection, entity_type: &str, entity_id: &str) -> CrmResult<()> {
    match entity_type {
        "contact" => storage::contacts::get_contact(conn, entity_id).map(|_| ()),
        "organization" => storage::organizations::get_organization(conn, entity_id).map(|_| ()),
        "deal" => storage::deals::get_deal(conn, entity_id).map(|_| ()),
        "activity" => storage::activities::get_activity(conn, entity_id).map(|_| ()),
        _ => Err(CrmError::InvalidInput(format!(
            "Unsupported entity_type '{}'",
            entity_type
        ))),
    }
}
