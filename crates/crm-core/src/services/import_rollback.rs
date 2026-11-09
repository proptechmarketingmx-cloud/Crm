use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    audit::ACTOR_DESKTOP_APP,
    storage::{
        self,
        activities::Activity,
        contacts::Contact,
        custom_fields::CustomFieldDefinition,
        deals::Deal,
        notes::Note,
        organizations::Organization,
        tags::{Tag, TargetTagLink},
    },
    utils::{errors::CrmError, uuid::new_uuid},
};

use super::{record_audit_json, CrmCore, CrmResult};

const CUSTOM_FIELD_ROLLBACK_PREFIX: &str = "custom_field:";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRollbackPlan {
    pub token: String,
    pub actions: Vec<ImportRollbackAction>,
}

impl ImportRollbackPlan {
    pub(crate) fn from_actions(actions: Vec<ImportRollbackAction>) -> Option<Self> {
        if actions.is_empty() {
            return None;
        }

        Some(Self {
            token: new_uuid(),
            actions,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "entity_type", rename_all = "snake_case")]
pub enum ImportRollbackAction {
    Contact {
        row_number: u32,
        entity_id: String,
        operation: ImportRollbackOperation,
        changed_fields: Vec<String>,
        before_import: Option<ContactImportRollbackSnapshot>,
        post_import: ContactImportRollbackSnapshot,
    },
    Deal {
        row_number: u32,
        entity_id: String,
        operation: ImportRollbackOperation,
        changed_fields: Vec<String>,
        before_import: Option<DealImportRollbackSnapshot>,
        post_import: DealImportRollbackSnapshot,
    },
    Activity {
        row_number: u32,
        entity_id: String,
        operation: ImportRollbackOperation,
        changed_fields: Vec<String>,
        before_import: Option<ActivityImportRollbackSnapshot>,
        post_import: ActivityImportRollbackSnapshot,
    },
    Organization {
        row_number: u32,
        entity_id: String,
        operation: ImportRollbackOperation,
        changed_fields: Vec<String>,
        before_import: Option<OrganizationImportRollbackSnapshot>,
        post_import: OrganizationImportRollbackSnapshot,
    },
    Note {
        row_number: u32,
        entity_id: String,
        operation: ImportRollbackOperation,
        changed_fields: Vec<String>,
        before_import: Option<NoteImportRollbackSnapshot>,
        post_import: NoteImportRollbackSnapshot,
    },
    TagDefinition {
        row_number: u32,
        entity_id: String,
        operation: ImportRollbackOperation,
        changed_fields: Vec<String>,
        before_import: Option<TagDefinitionImportRollbackSnapshot>,
        post_import: TagDefinitionImportRollbackSnapshot,
    },
    CustomFieldDefinition {
        row_number: u32,
        entity_id: String,
        operation: ImportRollbackOperation,
        changed_fields: Vec<String>,
        before_import: Option<CustomFieldDefinitionImportRollbackSnapshot>,
        post_import: CustomFieldDefinitionImportRollbackSnapshot,
    },
    TagLink {
        row_number: u32,
        entity_id: String,
        operation: ImportRollbackOperation,
        changed_fields: Vec<String>,
        before_import: Option<TagLinkImportRollbackSnapshot>,
        post_import: TagLinkImportRollbackSnapshot,
    },
}

impl ImportRollbackAction {
    pub(crate) fn created_contact(
        row_number: usize,
        contact: &Contact,
        custom_fields: BTreeMap<String, String>,
    ) -> Self {
        Self::Contact {
            row_number: row_number as u32,
            entity_id: contact.id.clone(),
            operation: ImportRollbackOperation::Created,
            changed_fields: Vec::new(),
            before_import: None,
            post_import: ContactImportRollbackSnapshot::from_contact(contact, custom_fields),
        }
    }

    pub(crate) fn merged_contact(
        row_number: usize,
        before_import: &Contact,
        post_import: &Contact,
        before_custom_fields: BTreeMap<String, String>,
        post_custom_fields: BTreeMap<String, String>,
    ) -> Option<Self> {
        let changed_fields = changed_contact_import_fields(
            before_import,
            post_import,
            &before_custom_fields,
            &post_custom_fields,
        );
        if changed_fields.is_empty() {
            return None;
        }

        Some(Self::Contact {
            row_number: row_number as u32,
            entity_id: post_import.id.clone(),
            operation: ImportRollbackOperation::Merged,
            changed_fields,
            before_import: Some(ContactImportRollbackSnapshot::from_contact(
                before_import,
                before_custom_fields,
            )),
            post_import: ContactImportRollbackSnapshot::from_contact(
                post_import,
                post_custom_fields,
            ),
        })
    }

    pub(crate) fn created_deal(
        row_number: usize,
        deal: &Deal,
        custom_fields: BTreeMap<String, String>,
    ) -> Self {
        Self::Deal {
            row_number: row_number as u32,
            entity_id: deal.id.clone(),
            operation: ImportRollbackOperation::Created,
            changed_fields: Vec::new(),
            before_import: None,
            post_import: DealImportRollbackSnapshot::from_deal(deal, custom_fields),
        }
    }

    pub(crate) fn merged_deal(
        row_number: usize,
        before_import: &Deal,
        post_import: &Deal,
        before_custom_fields: BTreeMap<String, String>,
        post_custom_fields: BTreeMap<String, String>,
    ) -> Option<Self> {
        let changed_fields = changed_deal_import_fields(
            before_import,
            post_import,
            &before_custom_fields,
            &post_custom_fields,
        );
        if changed_fields.is_empty() {
            return None;
        }

        Some(Self::Deal {
            row_number: row_number as u32,
            entity_id: post_import.id.clone(),
            operation: ImportRollbackOperation::Merged,
            changed_fields,
            before_import: Some(DealImportRollbackSnapshot::from_deal(
                before_import,
                before_custom_fields,
            )),
            post_import: DealImportRollbackSnapshot::from_deal(post_import, post_custom_fields),
        })
    }

    pub(crate) fn created_activity(
        row_number: usize,
        activity: &Activity,
        custom_fields: BTreeMap<String, String>,
    ) -> Self {
        Self::Activity {
            row_number: row_number as u32,
            entity_id: activity.id.clone(),
            operation: ImportRollbackOperation::Created,
            changed_fields: Vec::new(),
            before_import: None,
            post_import: ActivityImportRollbackSnapshot::from_activity(activity, custom_fields),
        }
    }

    pub(crate) fn created_organization(
        row_number: usize,
        organization: &Organization,
        custom_fields: BTreeMap<String, String>,
    ) -> Self {
        Self::Organization {
            row_number: row_number as u32,
            entity_id: organization.id.clone(),
            operation: ImportRollbackOperation::Created,
            changed_fields: Vec::new(),
            before_import: None,
            post_import: OrganizationImportRollbackSnapshot::from_organization(
                organization,
                custom_fields,
            ),
        }
    }

    pub(crate) fn created_note(row_number: usize, note: &Note) -> Self {
        Self::Note {
            row_number: row_number as u32,
            entity_id: note.id.clone(),
            operation: ImportRollbackOperation::Created,
            changed_fields: Vec::new(),
            before_import: None,
            post_import: NoteImportRollbackSnapshot::from(note),
        }
    }

    pub(crate) fn created_tag_definition(row_number: usize, tag: &Tag) -> Self {
        Self::TagDefinition {
            row_number: row_number as u32,
            entity_id: tag.id.clone(),
            operation: ImportRollbackOperation::Created,
            changed_fields: Vec::new(),
            before_import: None,
            post_import: TagDefinitionImportRollbackSnapshot::from(tag),
        }
    }

    pub(crate) fn created_custom_field_definition(
        row_number: usize,
        definition: &CustomFieldDefinition,
    ) -> Self {
        Self::CustomFieldDefinition {
            row_number: row_number as u32,
            entity_id: definition.id.clone(),
            operation: ImportRollbackOperation::Created,
            changed_fields: Vec::new(),
            before_import: None,
            post_import: CustomFieldDefinitionImportRollbackSnapshot::from(definition),
        }
    }

    pub(crate) fn created_tag_link(row_number: usize, link: &TargetTagLink) -> Self {
        Self::TagLink {
            row_number: row_number as u32,
            entity_id: tag_link_rollback_entity_id(
                &link.entity_type,
                &link.entity_id,
                &link.tag_id,
            ),
            operation: ImportRollbackOperation::Created,
            changed_fields: Vec::new(),
            before_import: None,
            post_import: TagLinkImportRollbackSnapshot {
                link_id: link.id.clone(),
                entity_type: link.entity_type.clone(),
                entity_id: link.entity_id.clone(),
                tag_id: link.tag_id.clone(),
                created_at: link.created_at.clone(),
            },
        }
    }

    pub(crate) fn merged_organization(
        row_number: usize,
        before_import: &Organization,
        post_import: &Organization,
        before_custom_fields: BTreeMap<String, String>,
        post_custom_fields: BTreeMap<String, String>,
    ) -> Option<Self> {
        let changed_fields = changed_organization_import_fields(
            before_import,
            post_import,
            &before_custom_fields,
            &post_custom_fields,
        );
        if changed_fields.is_empty() {
            return None;
        }

        Some(Self::Organization {
            row_number: row_number as u32,
            entity_id: post_import.id.clone(),
            operation: ImportRollbackOperation::Merged,
            changed_fields,
            before_import: Some(OrganizationImportRollbackSnapshot::from_organization(
                before_import,
                before_custom_fields,
            )),
            post_import: OrganizationImportRollbackSnapshot::from_organization(
                post_import,
                post_custom_fields,
            ),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportRollbackOperation {
    Created,
    Merged,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactImportRollbackSnapshot {
    pub contact_type: String,
    pub first_name: String,
    pub last_name: String,
    pub org_name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub city: String,
    pub country: String,
    pub org_id: Option<String>,
    pub organization_id: Option<String>,
    pub notes: String,
    pub updated_at: String,
    #[serde(default)]
    pub custom_fields: BTreeMap<String, String>,
}

impl From<&Contact> for ContactImportRollbackSnapshot {
    fn from(contact: &Contact) -> Self {
        Self::from_contact(contact, BTreeMap::new())
    }
}

impl ContactImportRollbackSnapshot {
    fn from_contact(contact: &Contact, custom_fields: BTreeMap<String, String>) -> Self {
        Self {
            contact_type: contact.contact_type.clone(),
            first_name: contact.first_name.clone(),
            last_name: contact.last_name.clone(),
            org_name: contact.org_name.clone(),
            email: contact.email.clone(),
            phone: contact.phone.clone(),
            address: contact.address.clone(),
            city: contact.city.clone(),
            country: contact.country.clone(),
            org_id: contact.org_id.clone(),
            organization_id: contact.organization_id.clone(),
            notes: contact.notes.clone(),
            updated_at: contact.updated_at.clone(),
            custom_fields,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DealImportRollbackSnapshot {
    pub title: String,
    pub value: f64,
    pub currency: String,
    pub stage: String,
    pub probability: i32,
    pub expected_close: Option<String>,
    pub contact_id: Option<String>,
    pub organization_id: Option<String>,
    pub notes: String,
    pub updated_at: String,
    #[serde(default)]
    pub custom_fields: BTreeMap<String, String>,
}

impl From<&Deal> for DealImportRollbackSnapshot {
    fn from(deal: &Deal) -> Self {
        Self::from_deal(deal, BTreeMap::new())
    }
}

impl DealImportRollbackSnapshot {
    fn from_deal(deal: &Deal, custom_fields: BTreeMap<String, String>) -> Self {
        Self {
            title: deal.title.clone(),
            value: deal.value,
            currency: deal.currency.clone(),
            stage: deal.stage.clone(),
            probability: deal.probability,
            expected_close: deal.expected_close.clone(),
            contact_id: deal.contact_id.clone(),
            organization_id: deal.organization_id.clone(),
            notes: deal.notes.clone(),
            updated_at: deal.updated_at.clone(),
            custom_fields,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityImportRollbackSnapshot {
    pub activity_type: String,
    pub title: String,
    pub description: String,
    pub due_date: Option<String>,
    pub completed: bool,
    pub contact_id: Option<String>,
    pub deal_id: Option<String>,
    pub updated_at: String,
    #[serde(default)]
    pub custom_fields: BTreeMap<String, String>,
}

impl From<&Activity> for ActivityImportRollbackSnapshot {
    fn from(activity: &Activity) -> Self {
        Self::from_activity(activity, BTreeMap::new())
    }
}

impl ActivityImportRollbackSnapshot {
    fn from_activity(activity: &Activity, custom_fields: BTreeMap<String, String>) -> Self {
        Self {
            activity_type: activity.activity_type.clone(),
            title: activity.title.clone(),
            description: activity.description.clone(),
            due_date: activity.due_date.clone(),
            completed: activity.completed,
            contact_id: activity.contact_id.clone(),
            deal_id: activity.deal_id.clone(),
            updated_at: activity.updated_at.clone(),
            custom_fields,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrganizationImportRollbackSnapshot {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub source: Option<String>,
    pub description: Option<String>,
    pub updated_at: String,
    #[serde(default)]
    pub custom_fields: BTreeMap<String, String>,
}

impl From<&Organization> for OrganizationImportRollbackSnapshot {
    fn from(organization: &Organization) -> Self {
        Self::from_organization(organization, BTreeMap::new())
    }
}

impl OrganizationImportRollbackSnapshot {
    fn from_organization(
        organization: &Organization,
        custom_fields: BTreeMap<String, String>,
    ) -> Self {
        Self {
            name: organization.name.clone(),
            email: organization.email.clone(),
            phone: organization.phone.clone(),
            website: organization.website.clone(),
            address_line1: organization.address_line1.clone(),
            address_line2: organization.address_line2.clone(),
            city: organization.city.clone(),
            region: organization.region.clone(),
            country: organization.country.clone(),
            postal_code: organization.postal_code.clone(),
            source: organization.source.clone(),
            description: organization.description.clone(),
            updated_at: organization.updated_at.clone(),
            custom_fields,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteImportRollbackSnapshot {
    pub entity_type: String,
    pub entity_id: String,
    pub content: String,
    pub updated_at: String,
}

impl From<&Note> for NoteImportRollbackSnapshot {
    fn from(note: &Note) -> Self {
        Self {
            entity_type: note.entity_type.clone(),
            entity_id: note.entity_id.clone(),
            content: note.content.clone(),
            updated_at: note.updated_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagDefinitionImportRollbackSnapshot {
    pub name: String,
    pub color: String,
    pub updated_at: String,
}

impl From<&Tag> for TagDefinitionImportRollbackSnapshot {
    fn from(tag: &Tag) -> Self {
        Self {
            name: tag.name.clone(),
            color: tag.color.clone(),
            updated_at: tag.updated_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomFieldDefinitionImportRollbackSnapshot {
    pub entity_type: String,
    pub field_name: String,
    pub field_type: String,
    pub field_options: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
}

impl From<&CustomFieldDefinition> for CustomFieldDefinitionImportRollbackSnapshot {
    fn from(definition: &CustomFieldDefinition) -> Self {
        Self {
            entity_type: definition.entity_type.clone(),
            field_name: definition.field_name.clone(),
            field_type: definition.field_type.clone(),
            field_options: definition.field_options.clone(),
            sort_order: definition.sort_order,
            created_at: definition.created_at.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagLinkImportRollbackSnapshot {
    pub link_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub tag_id: String,
    pub created_at: String,
}

impl TagLinkImportRollbackSnapshot {
    fn target_link(&self) -> TargetTagLink {
        TargetTagLink {
            id: self.link_id.clone(),
            entity_type: self.entity_type.clone(),
            entity_id: self.entity_id.clone(),
            tag_id: self.tag_id.clone(),
            created_at: self.created_at.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRollbackResult {
    pub token: String,
    pub rolled_back: u32,
    pub skipped: u32,
    pub errors: Vec<ImportRollbackRowError>,
}

impl ImportRollbackResult {
    fn new(token: String) -> Self {
        Self {
            token,
            rolled_back: 0,
            skipped: 0,
            errors: Vec::new(),
        }
    }

    fn record_rolled_back(&mut self) {
        self.rolled_back += 1;
    }

    fn record_skipped(
        &mut self,
        entity_type: &str,
        entity_id: &str,
        row_number: u32,
        code: &str,
        message: String,
    ) {
        self.skipped += 1;
        self.errors.push(ImportRollbackRowError {
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            row_number,
            code: code.to_string(),
            message,
        });
    }

    fn record_error(
        &mut self,
        entity_type: &str,
        entity_id: &str,
        row_number: u32,
        code: &str,
        message: String,
    ) {
        self.errors.push(ImportRollbackRowError {
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            row_number,
            code: code.to_string(),
            message,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportRollbackRowError {
    pub entity_type: String,
    pub entity_id: String,
    pub row_number: u32,
    pub code: String,
    pub message: String,
}

impl CrmCore {
    pub fn rollback_completed_import(
        &mut self,
        plan: &ImportRollbackPlan,
    ) -> CrmResult<ImportRollbackResult> {
        let mut result = ImportRollbackResult::new(plan.token.clone());

        for action in &plan.actions {
            match action {
                ImportRollbackAction::Contact {
                    row_number,
                    entity_id,
                    operation,
                    changed_fields,
                    before_import,
                    post_import,
                } => match operation {
                    ImportRollbackOperation::Created => self.rollback_created_contact(
                        *row_number,
                        entity_id,
                        post_import,
                        &mut result,
                    ),
                    ImportRollbackOperation::Merged => self.rollback_merged_contact(
                        *row_number,
                        entity_id,
                        changed_fields,
                        before_import.as_ref(),
                        post_import,
                        &mut result,
                    ),
                },
                ImportRollbackAction::Deal {
                    row_number,
                    entity_id,
                    operation,
                    changed_fields,
                    before_import,
                    post_import,
                } => match operation {
                    ImportRollbackOperation::Created => {
                        self.rollback_created_deal(*row_number, entity_id, post_import, &mut result)
                    }
                    ImportRollbackOperation::Merged => self.rollback_merged_deal(
                        *row_number,
                        entity_id,
                        changed_fields,
                        before_import.as_ref(),
                        post_import,
                        &mut result,
                    ),
                },
                ImportRollbackAction::Activity {
                    row_number,
                    entity_id,
                    operation,
                    post_import,
                    ..
                } => match operation {
                    ImportRollbackOperation::Created => self.rollback_created_activity(
                        *row_number,
                        entity_id,
                        post_import,
                        &mut result,
                    ),
                    ImportRollbackOperation::Merged => result.record_error(
                        "activity",
                        entity_id,
                        *row_number,
                        "invalid_plan",
                        "activity merge rollback is not supported".to_string(),
                    ),
                },
                ImportRollbackAction::Organization {
                    row_number,
                    entity_id,
                    operation,
                    changed_fields,
                    before_import,
                    post_import,
                } => match operation {
                    ImportRollbackOperation::Created => self.rollback_created_organization(
                        *row_number,
                        entity_id,
                        post_import,
                        &mut result,
                    ),
                    ImportRollbackOperation::Merged => self.rollback_merged_organization(
                        *row_number,
                        entity_id,
                        changed_fields,
                        before_import.as_ref(),
                        post_import,
                        &mut result,
                    ),
                },
                ImportRollbackAction::Note {
                    row_number,
                    entity_id,
                    operation,
                    post_import,
                    ..
                } => match operation {
                    ImportRollbackOperation::Created => {
                        self.rollback_created_note(*row_number, entity_id, post_import, &mut result)
                    }
                    ImportRollbackOperation::Merged => result.record_error(
                        "note",
                        entity_id,
                        *row_number,
                        "invalid_plan",
                        "note merge rollback is not supported".to_string(),
                    ),
                },
                ImportRollbackAction::TagDefinition {
                    row_number,
                    entity_id,
                    operation,
                    post_import,
                    ..
                } => match operation {
                    ImportRollbackOperation::Created => self.rollback_created_tag_definition(
                        *row_number,
                        entity_id,
                        post_import,
                        &mut result,
                    ),
                    ImportRollbackOperation::Merged => result.record_error(
                        "tag",
                        entity_id,
                        *row_number,
                        "invalid_plan",
                        "tag definition merge rollback is not supported".to_string(),
                    ),
                },
                ImportRollbackAction::CustomFieldDefinition {
                    row_number,
                    entity_id,
                    operation,
                    post_import,
                    ..
                } => match operation {
                    ImportRollbackOperation::Created => self
                        .rollback_created_custom_field_definition(
                            *row_number,
                            entity_id,
                            post_import,
                            &mut result,
                        ),
                    ImportRollbackOperation::Merged => result.record_error(
                        "custom_field_definition",
                        entity_id,
                        *row_number,
                        "invalid_plan",
                        "custom field definition merge rollback is not supported".to_string(),
                    ),
                },
                ImportRollbackAction::TagLink {
                    row_number,
                    entity_id,
                    operation,
                    post_import,
                    ..
                } => match operation {
                    ImportRollbackOperation::Created => self.rollback_created_tag_link(
                        *row_number,
                        entity_id,
                        post_import,
                        &mut result,
                    ),
                    ImportRollbackOperation::Merged => result.record_error(
                        "tag_link",
                        entity_id,
                        *row_number,
                        "invalid_plan",
                        "tag link merge rollback is not supported".to_string(),
                    ),
                },
            }
        }

        Ok(result)
    }

    fn rollback_created_contact(
        &mut self,
        row_number: u32,
        entity_id: &str,
        post_import: &ContactImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(current) = current_contact_for_rollback(self, row_number, entity_id, result)
        else {
            return;
        };

        let Some(current_snapshot) =
            contact_snapshot_for_rollback(self, row_number, entity_id, &current, result)
        else {
            return;
        };

        if current_snapshot != *post_import {
            record_conflict(result, "contact", entity_id, row_number);
            return;
        }

        match self.delete_contact(entity_id) {
            Ok(()) => match self.delete_custom_field_values_for_rollback("contact", entity_id) {
                Ok(()) => result.record_rolled_back(),
                Err(err) => result.record_error(
                    "contact",
                    entity_id,
                    row_number,
                    "rollback_failed",
                    err.to_string(),
                ),
            },
            Err(CrmError::NotFound(message)) => {
                result.record_skipped("contact", entity_id, row_number, "not_found", message)
            }
            Err(err) => result.record_error(
                "contact",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_merged_contact(
        &mut self,
        row_number: u32,
        entity_id: &str,
        changed_fields: &[String],
        before_import: Option<&ContactImportRollbackSnapshot>,
        post_import: &ContactImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(before_import) = before_import else {
            record_invalid_plan(result, "contact", entity_id, row_number);
            return;
        };
        let Some(current) = current_contact_for_rollback(self, row_number, entity_id, result)
        else {
            return;
        };

        let Some(current_snapshot) =
            contact_snapshot_for_rollback(self, row_number, entity_id, &current, result)
        else {
            return;
        };

        if current_snapshot != *post_import {
            record_conflict(result, "contact", entity_id, row_number);
            return;
        }

        match self.update_contact(
            entity_id,
            string_field_update(changed_fields, "contact_type", &before_import.contact_type),
            string_field_update(changed_fields, "first_name", &before_import.first_name),
            string_field_update(changed_fields, "last_name", &before_import.last_name),
            string_field_update(changed_fields, "org_name", &before_import.org_name),
            string_field_update(changed_fields, "email", &before_import.email),
            string_field_update(changed_fields, "phone", &before_import.phone),
            string_field_update(changed_fields, "address", &before_import.address),
            string_field_update(changed_fields, "city", &before_import.city),
            string_field_update(changed_fields, "country", &before_import.country),
            string_field_update(changed_fields, "notes", &before_import.notes),
            None,
        ) {
            Ok(_) => match restore_custom_field_changes(
                self,
                entity_id,
                changed_fields,
                &before_import.custom_fields,
            ) {
                Ok(()) => result.record_rolled_back(),
                Err(err) => result.record_error(
                    "contact",
                    entity_id,
                    row_number,
                    "rollback_failed",
                    err.to_string(),
                ),
            },
            Err(err) => result.record_error(
                "contact",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_created_deal(
        &mut self,
        row_number: u32,
        entity_id: &str,
        post_import: &DealImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(current) = current_deal_for_rollback(self, row_number, entity_id, result) else {
            return;
        };

        let Some(current_snapshot) =
            deal_snapshot_for_rollback(self, row_number, entity_id, &current, result)
        else {
            return;
        };

        if current_snapshot != *post_import {
            record_conflict(result, "deal", entity_id, row_number);
            return;
        }

        match self.delete_deal(entity_id) {
            Ok(()) => match self.delete_custom_field_values_for_rollback("deal", entity_id) {
                Ok(()) => result.record_rolled_back(),
                Err(err) => result.record_error(
                    "deal",
                    entity_id,
                    row_number,
                    "rollback_failed",
                    err.to_string(),
                ),
            },
            Err(CrmError::NotFound(message)) => {
                result.record_skipped("deal", entity_id, row_number, "not_found", message)
            }
            Err(err) => result.record_error(
                "deal",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_merged_deal(
        &mut self,
        row_number: u32,
        entity_id: &str,
        changed_fields: &[String],
        before_import: Option<&DealImportRollbackSnapshot>,
        post_import: &DealImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(before_import) = before_import else {
            record_invalid_plan(result, "deal", entity_id, row_number);
            return;
        };
        let Some(current) = current_deal_for_rollback(self, row_number, entity_id, result) else {
            return;
        };

        let Some(current_snapshot) =
            deal_snapshot_for_rollback(self, row_number, entity_id, &current, result)
        else {
            return;
        };

        if current_snapshot != *post_import {
            record_conflict(result, "deal", entity_id, row_number);
            return;
        }

        match self.update_deal(
            entity_id,
            string_field_update(changed_fields, "title", &before_import.title),
            f64_field_update(changed_fields, "value", before_import.value),
            string_field_update(changed_fields, "currency", &before_import.currency),
            string_field_update(changed_fields, "stage", &before_import.stage),
            i32_field_update(changed_fields, "probability", before_import.probability),
            option_string_field_update(
                changed_fields,
                "expected_close",
                &before_import.expected_close,
            ),
            option_string_field_update(changed_fields, "contact_id", &before_import.contact_id),
            option_string_field_update(
                changed_fields,
                "organization_id",
                &before_import.organization_id,
            ),
            string_field_update(changed_fields, "notes", &before_import.notes),
        ) {
            Ok(_) => match restore_custom_field_changes(
                self,
                entity_id,
                changed_fields,
                &before_import.custom_fields,
            ) {
                Ok(()) => result.record_rolled_back(),
                Err(err) => result.record_error(
                    "deal",
                    entity_id,
                    row_number,
                    "rollback_failed",
                    err.to_string(),
                ),
            },
            Err(err) => result.record_error(
                "deal",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_created_activity(
        &mut self,
        row_number: u32,
        entity_id: &str,
        post_import: &ActivityImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(current) = current_activity_for_rollback(self, row_number, entity_id, result)
        else {
            return;
        };

        let Some(current_snapshot) =
            activity_snapshot_for_rollback(self, row_number, entity_id, &current, result)
        else {
            return;
        };

        if current_snapshot != *post_import {
            record_conflict(result, "activity", entity_id, row_number);
            return;
        }

        match self.delete_activity(entity_id) {
            Ok(()) => match self.delete_custom_field_values_for_rollback("activity", entity_id) {
                Ok(()) => result.record_rolled_back(),
                Err(err) => result.record_error(
                    "activity",
                    entity_id,
                    row_number,
                    "rollback_failed",
                    err.to_string(),
                ),
            },
            Err(CrmError::NotFound(message)) => {
                result.record_skipped("activity", entity_id, row_number, "not_found", message)
            }
            Err(err) => result.record_error(
                "activity",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_created_organization(
        &mut self,
        row_number: u32,
        entity_id: &str,
        post_import: &OrganizationImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(current) = current_organization_for_rollback(self, row_number, entity_id, result)
        else {
            return;
        };

        let Some(current_snapshot) =
            organization_snapshot_for_rollback(self, row_number, entity_id, &current, result)
        else {
            return;
        };

        if current_snapshot != *post_import {
            record_conflict(result, "organization", entity_id, row_number);
            return;
        }

        match self.delete_organization(entity_id) {
            Ok(()) => match self.delete_custom_field_values_for_rollback("organization", entity_id)
            {
                Ok(()) => result.record_rolled_back(),
                Err(err) => result.record_error(
                    "organization",
                    entity_id,
                    row_number,
                    "rollback_failed",
                    err.to_string(),
                ),
            },
            Err(CrmError::NotFound(message)) => {
                result.record_skipped("organization", entity_id, row_number, "not_found", message)
            }
            Err(err) => result.record_error(
                "organization",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_created_note(
        &mut self,
        row_number: u32,
        entity_id: &str,
        post_import: &NoteImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(current) = current_note_for_rollback(self, row_number, entity_id, result) else {
            return;
        };

        let current_snapshot = NoteImportRollbackSnapshot::from(&current);
        if current_snapshot != *post_import {
            record_conflict(result, "note", entity_id, row_number);
            return;
        }

        match self.delete_note(entity_id) {
            Ok(()) => result.record_rolled_back(),
            Err(CrmError::NotFound(message)) => {
                result.record_skipped("note", entity_id, row_number, "not_found", message)
            }
            Err(err) => result.record_error(
                "note",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_created_tag_definition(
        &mut self,
        row_number: u32,
        entity_id: &str,
        post_import: &TagDefinitionImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(current) = current_tag_for_rollback(self, row_number, entity_id, result) else {
            return;
        };

        let current_snapshot = TagDefinitionImportRollbackSnapshot::from(&current);
        if current_snapshot != *post_import {
            record_conflict(result, "tag", entity_id, row_number);
            return;
        }

        match storage::tags::tag_has_active_links(&self.db.conn, entity_id) {
            Ok(true) => {
                result.record_skipped(
                    "tag",
                    entity_id,
                    row_number,
                    "conflict",
                    "created tag now has active links".to_string(),
                );
                return;
            }
            Ok(false) => {}
            Err(err) => {
                result.record_error("tag", entity_id, row_number, "read_failed", err.to_string());
                return;
            }
        }

        match self.delete_tag(entity_id) {
            Ok(()) => result.record_rolled_back(),
            Err(CrmError::NotFound(message)) => {
                result.record_skipped("tag", entity_id, row_number, "not_found", message)
            }
            Err(err) => result.record_error(
                "tag",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_created_custom_field_definition(
        &mut self,
        row_number: u32,
        entity_id: &str,
        post_import: &CustomFieldDefinitionImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(current) =
            current_custom_field_definition_for_rollback(self, row_number, entity_id, result)
        else {
            return;
        };

        let current_snapshot = CustomFieldDefinitionImportRollbackSnapshot::from(&current);
        if current_snapshot != *post_import {
            record_conflict(result, "custom_field_definition", entity_id, row_number);
            return;
        }

        match storage::custom_fields::definition_has_values(&self.db.conn, entity_id) {
            Ok(true) => {
                result.record_skipped(
                    "custom_field_definition",
                    entity_id,
                    row_number,
                    "conflict",
                    "created custom field definition now has values".to_string(),
                );
                return;
            }
            Ok(false) => {}
            Err(err) => {
                result.record_error(
                    "custom_field_definition",
                    entity_id,
                    row_number,
                    "read_failed",
                    err.to_string(),
                );
                return;
            }
        }

        match self.delete_custom_field_def(entity_id) {
            Ok(()) => result.record_rolled_back(),
            Err(CrmError::NotFound(message)) => result.record_skipped(
                "custom_field_definition",
                entity_id,
                row_number,
                "not_found",
                message,
            ),
            Err(err) => result.record_error(
                "custom_field_definition",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_created_tag_link(
        &mut self,
        row_number: u32,
        rollback_entity_id: &str,
        post_import: &TagLinkImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        match self.rollback_exact_created_tag_link(post_import) {
            Ok(false) => {
                result.record_skipped(
                    "tag_link",
                    rollback_entity_id,
                    row_number,
                    "not_found",
                    "created tag link is no longer active".to_string(),
                );
            }
            Ok(true) => result.record_rolled_back(),
            Err(err) => result.record_error(
                "tag_link",
                rollback_entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }

    fn rollback_exact_created_tag_link(
        &mut self,
        post_import: &TagLinkImportRollbackSnapshot,
    ) -> CrmResult<bool> {
        let target_link = post_import.target_link();
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let changed = storage::tags::soft_delete_exact_target_tag_link(&tx, &target_link)?;
        if !changed {
            tx.commit()?;
            return Ok(false);
        }

        storage::tags::delete_legacy_tag_link(
            &tx,
            &post_import.entity_type,
            &post_import.entity_id,
            &post_import.tag_id,
        )?;
        storage::sync::record_change(
            &tx,
            &post_import.entity_type,
            &post_import.entity_id,
            "tags",
            Some(&post_import.tag_id),
            None,
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "remove_tag",
            Some(post_import.entity_type.as_str()),
            Some(post_import.entity_id.as_str()),
            Some(post_import),
            Option::<&TagLinkImportRollbackSnapshot>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(true)
    }

    fn rollback_merged_organization(
        &mut self,
        row_number: u32,
        entity_id: &str,
        changed_fields: &[String],
        before_import: Option<&OrganizationImportRollbackSnapshot>,
        post_import: &OrganizationImportRollbackSnapshot,
        result: &mut ImportRollbackResult,
    ) {
        let Some(before_import) = before_import else {
            record_invalid_plan(result, "organization", entity_id, row_number);
            return;
        };
        let Some(current) = current_organization_for_rollback(self, row_number, entity_id, result)
        else {
            return;
        };

        let Some(current_snapshot) =
            organization_snapshot_for_rollback(self, row_number, entity_id, &current, result)
        else {
            return;
        };

        if current_snapshot != *post_import {
            record_conflict(result, "organization", entity_id, row_number);
            return;
        }

        match self.update_organization(
            entity_id,
            string_field_update(changed_fields, "name", &before_import.name),
            option_string_field_update(changed_fields, "email", &before_import.email),
            option_string_field_update(changed_fields, "phone", &before_import.phone),
            option_string_field_update(changed_fields, "website", &before_import.website),
            option_string_field_update(
                changed_fields,
                "address_line1",
                &before_import.address_line1,
            ),
            option_string_field_update(
                changed_fields,
                "address_line2",
                &before_import.address_line2,
            ),
            option_string_field_update(changed_fields, "city", &before_import.city),
            option_string_field_update(changed_fields, "region", &before_import.region),
            option_string_field_update(changed_fields, "country", &before_import.country),
            option_string_field_update(changed_fields, "postal_code", &before_import.postal_code),
            option_string_field_update(changed_fields, "description", &before_import.description),
        ) {
            Ok(_) => match restore_custom_field_changes(
                self,
                entity_id,
                changed_fields,
                &before_import.custom_fields,
            ) {
                Ok(()) => result.record_rolled_back(),
                Err(err) => result.record_error(
                    "organization",
                    entity_id,
                    row_number,
                    "rollback_failed",
                    err.to_string(),
                ),
            },
            Err(err) => result.record_error(
                "organization",
                entity_id,
                row_number,
                "rollback_failed",
                err.to_string(),
            ),
        }
    }
}

fn current_activity_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    result: &mut ImportRollbackResult,
) -> Option<Activity> {
    match core.get_activity(entity_id) {
        Ok(activity) => Some(activity),
        Err(CrmError::NotFound(message)) => {
            result.record_skipped("activity", entity_id, row_number, "not_found", message);
            None
        }
        Err(err) => {
            result.record_error(
                "activity",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn current_contact_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    result: &mut ImportRollbackResult,
) -> Option<Contact> {
    match core.get_contact(entity_id) {
        Ok(contact) => Some(contact),
        Err(CrmError::NotFound(message)) => {
            result.record_skipped("contact", entity_id, row_number, "not_found", message);
            None
        }
        Err(err) => {
            result.record_error(
                "contact",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn current_deal_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    result: &mut ImportRollbackResult,
) -> Option<Deal> {
    match core.get_deal(entity_id) {
        Ok(deal) => Some(deal),
        Err(CrmError::NotFound(message)) => {
            result.record_skipped("deal", entity_id, row_number, "not_found", message);
            None
        }
        Err(err) => {
            result.record_error(
                "deal",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn current_organization_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    result: &mut ImportRollbackResult,
) -> Option<Organization> {
    match core.get_organization(entity_id) {
        Ok(organization) => Some(organization),
        Err(CrmError::NotFound(message)) => {
            result.record_skipped("organization", entity_id, row_number, "not_found", message);
            None
        }
        Err(err) => {
            result.record_error(
                "organization",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn current_note_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    result: &mut ImportRollbackResult,
) -> Option<Note> {
    match core.get_note(entity_id) {
        Ok(note) => Some(note),
        Err(CrmError::NotFound(message)) => {
            result.record_skipped("note", entity_id, row_number, "not_found", message);
            None
        }
        Err(err) => {
            result.record_error(
                "note",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn current_tag_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    result: &mut ImportRollbackResult,
) -> Option<Tag> {
    match core.get_tag(entity_id) {
        Ok(tag) => Some(tag),
        Err(CrmError::NotFound(message)) => {
            result.record_skipped("tag", entity_id, row_number, "not_found", message);
            None
        }
        Err(err) => {
            result.record_error("tag", entity_id, row_number, "read_failed", err.to_string());
            None
        }
    }
}

fn current_custom_field_definition_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    result: &mut ImportRollbackResult,
) -> Option<CustomFieldDefinition> {
    match storage::custom_fields::get_definition(&core.db.conn, entity_id) {
        Ok(definition) => Some(definition),
        Err(CrmError::NotFound(message)) => {
            result.record_skipped(
                "custom_field_definition",
                entity_id,
                row_number,
                "not_found",
                message,
            );
            None
        }
        Err(err) => {
            result.record_error(
                "custom_field_definition",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn activity_snapshot_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    activity: &Activity,
    result: &mut ImportRollbackResult,
) -> Option<ActivityImportRollbackSnapshot> {
    match core.custom_field_snapshot("activity", entity_id) {
        Ok(custom_fields) => Some(ActivityImportRollbackSnapshot::from_activity(
            activity,
            custom_fields,
        )),
        Err(err) => {
            result.record_error(
                "activity",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn contact_snapshot_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    contact: &Contact,
    result: &mut ImportRollbackResult,
) -> Option<ContactImportRollbackSnapshot> {
    match core.custom_field_snapshot("contact", entity_id) {
        Ok(custom_fields) => Some(ContactImportRollbackSnapshot::from_contact(
            contact,
            custom_fields,
        )),
        Err(err) => {
            result.record_error(
                "contact",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn deal_snapshot_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    deal: &Deal,
    result: &mut ImportRollbackResult,
) -> Option<DealImportRollbackSnapshot> {
    match core.custom_field_snapshot("deal", entity_id) {
        Ok(custom_fields) => Some(DealImportRollbackSnapshot::from_deal(deal, custom_fields)),
        Err(err) => {
            result.record_error(
                "deal",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn organization_snapshot_for_rollback(
    core: &CrmCore,
    row_number: u32,
    entity_id: &str,
    organization: &Organization,
    result: &mut ImportRollbackResult,
) -> Option<OrganizationImportRollbackSnapshot> {
    match core.custom_field_snapshot("organization", entity_id) {
        Ok(custom_fields) => Some(OrganizationImportRollbackSnapshot::from_organization(
            organization,
            custom_fields,
        )),
        Err(err) => {
            result.record_error(
                "organization",
                entity_id,
                row_number,
                "read_failed",
                err.to_string(),
            );
            None
        }
    }
}

fn restore_custom_field_changes(
    core: &mut CrmCore,
    entity_id: &str,
    changed_fields: &[String],
    before_custom_fields: &BTreeMap<String, String>,
) -> CrmResult<()> {
    for field_def_id in changed_custom_field_ids(changed_fields) {
        if let Some(value) = before_custom_fields.get(&field_def_id) {
            core.restore_custom_field_value_for_rollback(&field_def_id, entity_id, value)?;
        } else {
            core.delete_custom_field_value_for_rollback(&field_def_id, entity_id)?;
        }
    }

    Ok(())
}

fn record_conflict(
    result: &mut ImportRollbackResult,
    entity_type: &str,
    entity_id: &str,
    row_number: u32,
) {
    result.record_skipped(
        entity_type,
        entity_id,
        row_number,
        "conflict",
        format!(
            "{} '{}' changed after import; row rollback skipped",
            entity_type, entity_id
        ),
    );
}

fn record_invalid_plan(
    result: &mut ImportRollbackResult,
    entity_type: &str,
    entity_id: &str,
    row_number: u32,
) {
    result.record_error(
        entity_type,
        entity_id,
        row_number,
        "invalid_plan",
        "rollback plan is missing before-import state for a merge action".to_string(),
    );
}

fn tag_link_rollback_entity_id(entity_type: &str, entity_id: &str, tag_id: &str) -> String {
    format!("{}:{}:{}", entity_type, entity_id, tag_id)
}

fn string_field_update(changed_fields: &[String], field: &str, value: &str) -> Option<String> {
    field_changed(changed_fields, field).then(|| value.to_string())
}

fn option_string_field_update(
    changed_fields: &[String],
    field: &str,
    value: &Option<String>,
) -> Option<Option<String>> {
    field_changed(changed_fields, field).then(|| value.clone())
}

fn f64_field_update(changed_fields: &[String], field: &str, value: f64) -> Option<f64> {
    field_changed(changed_fields, field).then_some(value)
}

fn i32_field_update(changed_fields: &[String], field: &str, value: i32) -> Option<i32> {
    field_changed(changed_fields, field).then_some(value)
}

fn field_changed(changed_fields: &[String], field: &str) -> bool {
    changed_fields.iter().any(|changed| changed == field)
}

fn changed_contact_import_fields(
    before: &Contact,
    after: &Contact,
    before_custom_fields: &BTreeMap<String, String>,
    after_custom_fields: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut fields = Vec::new();
    push_changed_string(
        &mut fields,
        "first_name",
        &before.first_name,
        &after.first_name,
    );
    push_changed_string(
        &mut fields,
        "last_name",
        &before.last_name,
        &after.last_name,
    );
    push_changed_string(&mut fields, "org_name", &before.org_name, &after.org_name);
    push_changed_string(&mut fields, "email", &before.email, &after.email);
    push_changed_string(&mut fields, "phone", &before.phone, &after.phone);
    push_changed_string(&mut fields, "address", &before.address, &after.address);
    push_changed_string(&mut fields, "city", &before.city, &after.city);
    push_changed_string(&mut fields, "country", &before.country, &after.country);
    push_changed_string(&mut fields, "notes", &before.notes, &after.notes);
    push_changed_custom_fields(&mut fields, before_custom_fields, after_custom_fields);
    fields
}

fn changed_deal_import_fields(
    before: &Deal,
    after: &Deal,
    before_custom_fields: &BTreeMap<String, String>,
    after_custom_fields: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut fields = Vec::new();
    if before.value != after.value {
        fields.push("value".to_string());
    }
    push_changed_option_string(
        &mut fields,
        "expected_close",
        &before.expected_close,
        &after.expected_close,
    );
    push_changed_string(&mut fields, "notes", &before.notes, &after.notes);
    push_changed_custom_fields(&mut fields, before_custom_fields, after_custom_fields);
    fields
}

fn changed_organization_import_fields(
    before: &Organization,
    after: &Organization,
    before_custom_fields: &BTreeMap<String, String>,
    after_custom_fields: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut fields = Vec::new();
    push_changed_option_string(&mut fields, "email", &before.email, &after.email);
    push_changed_option_string(&mut fields, "phone", &before.phone, &after.phone);
    push_changed_option_string(&mut fields, "website", &before.website, &after.website);
    push_changed_option_string(
        &mut fields,
        "address_line1",
        &before.address_line1,
        &after.address_line1,
    );
    push_changed_option_string(
        &mut fields,
        "address_line2",
        &before.address_line2,
        &after.address_line2,
    );
    push_changed_option_string(&mut fields, "city", &before.city, &after.city);
    push_changed_option_string(&mut fields, "region", &before.region, &after.region);
    push_changed_option_string(&mut fields, "country", &before.country, &after.country);
    push_changed_option_string(
        &mut fields,
        "postal_code",
        &before.postal_code,
        &after.postal_code,
    );
    push_changed_option_string(
        &mut fields,
        "description",
        &before.description,
        &after.description,
    );
    push_changed_custom_fields(&mut fields, before_custom_fields, after_custom_fields);
    fields
}

fn push_changed_string(fields: &mut Vec<String>, field: &str, before: &str, after: &str) {
    if before != after {
        fields.push(field.to_string());
    }
}

fn push_changed_option_string(
    fields: &mut Vec<String>,
    field: &str,
    before: &Option<String>,
    after: &Option<String>,
) {
    if before != after {
        fields.push(field.to_string());
    }
}

fn push_changed_custom_fields(
    fields: &mut Vec<String>,
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) {
    let field_def_ids = before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();

    for field_def_id in field_def_ids {
        if before.get(&field_def_id) != after.get(&field_def_id) {
            fields.push(format!("{CUSTOM_FIELD_ROLLBACK_PREFIX}{field_def_id}"));
        }
    }
}

fn changed_custom_field_ids(changed_fields: &[String]) -> Vec<String> {
    changed_fields
        .iter()
        .filter_map(|field| field.strip_prefix(CUSTOM_FIELD_ROLLBACK_PREFIX))
        .map(str::to_string)
        .collect()
}
