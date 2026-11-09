use crate::audit::ACTOR_DESKTOP_APP;
use crate::result::CrmResult;
use crate::storage::{
    self,
    activities::{Activity, ActivityLink, ActivityLinkEntityType},
};

use super::{record_audit_json, CrmCore};

impl CrmCore {
    pub fn list_activity_links(&self, activity_id: &str) -> CrmResult<Vec<ActivityLink>> {
        storage::activities::get_activity(&self.db.conn, activity_id)?;
        storage::activities::list_activity_links(&self.db.conn, activity_id)
    }

    pub fn list_activity_links_for_activities(
        &self,
        activity_ids: Vec<String>,
    ) -> CrmResult<Vec<ActivityLink>> {
        storage::activities::list_activity_links_for_activities(&self.db.conn, &activity_ids)
    }

    pub fn add_activity_link(
        &mut self,
        activity_id: &str,
        entity_type: &str,
        entity_id: &str,
    ) -> CrmResult<ActivityLink> {
        let entity_type = ActivityLinkEntityType::try_from(entity_type)?;
        validate_activity_link_reference(&self.db.conn, entity_type, entity_id)?;
        let before_activity = storage::activities::get_activity(&self.db.conn, activity_id)?;
        let before_link = storage::activities::get_active_activity_link(
            &self.db.conn,
            activity_id,
            entity_type,
            entity_id,
        )?;
        if let Some(link) = before_link {
            return Ok(link);
        }

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let link = storage::activities::add_activity_link(
            &tx,
            activity_id,
            entity_type,
            entity_id,
            &device_id,
        )?;

        record_activity_link_create_change(&tx, &link, &device_id)?;

        match entity_type {
            ActivityLinkEntityType::Contact => {
                remove_previous_legacy_mirror_link(
                    &tx,
                    &before_activity,
                    ActivityLinkEntityType::Contact,
                    before_activity.contact_id.as_deref(),
                    entity_id,
                    &device_id,
                )?;
                let after_activity = storage::activities::update_activity(
                    &tx,
                    activity_id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(Some(entity_id)),
                    None,
                )?;
                record_activity_mirror_change(
                    &tx,
                    &before_activity,
                    &after_activity,
                    "contact_id",
                    &device_id,
                )?;
            }
            ActivityLinkEntityType::Deal => {
                remove_previous_legacy_mirror_link(
                    &tx,
                    &before_activity,
                    ActivityLinkEntityType::Deal,
                    before_activity.deal_id.as_deref(),
                    entity_id,
                    &device_id,
                )?;
                let after_activity = storage::activities::update_activity(
                    &tx,
                    activity_id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(Some(entity_id)),
                )?;
                record_activity_mirror_change(
                    &tx,
                    &before_activity,
                    &after_activity,
                    "deal_id",
                    &device_id,
                )?;
            }
            ActivityLinkEntityType::Organization => {}
        }

        tx.commit()?;
        Ok(link)
    }

    pub fn remove_activity_link(
        &mut self,
        activity_id: &str,
        entity_type: &str,
        entity_id: &str,
    ) -> CrmResult<ActivityLink> {
        let entity_type = ActivityLinkEntityType::try_from(entity_type)?;
        let before_activity = storage::activities::get_activity(&self.db.conn, activity_id)?;

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let before_link = storage::activities::get_active_activity_link(
            &tx,
            activity_id,
            entity_type,
            entity_id,
        )?;
        let link =
            storage::activities::remove_activity_link(&tx, activity_id, entity_type, entity_id)?;
        record_activity_link_delete_change(&tx, before_link.as_ref(), &link, &device_id)?;

        match entity_type {
            ActivityLinkEntityType::Contact
                if before_activity.contact_id.as_deref() == Some(entity_id) =>
            {
                let after_activity = storage::activities::update_activity(
                    &tx,
                    activity_id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(None),
                    None,
                )?;
                record_activity_mirror_change(
                    &tx,
                    &before_activity,
                    &after_activity,
                    "contact_id",
                    &device_id,
                )?;
            }
            ActivityLinkEntityType::Deal
                if before_activity.deal_id.as_deref() == Some(entity_id) =>
            {
                let after_activity = storage::activities::update_activity(
                    &tx,
                    activity_id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(None),
                )?;
                record_activity_mirror_change(
                    &tx,
                    &before_activity,
                    &after_activity,
                    "deal_id",
                    &device_id,
                )?;
            }
            _ => {}
        }

        tx.commit()?;
        Ok(link)
    }
}

