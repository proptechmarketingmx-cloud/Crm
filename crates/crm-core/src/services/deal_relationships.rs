use crate::audit::ACTOR_DESKTOP_APP;
use crate::result::CrmResult;
use crate::storage::{
    self,
    deals::{Deal, DealContact},
};

use super::{normalize_optional_string, record_audit_json, CrmCore};

impl CrmCore {
    pub fn list_deal_contacts(&self, deal_id: &str) -> CrmResult<Vec<DealContact>> {
        storage::deals::get_deal(&self.db.conn, deal_id)?;
        storage::deals::list_deal_contacts(&self.db.conn, deal_id)
    }

    pub fn add_deal_contact(
        &mut self,
        deal_id: &str,
        contact_id: &str,
        role: Option<String>,
        is_primary: bool,
    ) -> CrmResult<DealContact> {
        let role = normalize_optional_string(role);
        let before_deal = storage::deals::get_deal(&self.db.conn, deal_id)?;
        storage::contacts::get_contact(&self.db.conn, contact_id)?;
        let before_links = storage::deals::list_deal_contacts(&self.db.conn, deal_id)?;
        let before_link = before_links
            .iter()
            .find(|link| link.contact_id == contact_id)
            .cloned();

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let deal_contact = storage::deals::add_deal_contact(
            &tx,
            deal_id,
            contact_id,
            role.as_deref(),
            is_primary,
            &device_id,
        )?;
        let after_deal = storage::deals::get_deal(&tx, deal_id)?;

        record_deal_contact_link_change(&tx, before_link.as_ref(), &deal_contact, &device_id)?;

        if is_primary {
            record_deal_contact_primary_demotions(&tx, &before_links, &deal_contact, &device_id)?;
        }

        record_deal_contact_mirror_change(&tx, &before_deal, &after_deal, &device_id)?;
        tx.commit()?;
        Ok(deal_contact)
    }

    pub fn remove_deal_contact(
        &mut self,
        deal_id: &str,
        contact_id: &str,
    ) -> CrmResult<DealContact> {
        let before_deal = storage::deals::get_deal(&self.db.conn, deal_id)?;
        storage::contacts::get_contact(&self.db.conn, contact_id)?;
        let before_link = storage::deals::list_deal_contacts(&self.db.conn, deal_id)?
            .into_iter()
            .find(|link| link.contact_id == contact_id);

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let deal_contact = storage::deals::remove_deal_contact(&tx, deal_id, contact_id)?;
        let after_deal = storage::deals::get_deal(&tx, deal_id)?;

        record_deal_contact_unlink_change(&tx, before_link.as_ref(), &deal_contact, &device_id)?;

        record_deal_contact_mirror_change(&tx, &before_deal, &after_deal, &device_id)?;
        tx.commit()?;
        Ok(deal_contact)
    }

    pub fn link_deal_to_organization(
        &mut self,
        deal_id: &str,
        organization_id: Option<String>,
    ) -> CrmResult<crate::storage::deals::Deal> {
        let organization_id = normalize_optional_string(organization_id);
        let before = storage::deals::get_deal(&self.db.conn, deal_id)?;
        if let Some(organization_id) = organization_id.as_deref() {
            storage::organizations::get_organization(&self.db.conn, organization_id)?;
        }

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let deal =
            storage::deals::link_deal_to_organization(&tx, deal_id, organization_id.as_deref())?;
        storage::sync::record_change(
            &tx,
            "deal",
            deal_id,
            "organization_id",
            before.organization_id.as_deref(),
            deal.organization_id.as_deref(),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            if organization_id.is_some() {
                "link_organization"
            } else {
                "unlink_organization"
            },
            Some("deal"),
            Some(deal_id),
            Some(&before),
            Some(&deal),
            &device_id,
        )?;
        tx.commit()?;
        Ok(deal)
    }
}

pub(super) fn create_primary_deal_contact_for_mirror(
    conn: &rusqlite::Connection,
    deal: &Deal,
    device_id: &str,
) -> CrmResult<()> {
    let Some(contact_id) = deal.contact_id.as_deref() else {
        return Ok(());
    };

    let deal_contact =
        storage::deals::add_deal_contact(conn, &deal.id, contact_id, None, true, device_id)?;
    record_deal_contact_link_change(conn, None, &deal_contact, device_id)
}

