use crate::audit::ACTOR_DESKTOP_APP;
use crate::crm_engine::contacts as contact_engine;
use crate::result::CrmResult;
use crate::storage::{self, contacts::Contact, organizations::Organization};
use crate::utils::errors::CrmError;

use super::{record_audit_json, CrmCore};

impl CrmCore {
    #[allow(clippy::too_many_arguments)]
    pub fn create_organization(
        &mut self,
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
    ) -> CrmResult<Organization> {
        let name = normalize_required_name(&name)?;
        let email = normalize_optional(email);
        contact_engine::validate_email_if_present(email.as_deref())?;

        let phone = normalize_optional(phone);
        let website = normalize_optional(website);
        let address_line1 = normalize_optional(address_line1);
        let address_line2 = normalize_optional(address_line2);
        let city = normalize_optional(city);
        let region = normalize_optional(region);
        let country = normalize_optional(country);
        let postal_code = normalize_optional(postal_code);
        let description = normalize_optional(description);

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let organization = storage::organizations::create_organization(
            &tx,
            &name,
            email.as_deref(),
            phone.as_deref(),
            website.as_deref(),
            address_line1.as_deref(),
            address_line2.as_deref(),
            city.as_deref(),
            region.as_deref(),
            country.as_deref(),
            postal_code.as_deref(),
            Some("desktop"),
            description.as_deref(),
            &device_id,
        )?;
        storage::sync::record_change(
            &tx,
            "organization",
            &organization.id,
            "__create__",
            None,
            Some(&organization.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "create",
            Some("organization"),
            Some(&organization.id),
            None::<&()>,
            Some(&organization),
            &device_id,
        )?;
        tx.commit()?;
        Ok(organization)
    }

    pub fn get_organization(&self, id: &str) -> CrmResult<Organization> {
        storage::organizations::get_organization(&self.db.conn, id)
    }

    pub fn list_organizations(&self) -> CrmResult<Vec<Organization>> {
        storage::organizations::list_organizations(&self.db.conn)
    }

    pub fn set_organization_owner(
        &mut self,
        id: &str,
        owner: Option<&str>,
    ) -> CrmResult<Organization> {
        storage::organizations::set_organization_owner(&self.db.conn, id, owner)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_organization(
        &mut self,
        id: &str,
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
    ) -> CrmResult<Organization> {
        let before = storage::organizations::get_organization(&self.db.conn, id)?;
        let name = match name {
            Some(value) => Some(normalize_required_name(&value)?),
            None => None,
        };
        let email = normalize_optional_update(email);
        if let Some(Some(value)) = email.as_ref() {
            contact_engine::validate_email_if_present(Some(value))?;
        }

        let phone = normalize_optional_update(phone);
        let website = normalize_optional_update(website);
        let address_line1 = normalize_optional_update(address_line1);
        let address_line2 = normalize_optional_update(address_line2);
        let city = normalize_optional_update(city);
        let region = normalize_optional_update(region);
        let country = normalize_optional_update(country);
        let postal_code = normalize_optional_update(postal_code);
        let description = normalize_optional_update(description);

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let organization = storage::organizations::update_organization(
            &tx,
            id,
            name.as_deref(),
            email.as_ref().map(|value| value.as_deref()),
            phone.as_ref().map(|value| value.as_deref()),
            website.as_ref().map(|value| value.as_deref()),
            address_line1.as_ref().map(|value| value.as_deref()),
            address_line2.as_ref().map(|value| value.as_deref()),
            city.as_ref().map(|value| value.as_deref()),
            region.as_ref().map(|value| value.as_deref()),
            country.as_ref().map(|value| value.as_deref()),
            postal_code.as_ref().map(|value| value.as_deref()),
            None,
            description.as_ref().map(|value| value.as_deref()),
        )?;
        storage::sync::record_change(
            &tx,
            "organization",
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
            Some("organization"),
            Some(id),
            Some(&before),
            Some(&organization),
            &device_id,
        )?;
        tx.commit()?;
        Ok(organization)
    }

    pub fn delete_organization(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::organizations::get_organization(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::organizations::soft_delete_organization(&tx, id)?;
        storage::sync::record_change(
            &tx,
            "organization",
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
            Some("organization"),
            Some(id),
            Some(&before),
            Option::<&Organization>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn link_contact_to_organization(
        &mut self,
        contact_id: &str,
        organization_id: Option<String>,
    ) -> CrmResult<Contact> {
        let before = storage::contacts::get_contact(&self.db.conn, contact_id)?;
        let organization = match organization_id {
            Some(id) if !id.trim().is_empty() => Some(storage::organizations::get_organization(
                &self.db.conn,
                id.trim(),
            )?),
            _ => None,
        };

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let contact = storage::contacts::link_contact_to_organization(
            &tx,
            contact_id,
            organization.as_ref().map(|org| org.id.as_str()),
            organization.as_ref().map(|org| org.name.as_str()),
        )?;
        storage::sync::record_change(
            &tx,
            "contact",
            contact_id,
            "organization_id",
            before.organization_id.as_deref(),
            contact.organization_id.as_deref(),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "link_organization",
            Some("contact"),
            Some(contact_id),
            Some(&before),
            Some(&contact),
            &device_id,
        )?;
        tx.commit()?;
        Ok(contact)
    }
}

fn normalize_required_name(value: &str) -> CrmResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CrmError::InvalidInput(
            "organization name is required".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|trimmed| !trimmed.is_empty())
}

fn normalize_optional_update(value: Option<Option<String>>) -> Option<Option<String>> {
    value.map(normalize_optional)
}