pub(super) fn create_activity_links_for_legacy_mirrors(
    conn: &rusqlite::Connection,
    activity: &Activity,
    device_id: &str,
) -> CrmResult<()> {
    if let Some(contact_id) = activity.contact_id.as_deref() {
        let link = storage::activities::add_activity_link(
            conn,
            &activity.id,
            ActivityLinkEntityType::Contact,
            contact_id,
            device_id,
        )?;
        record_activity_link_create_change(conn, &link, device_id)?;
    }

    if let Some(deal_id) = activity.deal_id.as_deref() {
        let link = storage::activities::add_activity_link(
            conn,
            &activity.id,
            ActivityLinkEntityType::Deal,
            deal_id,
            device_id,
        )?;
        record_activity_link_create_change(conn, &link, device_id)?;
    }

    Ok(())
}

pub(super) fn sync_activity_links_after_mirror_update(
    conn: &rusqlite::Connection,
    before: &Activity,
    after: &Activity,
    device_id: &str,
) -> CrmResult<()> {
    sync_one_legacy_mirror(
        conn,
        before,
        after,
        ActivityLinkEntityType::Contact,
        before.contact_id.as_deref(),
        after.contact_id.as_deref(),
        "contact_id",
        device_id,
    )?;
    sync_one_legacy_mirror(
        conn,
        before,
        after,
        ActivityLinkEntityType::Deal,
        before.deal_id.as_deref(),
        after.deal_id.as_deref(),
        "deal_id",
        device_id,
    )
}

pub(super) fn add_activity_link_in_transaction(
    conn: &rusqlite::Connection,
    activity_id: &str,
    entity_type: ActivityLinkEntityType,
    entity_id: &str,
    device_id: &str,
) -> CrmResult<ActivityLink> {
    storage::activities::get_activity(conn, activity_id)?;
    validate_activity_link_reference(conn, entity_type, entity_id)?;
    let before_link =
        storage::activities::get_active_activity_link(conn, activity_id, entity_type, entity_id)?;
    let link = storage::activities::add_activity_link(
        conn,
        activity_id,
        entity_type,
        entity_id,
        device_id,
    )?;
    if before_link.is_none() {
        record_activity_link_create_change(conn, &link, device_id)?;
    }
    Ok(link)
}

// Legacy mirror sync needs both relationship endpoints and the mirrored field name.
#[allow(clippy::too_many_arguments)]
fn sync_one_legacy_mirror(
    conn: &rusqlite::Connection,
    before: &Activity,
    after: &Activity,
    entity_type: ActivityLinkEntityType,
    before_entity_id: Option<&str>,
    after_entity_id: Option<&str>,
    field_name: &str,
    device_id: &str,
) -> CrmResult<()> {
    if before_entity_id == after_entity_id {
        return Ok(());
    }

    if let Some(previous_entity_id) = before_entity_id {
        if let Some(before_link) = storage::activities::get_active_activity_link(
            conn,
            &after.id,
            entity_type,
            previous_entity_id,
        )? {
            let removed = storage::activities::remove_activity_link(
                conn,
                &after.id,
                entity_type,
                previous_entity_id,
            )?;
            record_activity_link_delete_change(conn, Some(&before_link), &removed, device_id)?;
        }
    }

    if let Some(next_entity_id) = after_entity_id {
        let before_link = storage::activities::get_active_activity_link(
            conn,
            &after.id,
            entity_type,
            next_entity_id,
        )?;
        let link = storage::activities::add_activity_link(
            conn,
            &after.id,
            entity_type,
            next_entity_id,
            device_id,
        )?;
        if before_link.is_none() {
            record_activity_link_create_change(conn, &link, device_id)?;
        }
    }

    record_activity_mirror_change(conn, before, after, field_name, device_id)
}

