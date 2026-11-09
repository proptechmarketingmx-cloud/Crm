use crate::audit::ACTOR_DESKTOP_APP;
use crate::crm_engine::contacts as contact_engine;
use crate::result::CrmResult;
use crate::storage::{
    self,
    contacts::{Contact, ContactDuplicateCandidate, ContactListParams, ContactListResult},
};
use crate::utils::errors::CrmError;

use super::{record_audit_json, CrmCore};

impl CrmCore {
    // Preserve the existing field-level service API used by Tauri command wiring.
    #[allow(clippy::too_many_arguments)]
    pub fn create_contact(
        &mut self,
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
    ) -> CrmResult<Contact> {
        self.create_contact_with_lifecycle(
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
            None,
            None,
        )
    }

    /// Creates a contact and optionally sets lifecycle in the same transaction.
    #[allow(clippy::too_many_arguments)]
    pub fn create_contact_with_lifecycle(
        &mut self,
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
    ) -> CrmResult<Contact> {
        let input = contact_engine::ContactInput {
            contact_type: contact_type.clone(),
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            org_name: org_name.clone(),
            email: email.clone(),
            phone: phone.clone(),
            address: address.clone(),
            city: city.clone(),
            country: country.clone(),
            org_id: org_id.clone(),
            notes: notes.clone(),
        };
        contact_engine::validate_contact_for_create(&input)?;

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let contact = storage::contacts::create_contact(
            &tx,
            contact_type.as_deref().unwrap_or("person"),
            first_name.as_deref().unwrap_or(""),
            last_name.as_deref().unwrap_or(""),
            org_name.as_deref().unwrap_or(""),
            email.as_deref().unwrap_or(""),
            phone.as_deref().unwrap_or(""),
            address.as_deref().unwrap_or(""),
            city.as_deref().unwrap_or(""),
            country.as_deref().unwrap_or(""),
            org_id.as_deref(),
            notes.as_deref().unwrap_or(""),
            &device_id,
        )?;
        let contact = if let Some(lifecycle) = lifecycle.filter(|value| !value.trim().is_empty()) {
            if lifecycle != contact.lifecycle {
                storage::contacts::set_contact_lifecycle(&tx, &contact.id, &lifecycle)?
            } else {
                contact
            }
        } else {
            contact
        };
        let contact = if owner.as_ref().is_some() {
            storage::contacts::set_contact_owner(&tx, &contact.id, owner.as_deref())?
        } else {
            contact
        };
        storage::sync::record_change(
            &tx,
            "contact",
            &contact.id,
            "__create__",
            None,
            Some(&contact.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "create",
            Some("contact"),
            Some(&contact.id),
            None::<&()>,
            Some(&contact),
            &device_id,
        )?;
        tx.commit()?;
        Ok(contact)
    }

    pub fn get_contact(&self, id: &str) -> CrmResult<Contact> {
        storage::contacts::get_contact(&self.db.conn, id)
    }

    pub fn list_contacts(&self, params: Option<ContactListParams>) -> CrmResult<ContactListResult> {
        storage::contacts::list_contacts(&self.db.conn, &params.unwrap_or_default())
    }

    // Preserve the existing field-level service API used by Tauri command wiring.
    #[allow(clippy::too_many_arguments)]
    pub fn update_contact(
        &mut self,
        id: &str,
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
        owner: Option<Option<String>>,
    ) -> CrmResult<Contact> {
        if let Some(ct) = contact_type.as_deref() {
            if ct != "person" && ct != "organization" {
                return Err(CrmError::InvalidInput(format!(
                    "Invalid contact_type '{}'. Must be 'person' or 'organization'",
                    ct
                )));
            }
        }

        contact_engine::validate_email_if_present(email.as_deref())?;

        let before = storage::contacts::get_contact(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let contact = storage::contacts::update_contact(
            &tx,
            id,
            contact_type.as_deref(),
            first_name.as_deref(),
            last_name.as_deref(),
            org_name.as_deref(),
            email.as_deref(),
            phone.as_deref(),
            address.as_deref(),
            city.as_deref(),
            country.as_deref(),
            None,
            None,
            notes.as_deref(),
        )?;
        let contact = if let Some(owner) = owner {
            storage::contacts::set_contact_owner(&tx, id, owner.as_deref())?
        } else {
            contact
        };
        storage::sync::record_change(&tx, "contact", id, "__update__", None, Some(id), &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "update",
            Some("contact"),
            Some(id),
            Some(&before),
            Some(&contact),
            &device_id,
        )?;
        tx.commit()?;
        Ok(contact)
    }

    pub fn delete_contact(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::contacts::get_contact(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::contacts::soft_delete_contact(&tx, id)?;
        storage::sync::record_change(&tx, "contact", id, "__delete__", Some(id), None, &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "delete",
            Some("contact"),
            Some(id),
            Some(&before),
            Option::<&Contact>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn restore_contact(&mut self, id: &str) -> CrmResult<Contact> {
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let contact = storage::contacts::restore_contact(&tx, id)?;
        storage::sync::record_change(
            &tx,
            "contact",
            id,
            "__restore__",
            None,
            Some(id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "restore",
            Some("contact"),
            Some(id),
            None::<&Contact>,
            Some(&contact),
            &device_id,
        )?;
        tx.commit()?;
        Ok(contact)
    }

    pub fn search_contacts(&self, query: &str) -> CrmResult<Vec<Contact>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        storage::contacts::search_contacts(&self.db.conn, query)
    }

    pub fn list_contact_duplicate_candidates(&self) -> CrmResult<Vec<ContactDuplicateCandidate>> {
        storage::contacts::find_active_contact_duplicate_candidates(&self.db.conn)
    }

    pub fn merge_contacts(&mut self, target_id: &str, source_id: &str) -> CrmResult<Contact> {
        let before = storage::contacts::get_contact(&self.db.conn, target_id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let contact = contact_engine::merge_contacts(&tx, target_id, source_id, &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "merge",
            Some("contact"),
            Some(target_id),
            Some(&before),
            Some(&contact),
            &device_id,
        )?;
        tx.commit()?;
        Ok(contact)
    }

    pub fn set_contact_lifecycle(&mut self, id: &str, lifecycle: &str) -> CrmResult<Contact> {
        let before = storage::contacts::get_contact(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let contact = storage::contacts::set_contact_lifecycle(&tx, id, lifecycle)?;
        if contact.lifecycle != before.lifecycle {
            storage::sync::record_change(
                &tx,
                "contact",
                id,
                "lifecycle",
                Some(&before.lifecycle),
                Some(&contact.lifecycle),
                &device_id,
            )?;
            record_audit_json(
                &tx,
                ACTOR_DESKTOP_APP,
                "update",
                Some("contact"),
                Some(id),
                Some(&before),
                Some(&contact),
                &device_id,
            )?;
        }
        tx.commit()?;
        Ok(contact)
    }
}