pub(super) fn sync_primary_deal_contact_after_mirror_update(
    conn: &rusqlite::Connection,
    before: &Deal,
    after: &Deal,
    device_id: &str,
) -> CrmResult<()> {
    if before.contact_id == after.contact_id {
        return Ok(());
    }

    let before_links = storage::deals::list_deal_contacts(conn, &after.id)?;
    match after.contact_id.as_deref() {
        Some(contact_id) => {
            let before_link = before_links
                .iter()
                .find(|link| link.contact_id == contact_id)
                .cloned();
            let deal_contact = storage::deals::add_deal_contact(
                conn, &after.id, contact_id, None, true, device_id,
            )?;
            record_deal_contact_link_change(conn, before_link.as_ref(), &deal_contact, device_id)?;
            record_deal_contact_primary_demotions(conn, &before_links, &deal_contact, device_id)?;
        }
        None => {
            if let Some(previous_contact_id) = before.contact_id.as_deref() {
                let before_link = before_links
                    .iter()
                    .find(|link| link.contact_id == previous_contact_id && link.is_primary)
                    .cloned();
                if before_link.is_some() {
                    let deal_contact =
                        storage::deals::remove_deal_contact(conn, &after.id, previous_contact_id)?;
                    record_deal_contact_unlink_change(
                        conn,
                        before_link.as_ref(),
                        &deal_contact,
                        device_id,
                    )?;
                }
            }
        }
    }

    let after = storage::deals::get_deal(conn, &after.id)?;
    record_deal_contact_mirror_change(conn, before, &after, device_id)
}

fn record_deal_contact_link_change(
    conn: &rusqlite::Connection,
    before: Option<&DealContact>,
    after: &DealContact,
    device_id: &str,
) -> CrmResult<()> {
    let field_name = if before.is_some() {
        "__update__"
    } else {
        "__create__"
    };
    storage::sync::record_change(
        conn,
        "deal_contact",
        &after.id,
        field_name,
        before.map(|link| link.id.as_str()),
        Some(&after.id),
        device_id,
    )?;
    record_audit_json(
        conn,
        ACTOR_DESKTOP_APP,
        "link_contact",
        Some("deal_contact"),
        Some(&after.id),
        before,
        Some(after),
        device_id,
    )?;
    Ok(())
}

fn record_deal_contact_unlink_change(
    conn: &rusqlite::Connection,
    before: Option<&DealContact>,
    after: &DealContact,
    device_id: &str,
) -> CrmResult<()> {
    storage::sync::record_change(
        conn,
        "deal_contact",
        &after.id,
        "__delete__",
        Some(&after.id),
        None,
        device_id,
    )?;
    record_audit_json(
        conn,
        ACTOR_DESKTOP_APP,
        "unlink_contact",
        Some("deal_contact"),
        Some(&after.id),
        before,
        Some(after),
        device_id,
    )?;
    Ok(())
}

fn record_deal_contact_primary_demotions(
    conn: &rusqlite::Connection,
    before_links: &[DealContact],
    primary_after: &DealContact,
    device_id: &str,
) -> CrmResult<()> {
    for demoted in before_links
        .iter()
        .filter(|link| link.is_primary && link.id != primary_after.id)
    {
        let after = DealContact {
            is_primary: false,
            ..demoted.clone()
        };
        storage::sync::record_change(
            conn,
            "deal_contact",
            &demoted.id,
            "is_primary",
            Some("1"),
            Some("0"),
            device_id,
        )?;
        record_audit_json(
            conn,
            ACTOR_DESKTOP_APP,
            "update_primary_contact",
            Some("deal_contact"),
            Some(&demoted.id),
            Some(demoted),
            Some(&after),
            device_id,
        )?;
    }
    Ok(())
}

fn record_deal_contact_mirror_change(
    conn: &rusqlite::Connection,
    before: &Deal,
    after: &Deal,
    device_id: &str,
) -> CrmResult<()> {
    if before.contact_id == after.contact_id {
        return Ok(());
    }

    storage::sync::record_change(
        conn,
        "deal",
        &after.id,
        "contact_id",
        before.contact_id.as_deref(),
        after.contact_id.as_deref(),
        device_id,
    )?;
    record_audit_json(
        conn,
        ACTOR_DESKTOP_APP,
        "update_primary_contact",
        Some("deal"),
        Some(&after.id),
        Some(before),
        Some(after),
        device_id,
    )?;
    Ok(())
}