fn remove_previous_legacy_mirror_link(
    conn: &rusqlite::Connection,
    activity: &Activity,
    entity_type: ActivityLinkEntityType,
    previous_entity_id: Option<&str>,
    next_entity_id: &str,
    device_id: &str,
) -> CrmResult<()> {
    let Some(previous_entity_id) = previous_entity_id else {
        return Ok(());
    };
    if previous_entity_id == next_entity_id {
        return Ok(());
    }
    if let Some(before_link) = storage::activities::get_active_activity_link(
        conn,
        &activity.id,
        entity_type,
        previous_entity_id,
    )? {
        let removed = storage::activities::remove_activity_link(
            conn,
            &activity.id,
            entity_type,
            previous_entity_id,
        )?;
        record_activity_link_delete_change(conn, Some(&before_link), &removed, device_id)?;
    }
    Ok(())
}

fn validate_activity_link_reference(
    conn: &rusqlite::Connection,
    entity_type: ActivityLinkEntityType,
    entity_id: &str,
) -> CrmResult<()> {
    match entity_type {
        ActivityLinkEntityType::Contact => {
            storage::contacts::get_contact(conn, entity_id).map(|_| ())
        }
        ActivityLinkEntityType::Organization => {
            storage::organizations::get_organization(conn, entity_id).map(|_| ())
        }
        ActivityLinkEntityType::Deal => storage::deals::get_deal(conn, entity_id).map(|_| ()),
    }
}

fn record_activity_link_create_change(
    conn: &rusqlite::Connection,
    link: &ActivityLink,
    device_id: &str,
) -> CrmResult<()> {
    storage::sync::record_change(
        conn,
        "activity_link",
        &link.id,
        "__create__",
        None,
        Some(&link.id),
        device_id,
    )?;
    record_audit_json(
        conn,
        ACTOR_DESKTOP_APP,
        link_action("link", link.entity_type),
        Some("activity_link"),
        Some(&link.id),
        None::<&()>,
        Some(link),
        device_id,
    )?;
    Ok(())
}

fn record_activity_link_delete_change(
    conn: &rusqlite::Connection,
    before: Option<&ActivityLink>,
    after: &ActivityLink,
    device_id: &str,
) -> CrmResult<()> {
    storage::sync::record_change(
        conn,
        "activity_link",
        &after.id,
        "__delete__",
        Some(&after.id),
        None,
        device_id,
    )?;
    record_audit_json(
        conn,
        ACTOR_DESKTOP_APP,
        link_action("unlink", after.entity_type),
        Some("activity_link"),
        Some(&after.id),
        before,
        Some(after),
        device_id,
    )?;
    Ok(())
}

fn record_activity_mirror_change(
    conn: &rusqlite::Connection,
    before: &Activity,
    after: &Activity,
    field_name: &str,
    device_id: &str,
) -> CrmResult<()> {
    let (old_value, new_value) = match field_name {
        "contact_id" if before.contact_id != after.contact_id => {
            (before.contact_id.as_deref(), after.contact_id.as_deref())
        }
        "deal_id" if before.deal_id != after.deal_id => {
            (before.deal_id.as_deref(), after.deal_id.as_deref())
        }
        _ => return Ok(()),
    };

    storage::sync::record_change(
        conn, "activity", &after.id, field_name, old_value, new_value, device_id,
    )?;
    record_audit_json(
        conn,
        ACTOR_DESKTOP_APP,
        "update_activity_relationship",
        Some("activity"),
        Some(&after.id),
        Some(before),
        Some(after),
        device_id,
    )?;
    Ok(())
}

fn link_action(prefix: &str, entity_type: ActivityLinkEntityType) -> &'static str {
    match (prefix, entity_type) {
        ("link", ActivityLinkEntityType::Contact) => "link_contact",
        ("link", ActivityLinkEntityType::Organization) => "link_organization",
        ("link", ActivityLinkEntityType::Deal) => "link_deal",
        ("unlink", ActivityLinkEntityType::Contact) => "unlink_contact",
        ("unlink", ActivityLinkEntityType::Organization) => "unlink_organization",
        ("unlink", ActivityLinkEntityType::Deal) => "unlink_deal",
        _ => "update_activity_relationship",
    }
}
