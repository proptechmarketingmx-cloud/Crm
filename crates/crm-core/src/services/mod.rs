use std::collections::BTreeMap;
use std::fs;
use std::io::BufWriter;
use std::path::Path;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::audit::{ACTOR_DESKTOP_APP, ACTOR_IMPORT};
use crate::crm_engine::{activities as activity_engine, deals as deal_engine, CrmEngine};
use crate::result::CrmResult;
use crate::services::activity_relationships::{
    create_activity_links_for_legacy_mirrors, sync_activity_links_after_mirror_update,
};
use crate::services::deal_relationships::{
    create_primary_deal_contact_for_mirror, sync_primary_deal_contact_after_mirror_update,
};
use crate::storage::{
    self,
    activities::Activity,
    audit::AuditLogEntry,
    contacts::ContactListParams,
    custom_fields::{
        CustomFieldDefinition, CustomFieldValue, EntityCustomFieldValue, EntityTypeCustomFieldValue,
    },
    dashboard::CurrencyPipelineValue,
    deals::{Deal, PipelineSummary},
    reporting::{ActivityFunnelReport, PipelineConversionReport},
};
use crate::utils::{
    csv::{
        parse_activities_csv_with_mapping_targets, parse_activities_csv_with_row_numbers,
        parse_activities_json_with_mapping_targets, parse_activities_json_with_row_numbers,
        parse_contacts_csv_with_mapping_targets, parse_contacts_csv_with_row_numbers,
        parse_contacts_json_with_mapping_targets, parse_contacts_json_with_row_numbers,
        parse_custom_field_definitions_csv_with_mapping,
        parse_custom_field_definitions_csv_with_row_numbers,
        parse_custom_field_definitions_json_with_mapping,
        parse_custom_field_definitions_json_with_row_numbers, parse_deals_csv_with_mapping_targets,
        parse_deals_csv_with_row_numbers, parse_deals_json_with_mapping_targets,
        parse_deals_json_with_row_numbers, parse_notes_csv_with_mapping,
        parse_notes_csv_with_row_numbers, parse_notes_json_with_mapping,
        parse_notes_json_with_row_numbers, parse_organizations_csv_with_mapping_targets,
        parse_organizations_csv_with_row_numbers, parse_organizations_json_with_mapping_targets,
        parse_organizations_json_with_row_numbers, parse_tag_definitions_csv_with_mapping,
        parse_tag_definitions_csv_with_row_numbers, parse_tag_definitions_json_with_mapping,
        parse_tag_definitions_json_with_row_numbers, parse_tag_links_csv_with_mapping,
        parse_tag_links_csv_with_row_numbers, parse_tag_links_json_with_mapping,
        parse_tag_links_json_with_row_numbers, preview_activities_json_import,
        preview_contacts_json_import, preview_custom_field_definitions_json_import,
        preview_deals_json_import, preview_notes_json_import, preview_organizations_json_import,
        preview_tag_definitions_json_import, preview_tag_links_json_import, write_activities_csv,
        write_contacts_csv, write_custom_field_definitions_csv, write_deals_csv, write_notes_csv,
        write_organizations_csv, write_tag_definitions_csv, write_tag_links_csv, ActivityCsvRow,
        ContactCsvRow, CustomFieldDefinitionCsvRow, DealCsvRow, ImportColumnMapping,
        JsonImportPreview, NoteCsvRow, OrganizationCsvRow, TagDefinitionCsvRow, TagLinkCsvRow,
        CUSTOM_FIELD_PREFIX,
    },
    datetime::now_iso8601,
    errors::{CrmError, CrmResult as InternalCrmResult},
    uuid::new_uuid,
};

mod activity_relationships;
mod audit;
mod backup;
mod contacts;
mod deal_relationships;
mod external_client_permissions;
mod external_clients;
mod import_rollback;
mod links;
mod migration_readiness;
mod notes_tags;
mod organizations;
mod proposed_actions;
mod saved_views;
mod search;
mod settings;

pub use backup::{LocalBackup, LocalBackupMetadata, LocalBackupValidation, LocalRestoreResult};
pub use import_rollback::{ImportRollbackPlan, ImportRollbackResult};
pub use migration_readiness::NormalizationMigrationPreflight;
pub use notes_tags::TagColorUpdate;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_contacts: i64,
    pub total_organizations: i64,
    pub active_deals: i64,
    pub pipeline_value: f64,
    pub pipeline_value_by_currency: Vec<CurrencyPipelineValue>,
    pub weighted_pipeline: f64,
    pub upcoming_activities: i64,
    pub overdue_activities: i64,
    pub win_rate: f64,
    pub new_contacts_this_month: i64,
    pub new_deals_this_month: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub created: u32,
    #[serde(default)]
    pub merged: u32,
    pub skipped: u32,
    pub errors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_plan: Option<ImportRollbackPlan>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ImportOptions {
    pub merge_duplicates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreflightReport {
    pub entity_type: String,
    pub total_rows: u32,
    pub duplicate_warning_count: u32,
    pub warnings: Vec<ImportDuplicateWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDuplicateWarning {
    pub entity_type: String,
    pub row_number: u32,
    pub match_type: String,
    pub csv_value: String,
    pub existing_entity_type: String,
    pub existing_entity_id: String,
    pub existing_display_label: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub state: String,
    pub last_sync_at: Option<String>,
    pub error_message: Option<String>,
    pub pending_changes: u32,
}

pub struct CrmCore {
    pub(crate) engine: CrmEngine,
    pub(crate) db: storage::Database,
    pub(crate) device_id: String,
}

impl CrmCore {
    pub fn open(app_data_dir: &Path) -> CrmResult<Self> {
        let db = storage::Database::new(app_data_dir)?;
        let device_id = load_or_create_device_id(&db)?;

        Ok(Self {
            engine: CrmEngine::new(),
            db,
            device_id,
        })
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn default_stage_count(&self) -> usize {
        self.engine.default_stages.len()
    }

    // Preserve the existing field-level service API used by Tauri command wiring.
    #[allow(clippy::too_many_arguments)]
    pub fn create_deal(
        &mut self,
        title: String,
        value: Option<f64>,
        currency: Option<String>,
        stage: Option<String>,
        probability: Option<i32>,
        expected_close: Option<String>,
        contact_id: Option<String>,
        organization_id: Option<String>,
        notes: Option<String>,
    ) -> CrmResult<Deal> {
        let contact_id = normalize_optional_string(contact_id);
        let organization_id = normalize_optional_string(organization_id);
        let input = deal_engine::DealInput {
            title: Some(title.clone()),
            value,
            currency: currency.clone(),
            stage: stage.clone(),
            probability,
            expected_close: expected_close.clone(),
            contact_id: contact_id.clone(),
            notes: notes.clone(),
        };
        deal_engine::validate_deal_for_create(&input)?;
        if let Some(contact_id) = contact_id.as_deref() {
            storage::contacts::get_contact(&self.db.conn, contact_id)?;
        }
        if let Some(organization_id) = organization_id.as_deref() {
            storage::organizations::get_organization(&self.db.conn, organization_id)?;
        }

        let prob = probability.unwrap_or_else(|| {
            let stage_name = stage.as_deref().unwrap_or("Lead");
            self.engine.default_probability(stage_name)
        });
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let deal = storage::deals::create_deal(
            &tx,
            &title,
            value.unwrap_or(0.0),
            currency.as_deref().unwrap_or("USD"),
            stage.as_deref().unwrap_or("Lead"),
            prob,
            expected_close.as_deref(),
            contact_id.as_deref(),
            organization_id.as_deref(),
            notes.as_deref().unwrap_or(""),
            &device_id,
        )?;
        create_primary_deal_contact_for_mirror(&tx, &deal, &device_id)?;
        let deal = storage::deals::get_deal(&tx, &deal.id)?;
        storage::sync::record_change(
            &tx,
            "deal",
            &deal.id,
            "__create__",
            None,
            Some(&deal.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "create",
            Some("deal"),
            Some(&deal.id),
            None::<&()>,
            Some(&deal),
            &device_id,
        )?;
        tx.commit()?;
        Ok(deal)
    }

    pub fn get_deal(&self, id: &str) -> CrmResult<Deal> {
        storage::deals::get_deal(&self.db.conn, id)
    }

    pub fn list_deals(&self) -> CrmResult<Vec<Deal>> {
        storage::deals::list_deals(&self.db.conn)
    }

    pub fn list_deals_windowed(&self, limit: Option<u32>, offset: u32) -> CrmResult<Vec<Deal>> {
        storage::deals::list_deals_windowed(&self.db.conn, limit, offset)
    }

    pub fn list_deals_by_stage(&self, stage: &str) -> CrmResult<Vec<Deal>> {
        storage::deals::list_deals_by_stage(&self.db.conn, stage)
    }

    pub fn set_deal_owner(&mut self, id: &str, owner: Option<&str>) -> CrmResult<Deal> {
        storage::deals::set_deal_owner(&self.db.conn, id, owner)
    }

    // Preserve the existing field-level service API used by Tauri command wiring.
    #[allow(clippy::too_many_arguments)]
    pub fn update_deal(
        &mut self,
        id: &str,
        title: Option<String>,
        value: Option<f64>,
        currency: Option<String>,
        stage: Option<String>,
        probability: Option<i32>,
        expected_close: Option<Option<String>>,
        contact_id: Option<Option<String>>,
        organization_id: Option<Option<String>>,
        notes: Option<String>,
    ) -> CrmResult<Deal> {
        let expected_close = normalize_optional_update_string(expected_close);
        let contact_id = normalize_optional_update_string(contact_id);
        let organization_id = organization_id.map(normalize_optional_string);
        if let Some(Some(contact_id)) = contact_id.as_ref() {
            storage::contacts::get_contact(&self.db.conn, contact_id)?;
        }
        if let Some(Some(organization_id)) = organization_id.as_ref() {
            storage::organizations::get_organization(&self.db.conn, organization_id)?;
        }
        let before = storage::deals::get_deal(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let deal = storage::deals::update_deal(
            &tx,
            id,
            title.as_deref(),
            value,
            currency.as_deref(),
            stage.as_deref(),
            probability,
            expected_close.as_ref().map(|value| value.as_deref()),
            contact_id.as_ref().map(|value| value.as_deref()),
            organization_id.as_ref().map(|value| value.as_deref()),
            notes.as_deref(),
        )?;
        sync_primary_deal_contact_after_mirror_update(&tx, &before, &deal, &device_id)?;
        let deal = storage::deals::get_deal(&tx, id)?;
        storage::sync::record_change(&tx, "deal", id, "__update__", None, Some(id), &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "update",
            Some("deal"),
            Some(id),
            Some(&before),
            Some(&deal),
            &device_id,
        )?;
        tx.commit()?;
        Ok(deal)
    }

    pub fn move_deal_stage(
        &mut self,
        id: &str,
        stage: &str,
        probability: Option<i32>,
    ) -> CrmResult<Deal> {
        let before = storage::deals::get_deal(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let deal = storage::deals::move_deal_stage(&tx, id, stage, probability)?;
        storage::sync::record_change(&tx, "deal", id, "stage", None, Some(stage), &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "move_stage",
            Some("deal"),
            Some(id),
            Some(&before),
            Some(&deal),
            &device_id,
        )?;
        tx.commit()?;
        Ok(deal)
    }

    pub fn delete_deal(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::deals::get_deal(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::deals::soft_delete_deal(&tx, id)?;
        storage::sync::record_change(&tx, "deal", id, "__delete__", Some(id), None, &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "delete",
            Some("deal"),
            Some(id),
            Some(&before),
            Option::<&Deal>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_pipeline_summary(&self) -> CrmResult<Vec<PipelineSummary>> {
        storage::deals::get_pipeline_summary(&self.db.conn)
    }

    pub fn create_activity(
        &mut self,
        activity_type: String,
        title: String,
        description: Option<String>,
        due_date: Option<String>,
        contact_id: Option<String>,
        deal_id: Option<String>,
    ) -> CrmResult<Activity> {
        activity_engine::validate_activity_for_create(&title, &activity_type)?;
        let contact_id = normalize_optional_string(contact_id);
        let deal_id = normalize_optional_string(deal_id);
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let activity = create_activity_in_transaction(
            &tx,
            &device_id,
            &activity_type,
            &title,
            description.as_deref(),
            due_date.as_deref(),
            contact_id.as_deref(),
            deal_id.as_deref(),
        )?;
        tx.commit()?;
        Ok(activity)
    }

    pub fn get_activity(&self, id: &str) -> CrmResult<Activity> {
        storage::activities::get_activity(&self.db.conn, id)
    }

    pub fn list_activities(&self) -> CrmResult<Vec<Activity>> {
        storage::activities::list_activities(&self.db.conn)
    }

    pub fn list_activities_windowed(
        &self,
        limit: Option<u32>,
        offset: u32,
    ) -> CrmResult<Vec<Activity>> {
        storage::activities::list_activities_windowed(&self.db.conn, limit, offset)
    }

    pub fn list_activities_for_deal_ids(&self, deal_ids: Vec<String>) -> CrmResult<Vec<Activity>> {
        storage::activities::list_activities_for_deal_ids(&self.db.conn, &deal_ids)
    }

    pub fn list_activities_for_contact(&self, contact_id: &str) -> CrmResult<Vec<Activity>> {
        storage::activities::list_activities_for_contact(&self.db.conn, contact_id)
    }

    pub fn list_activities_for_deal(&self, deal_id: &str) -> CrmResult<Vec<Activity>> {
        storage::activities::list_activities_for_deal(&self.db.conn, deal_id)
    }

    pub fn list_upcoming_activities(&self, limit: u32) -> CrmResult<Vec<Activity>> {
        storage::activities::list_upcoming_activities(&self.db.conn, limit)
    }

    pub fn mark_activity_complete(&mut self, id: &str) -> CrmResult<Activity> {
        self.set_activity_completion(id, true)
    }

    pub fn mark_activity_incomplete(&mut self, id: &str) -> CrmResult<Activity> {
        self.set_activity_completion(id, false)
    }

    // Preserve the existing field-level service API used by Tauri command wiring.
    #[allow(clippy::too_many_arguments)]
    pub fn update_activity(
        &mut self,
        id: &str,
        activity_type: Option<String>,
        title: Option<String>,
        description: Option<String>,
        due_date: Option<Option<String>>,
        completed: Option<bool>,
        contact_id: Option<Option<String>>,
        deal_id: Option<Option<String>>,
    ) -> CrmResult<Activity> {
        let due_date = normalize_optional_update_string(due_date);
        let contact_id = normalize_optional_update_string(contact_id);
        let deal_id = normalize_optional_update_string(deal_id);
        if let Some(Some(contact_id)) = contact_id.as_ref() {
            storage::contacts::get_contact(&self.db.conn, contact_id)?;
        }
        if let Some(Some(deal_id)) = deal_id.as_ref() {
            storage::deals::get_deal(&self.db.conn, deal_id)?;
        }
        let before = storage::activities::get_activity(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let activity = storage::activities::update_activity(
            &tx,
            id,
            activity_type.as_deref(),
            title.as_deref(),
            description.as_deref(),
            due_date.as_ref().map(|value| value.as_deref()),
            completed,
            contact_id.as_ref().map(|value| value.as_deref()),
            deal_id.as_ref().map(|value| value.as_deref()),
        )?;
        sync_activity_links_after_mirror_update(&tx, &before, &activity, &device_id)?;
        let activity = storage::activities::get_activity(&tx, id)?;
        storage::sync::record_change(
            &tx,
            "activity",
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
            Some("activity"),
            Some(id),
            Some(&before),
            Some(&activity),
            &device_id,
        )?;
        tx.commit()?;
        Ok(activity)
    }

    pub fn delete_activity(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::activities::get_activity(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::activities::soft_delete_activity(&tx, id)?;
        storage::sync::record_change(
            &tx,
            "activity",
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
            Some("activity"),
            Some(id),
            Some(&before),
            Option::<&Activity>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_dashboard_stats(&self) -> CrmResult<DashboardStats> {
        let pipeline_summaries = storage::deals::get_pipeline_summary(&self.db.conn)?;
        let active_deals = pipeline_summaries
            .iter()
            .filter(|s| s.stage != "Closed Won" && s.stage != "Closed Lost")
            .map(|s| s.count)
            .sum();
        let pipeline_value = deal_engine::calculate_total_pipeline_value(&pipeline_summaries);
        let weighted_pipeline = deal_engine::calculate_weighted_pipeline(&pipeline_summaries);
        let win_rate = deal_engine::calculate_win_rate(&pipeline_summaries);
        let activity_stats = activity_engine::get_activity_stats(&self.db.conn)?;
        let now = now_iso8601();
        let counts =
            storage::dashboard::get_dashboard_counts(&self.db.conn, &format!("{}%", &now[..7]))?;
        let pipeline_value_by_currency =
            storage::dashboard::get_pipeline_value_by_currency(&self.db.conn)?;

        Ok(DashboardStats {
            total_contacts: counts.total_contacts,
            total_organizations: counts.total_organizations,
            active_deals,
            pipeline_value,
            pipeline_value_by_currency,
            weighted_pipeline,
            upcoming_activities: activity_stats.pending,
            overdue_activities: activity_stats.overdue,
            win_rate,
            new_contacts_this_month: counts.new_contacts_this_month,
            new_deals_this_month: counts.new_deals_this_month,
        })
    }

    pub fn list_custom_field_defs(
        &self,
        entity_type: Option<String>,
    ) -> CrmResult<Vec<CustomFieldDefinition>> {
        storage::custom_fields::list_definitions(&self.db.conn, entity_type.as_deref())
    }

    pub fn create_custom_field_def(
        &mut self,
        entity_type: String,
        field_name: String,
        field_type: String,
        field_options: Option<String>,
        sort_order: Option<i32>,
    ) -> CrmResult<CustomFieldDefinition> {
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let definition = storage::custom_fields::create_definition(
            &tx,
            &entity_type,
            &field_name,
            &field_type,
            field_options.as_deref(),
            sort_order.unwrap_or(0),
        )?;
        storage::sync::record_change(
            &tx,
            "custom_field_def",
            &definition.id,
            "__create__",
            None,
            Some(&definition.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "create",
            Some("custom_field_def"),
            Some(&definition.id),
            None::<&()>,
            Some(&definition),
            &device_id,
        )?;
        tx.commit()?;
        Ok(definition)
    }

    pub fn update_custom_field_def(
        &mut self,
        id: &str,
        field_name: Option<String>,
        field_type: Option<String>,
        field_options: Option<String>,
        sort_order: Option<i32>,
    ) -> CrmResult<CustomFieldDefinition> {
        let before = storage::custom_fields::get_definition(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let definition = storage::custom_fields::update_definition(
            &tx,
            id,
            field_name.as_deref(),
            field_type.as_deref(),
            field_options.as_deref(),
            sort_order,
        )?;
        storage::sync::record_change(
            &tx,
            "custom_field_def",
            &definition.id,
            "__update__",
            None,
            Some(&definition.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "update",
            Some("custom_field_def"),
            Some(&definition.id),
            Some(&before),
            Some(&definition),
            &device_id,
        )?;
        tx.commit()?;
        Ok(definition)
    }

    pub fn delete_custom_field_def(&mut self, id: &str) -> CrmResult<()> {
        let before = storage::custom_fields::get_definition(&self.db.conn, id)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        storage::custom_fields::delete_definition(&tx, id)?;
        storage::sync::record_change(
            &tx,
            "custom_field_def",
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
            Some("custom_field_def"),
            Some(id),
            Some(&before),
            Option::<&CustomFieldDefinition>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn set_custom_field_value(
        &mut self,
        field_def_id: String,
        entity_id: String,
        value: String,
    ) -> CrmResult<CustomFieldValue> {
        let definition = storage::custom_fields::get_definition(&self.db.conn, &field_def_id)?;
        self.ensure_custom_field_entity_exists(&definition.entity_type, &entity_id)?;

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let field_value =
            storage::custom_fields::set_value(&tx, &field_def_id, &entity_id, &value)?;
        storage::sync::record_change(
            &tx,
            "custom_field_value",
            &field_value.id,
            "value",
            None,
            Some(&field_value.value),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "set_value",
            Some("custom_field_value"),
            Some(&field_value.id),
            None::<&()>,
            Some(&field_value),
            &device_id,
        )?;
        tx.commit()?;
        Ok(field_value)
    }

    fn ensure_custom_field_entity_exists(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> CrmResult<()> {
        match entity_type {
            "contact" => storage::contacts::get_contact(&self.db.conn, entity_id).map(|_| ()),
            "deal" => storage::deals::get_deal(&self.db.conn, entity_id).map(|_| ()),
            "activity" => storage::activities::get_activity(&self.db.conn, entity_id).map(|_| ()),
            "organization" => {
                storage::organizations::get_organization(&self.db.conn, entity_id).map(|_| ())
            }
            _ => Err(CrmError::InvalidInput(format!(
                "Invalid entity_type '{}'",
                entity_type
            ))),
        }
    }

    pub fn list_custom_field_values(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> CrmResult<Vec<EntityCustomFieldValue>> {
        storage::custom_fields::list_values_for_entity(&self.db.conn, entity_type, entity_id)
    }

    pub fn list_custom_field_values_for_type(
        &self,
        entity_type: &str,
    ) -> CrmResult<Vec<EntityTypeCustomFieldValue>> {
        storage::custom_fields::list_values_for_entity_type(&self.db.conn, entity_type)
    }

    fn custom_field_import_target_keys(&self, entity_type: &str) -> CrmResult<Vec<String>> {
        Ok(self
            .custom_field_import_targets(entity_type)?
            .keys()
            .cloned()
            .collect())
    }

    fn custom_field_import_targets(
        &self,
        entity_type: &str,
    ) -> CrmResult<BTreeMap<String, String>> {
        let definitions =
            storage::custom_fields::list_definitions(&self.db.conn, Some(entity_type))?;
        Ok(custom_field_targets(definitions))
    }

    fn custom_field_export_targets(
        &self,
        entity_type: &str,
    ) -> CrmResult<BTreeMap<String, String>> {
        let definitions =
            storage::custom_fields::list_definitions(&self.db.conn, Some(entity_type))?;
        Ok(custom_field_targets(definitions))
    }

    fn custom_field_snapshot(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> CrmResult<BTreeMap<String, String>> {
        Ok(
            storage::custom_fields::list_values_for_entity(&self.db.conn, entity_type, entity_id)?
                .into_iter()
                .map(|value| (value.field_def_id, value.value))
                .collect(),
        )
    }

    fn restore_custom_field_value_for_rollback(
        &mut self,
        field_def_id: &str,
        entity_id: &str,
        value: &str,
    ) -> CrmResult<()> {
        self.set_custom_field_value(
            field_def_id.to_string(),
            entity_id.to_string(),
            value.to_string(),
        )?;
        Ok(())
    }

    fn delete_custom_field_value_for_rollback(
        &mut self,
        field_def_id: &str,
        entity_id: &str,
    ) -> CrmResult<()> {
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let Some(before) =
            storage::custom_fields::get_value_for_entity_field(&tx, field_def_id, entity_id)?
        else {
            tx.commit()?;
            return Ok(());
        };

        storage::custom_fields::delete_value_for_entity_field(&tx, field_def_id, entity_id)?;
        storage::sync::record_change(
            &tx,
            "custom_field_value",
            &before.id,
            "__delete__",
            Some(&before.value),
            None,
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "delete_value",
            Some("custom_field_value"),
            Some(&before.id),
            Some(&before),
            Option::<&CustomFieldValue>::None,
            &device_id,
        )?;
        tx.commit()?;
        Ok(())
    }

    fn delete_custom_field_values_for_rollback(
        &mut self,
        entity_type: &str,
        entity_id: &str,
    ) -> CrmResult<()> {
        let field_def_ids = self
            .custom_field_snapshot(entity_type, entity_id)?
            .keys()
            .cloned()
            .collect::<Vec<_>>();

        for field_def_id in field_def_ids {
            self.delete_custom_field_value_for_rollback(&field_def_id, entity_id)?;
        }

        Ok(())
    }

    fn apply_custom_field_import_updates(
        &mut self,
        entity_id: &str,
        updates: &BTreeMap<String, String>,
    ) -> CrmResult<()> {
        for (field_def_id, value) in updates {
            self.set_custom_field_value(
                field_def_id.clone(),
                entity_id.to_string(),
                value.clone(),
            )?;
        }

        Ok(())
    }

    pub fn get_pipeline_conversion_report(&self) -> CrmResult<PipelineConversionReport> {
        storage::reporting::get_pipeline_conversion_report(&self.db.conn)
    }

    pub fn get_activity_funnel_report(&self) -> CrmResult<ActivityFunnelReport> {
        storage::reporting::get_activity_funnel_report(&self.db.conn)
    }

    pub fn get_sync_status(&self) -> CrmResult<SyncStatus> {
        let pending_changes = storage::sync::get_all_pending_changes(&self.db.conn)?.len() as u32;
        let last_sync_at = storage::sync::get_latest_change_timestamp(&self.db.conn)?;
        let sync_enabled = storage::settings::get_setting(&self.db.conn, "sync_enabled")?
            .map(|s| s.value)
            .unwrap_or_else(|| "false".to_string());
        let sync_url = storage::settings::get_setting(&self.db.conn, "sync_url")?
            .map(|s| s.value)
            .unwrap_or_default();

        sync_status_from_settings(
            sync_enabled.as_str(),
            sync_url.as_str(),
            last_sync_at,
            pending_changes,
        )
    }

    pub fn trigger_sync(&self) -> CrmResult<SyncStatus> {
        self.get_sync_status()
    }

    pub fn import_contacts_csv(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_contacts_csv_with_options(file_path, ImportOptions::default())
    }

    pub fn import_contacts_csv_with_options(
        &mut self,
        file_path: &str,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_csv_with_row_numbers(file_content.as_slice())?;
        self.import_contact_rows(rows, options)
    }

    pub fn import_contacts_csv_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_contacts_csv_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_contacts_csv_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("contact")?;
        let rows = parse_contacts_csv_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.import_contact_rows(rows, options)
    }

    pub fn import_contacts_json(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_contacts_json_with_options(file_path, ImportOptions::default())
    }

    pub fn import_contacts_json_with_options(
        &mut self,
        file_path: &str,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_json_with_row_numbers(file_content.as_slice())?;
        self.import_contact_rows(rows, options)
    }

    pub fn import_contacts_json_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_contacts_json_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_contacts_json_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("contact")?;
        let rows = parse_contacts_json_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.import_contact_rows(rows, options)
    }

    pub fn preview_contacts_json_import(&self, file_path: &str) -> CrmResult<JsonImportPreview> {
        let file_content = fs::read(file_path)?;
        preview_contacts_json_import(file_content.as_slice())
    }

    fn import_contact_rows(
        &mut self,
        rows: Vec<(usize, ContactCsvRow)>,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let custom_targets = self.custom_field_import_targets("contact")?;
        let custom_rows = rows
            .iter()
            .map(|(row_number, row)| (*row_number, row.custom_fields.clone()))
            .collect::<Vec<_>>();
        validate_import_custom_fields(&custom_rows, &custom_targets)?;

        let mut created = 0u32;
        let mut merged = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();
        let mut rollback_actions = Vec::new();

        for (row_number, row) in rows {
            if options.merge_duplicates {
                match self.find_unique_contact_import_match(&row) {
                    Ok(Some(contact)) => {
                        match self.merge_contact_import_row(
                            row_number,
                            &contact.id,
                            &row,
                            &custom_targets,
                        ) {
                            Ok(rollback_action) => {
                                if let Some(action) = rollback_action {
                                    rollback_actions.push(action);
                                }
                                if let Err(e) = storage::audit::record_audit(
                                    &self.db.conn,
                                    ACTOR_IMPORT,
                                    None,
                                    "import_row_merge",
                                    Some("contact"),
                                    Some(&contact.id),
                                    None,
                                    None,
                                    &self.device_id,
                                ) {
                                    errors.push(format!(
                                        "Row {}: audit record failed: {}",
                                        row_number, e
                                    ));
                                }
                                merged += 1;
                                continue;
                            }
                            Err(e) => {
                                errors.push(format!(
                                    "Row {}: {} ({})",
                                    row_number, e, row.first_name
                                ));
                                skipped += 1;
                                continue;
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        errors.push(format!("Row {}: {} ({})", row_number, e, row.first_name));
                        skipped += 1;
                        continue;
                    }
                }
            }

            match self.create_contact(
                Some("person".to_string()),
                Some(row.first_name.clone()),
                row.last_name.clone(),
                row.org_name.clone(),
                row.email.clone(),
                row.phone.clone(),
                row.address.clone(),
                row.city.clone(),
                row.country.clone(),
                None,
                row.notes.clone(),
            ) {
                Ok(contact) => {
                    match custom_field_import_updates(&row.custom_fields, &custom_targets).and_then(
                        |updates| {
                            self.apply_custom_field_import_updates(&contact.id, &updates)?;
                            self.custom_field_snapshot("contact", &contact.id)
                        },
                    ) {
                        Ok(custom_fields) => {
                            rollback_actions.push(
                                import_rollback::ImportRollbackAction::created_contact(
                                    row_number,
                                    &contact,
                                    custom_fields,
                                ),
                            );
                            if let Err(e) = storage::audit::record_audit(
                                &self.db.conn,
                                ACTOR_IMPORT,
                                None,
                                "import_row",
                                Some("contact"),
                                Some(&contact.id),
                                None,
                                None,
                                &self.device_id,
                            ) {
                                errors.push(format!(
                                    "Row {}: audit record failed: {}",
                                    row_number, e
                                ));
                            }
                            created += 1;
                        }
                        Err(e) => {
                            errors.push(format!("Row {}: {} ({})", row_number, e, row.first_name));
                            skipped += 1;
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Row {}: {} ({})", row_number, e, row.first_name));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            merged,
            skipped,
            errors,
            rollback_plan: import_rollback::ImportRollbackPlan::from_actions(rollback_actions),
        })
    }

    fn find_unique_contact_import_match(
        &self,
        row: &ContactCsvRow,
    ) -> CrmResult<Option<storage::contacts::Contact>> {
        let mut matches = BTreeMap::new();

        if let Some(email) = trimmed_optional(&row.email) {
            for contact in storage::contacts::find_active_contacts_by_email(&self.db.conn, &email)?
            {
                matches.insert(contact.id.clone(), contact);
            }
        }

        if let Some(phone) = trimmed_optional(&row.phone) {
            for contact in storage::contacts::find_active_contacts_by_phone(&self.db.conn, &phone)?
            {
                matches.insert(contact.id.clone(), contact);
            }
        }

        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.into_values().next()),
            _ => Err(CrmError::InvalidInput(format!(
                "duplicate auto-merge skipped because multiple contacts match: {}",
                matches.keys().cloned().collect::<Vec<_>>().join(", ")
            ))),
        }
    }

    fn merge_contact_import_row(
        &mut self,
        row_number: usize,
        contact_id: &str,
        row: &ContactCsvRow,
        custom_targets: &BTreeMap<String, String>,
    ) -> CrmResult<Option<import_rollback::ImportRollbackAction>> {
        let existing = self.get_contact(contact_id)?;
        let before_custom_fields = self.custom_field_snapshot("contact", contact_id)?;
        let incoming_custom_fields =
            custom_field_import_updates(&row.custom_fields, custom_targets)?;
        let custom_updates =
            custom_field_auto_merge_updates(&before_custom_fields, &incoming_custom_fields);
        let first_name = fill_blank_string(&existing.first_name, Some(&row.first_name));
        let last_name = fill_blank_string(&existing.last_name, row.last_name.as_ref());
        let org_name = fill_blank_string(&existing.org_name, row.org_name.as_ref());
        let email = fill_blank_string(&existing.email, row.email.as_ref());
        let phone = fill_blank_string(&existing.phone, row.phone.as_ref());
        let address = fill_blank_string(&existing.address, row.address.as_ref());
        let city = fill_blank_string(&existing.city, row.city.as_ref());
        let country = fill_blank_string(&existing.country, row.country.as_ref());
        let notes = fill_blank_string(&existing.notes, row.notes.as_ref());

        if first_name.is_none()
            && last_name.is_none()
            && org_name.is_none()
            && email.is_none()
            && phone.is_none()
            && address.is_none()
            && city.is_none()
            && country.is_none()
            && notes.is_none()
            && custom_updates.is_empty()
        {
            return Ok(None);
        }

        let updated = if first_name.is_some()
            || last_name.is_some()
            || org_name.is_some()
            || email.is_some()
            || phone.is_some()
            || address.is_some()
            || city.is_some()
            || country.is_some()
            || notes.is_some()
        {
            self.update_contact(
                contact_id, None, first_name, last_name, org_name, email, phone, address, city,
                country, notes, None,
            )?
        } else {
            existing.clone()
        };
        self.apply_custom_field_import_updates(contact_id, &custom_updates)?;
        let post_custom_fields = self.custom_field_snapshot("contact", contact_id)?;
        Ok(import_rollback::ImportRollbackAction::merged_contact(
            row_number,
            &existing,
            &updated,
            before_custom_fields,
            post_custom_fields,
        ))
    }

    pub fn preflight_contacts_csv_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_csv_with_row_numbers(file_content.as_slice())?;
        self.preflight_contact_rows(rows)
    }

    pub fn preflight_contacts_csv_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("contact")?;
        let rows = parse_contacts_csv_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.preflight_contact_rows(rows)
    }

    pub fn preflight_contacts_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_contacts_json_with_row_numbers(file_content.as_slice())?;
        self.preflight_contact_rows(rows)
    }

    pub fn preflight_contacts_json_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("contact")?;
        let rows = parse_contacts_json_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.preflight_contact_rows(rows)
    }

    fn preflight_contact_rows(
        &self,
        rows: Vec<(usize, ContactCsvRow)>,
    ) -> CrmResult<ImportPreflightReport> {
        let custom_targets = self.custom_field_export_targets("contact")?;
        let custom_rows = rows
            .iter()
            .map(|(row_number, row)| (*row_number, row.custom_fields.clone()))
            .collect::<Vec<_>>();
        validate_import_custom_fields(&custom_rows, &custom_targets)?;

        let mut warnings = Vec::new();

        for (row_number, row) in &rows {
            if let Some(email) = trimmed_optional(&row.email) {
                for contact in
                    storage::contacts::find_active_contacts_by_email(&self.db.conn, &email)?
                {
                    warnings.push(import_duplicate_warning(
                        "contacts",
                        *row_number,
                        "email",
                        &email,
                        "contact",
                        &contact.id,
                        &contact_display_label(&contact),
                        format!("Email '{}' matches existing contact", email),
                    ));
                }
            }

            if let Some(phone) = trimmed_optional(&row.phone) {
                for contact in
                    storage::contacts::find_active_contacts_by_phone(&self.db.conn, &phone)?
                {
                    warnings.push(import_duplicate_warning(
                        "contacts",
                        *row_number,
                        "phone",
                        &phone,
                        "contact",
                        &contact.id,
                        &contact_display_label(&contact),
                        format!("Phone '{}' matches existing contact", phone),
                    ));
                }
            }
        }

        Ok(import_preflight_report("contacts", rows.len(), warnings))
    }

    pub fn export_contacts_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_contact_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_contacts_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_contacts_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_contact_rows()?;
        let count = rows.len() as u32;
        write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_contact_rows(&self) -> CrmResult<Vec<ContactCsvRow>> {
        let custom_targets = self.custom_field_import_targets("contact")?;
        let field_targets = custom_targets
            .iter()
            .map(|(target, field_def_id)| (field_def_id.clone(), target.clone()))
            .collect::<BTreeMap<_, _>>();
        let custom_values = self
            .list_custom_field_values_for_type("contact")?
            .into_iter()
            .map(|value| ((value.entity_id, value.field_def_id), value.value))
            .collect::<BTreeMap<_, _>>();
        let params = ContactListParams {
            page: 1,
            per_page: 100_000,
            sort_by: "first_name".to_string(),
            sort_dir: "asc".to_string(),
            filter_type: None,
            filter_lifecycle: None,
            filter_owner: None,
            search_query: None,
            custom_field_def_id: None,
            custom_field_query: None,
        };
        let result = storage::contacts::list_contacts(&self.db.conn, &params)?;
        Ok(result
            .contacts
            .iter()
            .map(|c| ContactCsvRow {
                first_name: c.first_name.clone(),
                last_name: Some(c.last_name.clone()).filter(|s| !s.is_empty()),
                org_name: Some(c.org_name.clone()).filter(|s| !s.is_empty()),
                email: Some(c.email.clone()).filter(|s| !s.is_empty()),
                phone: Some(c.phone.clone()).filter(|s| !s.is_empty()),
                address: Some(c.address.clone()).filter(|s| !s.is_empty()),
                city: Some(c.city.clone()).filter(|s| !s.is_empty()),
                country: Some(c.country.clone()).filter(|s| !s.is_empty()),
                notes: Some(c.notes.clone()).filter(|s| !s.is_empty()),
                custom_fields: field_targets
                    .iter()
                    .map(|(field_def_id, target)| {
                        (
                            target.clone(),
                            custom_values
                                .get(&(c.id.clone(), field_def_id.clone()))
                                .cloned()
                                .unwrap_or_default(),
                        )
                    })
                    .collect(),
            })
            .collect())
    }

    pub fn import_deals_csv(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_deals_csv_with_options(file_path, ImportOptions::default())
    }

    pub fn import_deals_csv_with_options(
        &mut self,
        file_path: &str,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_csv_with_row_numbers(file_content.as_slice())?;
        self.import_deal_rows(rows, options)
    }

    pub fn import_deals_csv_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_deals_csv_with_mapping_and_options(file_path, mapping, ImportOptions::default())
    }

    pub fn import_deals_csv_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("deal")?;
        let rows = parse_deals_csv_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.import_deal_rows(rows, options)
    }

    pub fn import_deals_json(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_deals_json_with_options(file_path, ImportOptions::default())
    }

    pub fn import_deals_json_with_options(
        &mut self,
        file_path: &str,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_json_with_row_numbers(file_content.as_slice())?;
        self.import_deal_rows(rows, options)
    }

    pub fn import_deals_json_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_deals_json_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_deals_json_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("deal")?;
        let rows = parse_deals_json_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.import_deal_rows(rows, options)
    }

    pub fn preview_deals_json_import(&self, file_path: &str) -> CrmResult<JsonImportPreview> {
        let file_content = fs::read(file_path)?;
        preview_deals_json_import(file_content.as_slice())
    }

    fn import_deal_rows(
        &mut self,
        rows: Vec<(usize, DealCsvRow)>,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let custom_targets = self.custom_field_import_targets("deal")?;
        let custom_rows = rows
            .iter()
            .map(|(row_number, row)| (*row_number, row.custom_fields.clone()))
            .collect::<Vec<_>>();
        validate_import_custom_fields(&custom_rows, &custom_targets)?;

        let mut created = 0u32;
        let mut merged = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();
        let mut rollback_actions = Vec::new();

        for (row_number, row) in rows {
            if options.merge_duplicates {
                match self.find_unique_deal_import_match(&row) {
                    Ok(Some(deal)) => {
                        match self.merge_deal_import_row(
                            row_number,
                            &deal.id,
                            &row,
                            &custom_targets,
                        ) {
                            Ok(rollback_action) => {
                                if let Some(action) = rollback_action {
                                    rollback_actions.push(action);
                                }
                                if let Err(e) = storage::audit::record_audit(
                                    &self.db.conn,
                                    ACTOR_IMPORT,
                                    None,
                                    "import_row_merge",
                                    Some("deal"),
                                    Some(&deal.id),
                                    None,
                                    None,
                                    &self.device_id,
                                ) {
                                    errors.push(format!(
                                        "Row {}: audit record failed: {}",
                                        row_number, e
                                    ));
                                }
                                merged += 1;
                                continue;
                            }
                            Err(e) => {
                                errors.push(format!("Row {}: {} ({})", row_number, e, row.title));
                                skipped += 1;
                                continue;
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        errors.push(format!("Row {}: {} ({})", row_number, e, row.title));
                        skipped += 1;
                        continue;
                    }
                }
            }

            let value = row
                .value
                .as_deref()
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0.0);
            match self.create_deal(
                row.title.clone(),
                Some(value),
                row.currency.clone(),
                row.stage.clone(),
                Some(0),
                row.expected_close.clone(),
                None,
                None,
                row.notes.clone(),
            ) {
                Ok(deal) => {
                    match custom_field_import_updates(&row.custom_fields, &custom_targets).and_then(
                        |updates| {
                            self.apply_custom_field_import_updates(&deal.id, &updates)?;
                            self.custom_field_snapshot("deal", &deal.id)
                        },
                    ) {
                        Ok(custom_fields) => {
                            rollback_actions.push(
                                import_rollback::ImportRollbackAction::created_deal(
                                    row_number,
                                    &deal,
                                    custom_fields,
                                ),
                            );
                            if let Err(e) = storage::audit::record_audit(
                                &self.db.conn,
                                ACTOR_IMPORT,
                                None,
                                "import_row",
                                Some("deal"),
                                Some(&deal.id),
                                None,
                                None,
                                &self.device_id,
                            ) {
                                errors.push(format!(
                                    "Row {}: audit record failed: {}",
                                    row_number, e
                                ));
                            }
                            created += 1;
                        }
                        Err(e) => {
                            errors.push(format!("Row {}: {} ({})", row_number, e, row.title));
                            skipped += 1;
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Row {}: {} ({})", row_number, e, row.title));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            merged,
            skipped,
            errors,
            rollback_plan: import_rollback::ImportRollbackPlan::from_actions(rollback_actions),
        })
    }

    fn find_unique_deal_import_match(
        &self,
        row: &DealCsvRow,
    ) -> CrmResult<Option<storage::deals::Deal>> {
        let title = row.title.trim().to_string();
        if title.is_empty() {
            return Ok(None);
        }

        let matches = storage::deals::find_active_deals_by_title(&self.db.conn, &title)?;

        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.into_iter().next()),
            _ => Err(CrmError::InvalidInput(format!(
                "duplicate auto-merge skipped because multiple deals match title '{}': {}",
                title,
                matches
                    .iter()
                    .map(|deal| deal.id.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        }
    }

    fn merge_deal_import_row(
        &mut self,
        row_number: usize,
        deal_id: &str,
        row: &DealCsvRow,
        custom_targets: &BTreeMap<String, String>,
    ) -> CrmResult<Option<import_rollback::ImportRollbackAction>> {
        let existing = self.get_deal(deal_id)?;
        let before_custom_fields = self.custom_field_snapshot("deal", deal_id)?;
        let incoming_custom_fields =
            custom_field_import_updates(&row.custom_fields, custom_targets)?;
        let custom_updates =
            custom_field_auto_merge_updates(&before_custom_fields, &incoming_custom_fields);
        let value = fill_zero_value(existing.value, row.value.as_ref());
        let expected_close =
            fill_blank_option(&existing.expected_close, row.expected_close.as_ref()).map(Some);
        let notes = fill_blank_string(&existing.notes, row.notes.as_ref());

        if value.is_none()
            && expected_close.is_none()
            && notes.is_none()
            && custom_updates.is_empty()
        {
            return Ok(None);
        }

        let updated = if value.is_some() || expected_close.is_some() || notes.is_some() {
            self.update_deal(
                deal_id,
                None,
                value,
                None,
                None,
                None,
                expected_close,
                None,
                None,
                notes,
            )?
        } else {
            existing.clone()
        };
        self.apply_custom_field_import_updates(deal_id, &custom_updates)?;
        let post_custom_fields = self.custom_field_snapshot("deal", deal_id)?;
        Ok(import_rollback::ImportRollbackAction::merged_deal(
            row_number,
            &existing,
            &updated,
            before_custom_fields,
            post_custom_fields,
        ))
    }

    pub fn preflight_deals_csv_import(&self, file_path: &str) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_csv_with_row_numbers(file_content.as_slice())?;
        self.preflight_deal_rows(rows)
    }

    pub fn preflight_deals_csv_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("deal")?;
        let rows = parse_deals_csv_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.preflight_deal_rows(rows)
    }

    pub fn preflight_deals_json_import(&self, file_path: &str) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_deals_json_with_row_numbers(file_content.as_slice())?;
        self.preflight_deal_rows(rows)
    }

    pub fn preflight_deals_json_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("deal")?;
        let rows = parse_deals_json_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.preflight_deal_rows(rows)
    }

    fn preflight_deal_rows(
        &self,
        rows: Vec<(usize, DealCsvRow)>,
    ) -> CrmResult<ImportPreflightReport> {
        let custom_targets = self.custom_field_export_targets("deal")?;
        let custom_rows = rows
            .iter()
            .map(|(row_number, row)| (*row_number, row.custom_fields.clone()))
            .collect::<Vec<_>>();
        validate_import_custom_fields(&custom_rows, &custom_targets)?;

        let mut warnings = Vec::new();

        for (row_number, row) in &rows {
            let title = row.title.trim().to_string();
            for deal in storage::deals::find_active_deals_by_title(&self.db.conn, &title)? {
                warnings.push(import_duplicate_warning(
                    "deals",
                    *row_number,
                    "title",
                    &title,
                    "deal",
                    &deal.id,
                    &deal_display_label(&deal),
                    format!("Title '{}' matches existing deal", title),
                ));
            }
        }

        Ok(import_preflight_report("deals", rows.len(), warnings))
    }

    pub fn export_deals_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_deal_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_deals_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_deals_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_deal_rows()?;
        let count = rows.len() as u32;
        write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_deal_rows(&self) -> CrmResult<Vec<DealCsvRow>> {
        let custom_targets = self.custom_field_import_targets("deal")?;
        let field_targets = custom_targets
            .iter()
            .map(|(target, field_def_id)| (field_def_id.clone(), target.clone()))
            .collect::<BTreeMap<_, _>>();
        let custom_values = self
            .list_custom_field_values_for_type("deal")?
            .into_iter()
            .map(|value| ((value.entity_id, value.field_def_id), value.value))
            .collect::<BTreeMap<_, _>>();
        let all_deals = storage::deals::list_deals(&self.db.conn)?;
        Ok(all_deals
            .iter()
            .map(|d| DealCsvRow {
                title: d.title.clone(),
                value: Some(format!("{:.2}", d.value)),
                currency: Some(d.currency.clone()),
                stage: Some(d.stage.clone()),
                expected_close: d.expected_close.clone(),
                notes: Some(d.notes.clone()).filter(|s| !s.is_empty()),
                custom_fields: field_targets
                    .iter()
                    .map(|(field_def_id, target)| {
                        (
                            target.clone(),
                            custom_values
                                .get(&(d.id.clone(), field_def_id.clone()))
                                .cloned()
                                .unwrap_or_default(),
                        )
                    })
                    .collect(),
            })
            .collect())
    }

    pub fn import_activities_csv(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_activities_csv_with_options(file_path, ImportOptions::default())
    }

    pub fn import_activities_csv_with_options(
        &mut self,
        file_path: &str,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_activities_csv_with_row_numbers(file_content.as_slice())?;
        self.import_activity_rows(rows)
    }

    pub fn import_activities_csv_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_activities_csv_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_activities_csv_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("activity")?;
        let rows = parse_activities_csv_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.import_activity_rows(rows)
    }

    pub fn import_activities_json(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_activities_json_with_options(file_path, ImportOptions::default())
    }

    pub fn import_activities_json_with_options(
        &mut self,
        file_path: &str,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_activities_json_with_row_numbers(file_content.as_slice())?;
        self.import_activity_rows(rows)
    }

    pub fn import_activities_json_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_activities_json_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_activities_json_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("activity")?;
        let rows = parse_activities_json_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.import_activity_rows(rows)
    }

    pub fn preview_activities_json_import(&self, file_path: &str) -> CrmResult<JsonImportPreview> {
        let file_content = fs::read(file_path)?;
        preview_activities_json_import(file_content.as_slice())
    }

    fn import_activity_rows(
        &mut self,
        rows: Vec<(usize, ActivityCsvRow)>,
    ) -> CrmResult<ImportResult> {
        let custom_targets = self.custom_field_import_targets("activity")?;
        let custom_rows = rows
            .iter()
            .map(|(row_number, row)| (*row_number, row.custom_fields.clone()))
            .collect::<Vec<_>>();
        validate_import_custom_fields(&custom_rows, &custom_targets)?;

        let mut created = 0u32;
        let merged = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();
        let mut rollback_actions = Vec::new();

        for (row_number, row) in rows {
            match self.create_activity(
                row.activity_type.clone(),
                row.title.clone(),
                row.description.clone(),
                row.due_date.clone(),
                row.contact_id.clone(),
                row.deal_id.clone(),
            ) {
                Ok(activity) => {
                    let activity = if row.completed.unwrap_or(false) {
                        self.mark_activity_complete(&activity.id)
                    } else {
                        Ok(activity)
                    };

                    match activity.and_then(|activity| {
                        custom_field_import_updates(&row.custom_fields, &custom_targets).and_then(
                            |updates| {
                                self.apply_custom_field_import_updates(&activity.id, &updates)?;
                                Ok(activity)
                            },
                        )
                    }) {
                        Ok(activity) => {
                            match self.custom_field_snapshot("activity", &activity.id) {
                                Ok(custom_fields) => {
                                    rollback_actions.push(
                                        import_rollback::ImportRollbackAction::created_activity(
                                            row_number,
                                            &activity,
                                            custom_fields,
                                        ),
                                    );
                                    if let Err(e) = storage::audit::record_audit(
                                        &self.db.conn,
                                        ACTOR_IMPORT,
                                        None,
                                        "import_row",
                                        Some("activity"),
                                        Some(&activity.id),
                                        None,
                                        None,
                                        &self.device_id,
                                    ) {
                                        errors.push(format!(
                                            "Row {}: audit record failed: {}",
                                            row_number, e
                                        ));
                                    }
                                    created += 1;
                                }
                                Err(e) => {
                                    errors
                                        .push(format!("Row {}: {} ({})", row_number, e, row.title));
                                    skipped += 1;
                                }
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Row {}: {} ({})", row_number, e, row.title));
                            skipped += 1;
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Row {}: {} ({})", row_number, e, row.title));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            merged,
            skipped,
            errors,
            rollback_plan: import_rollback::ImportRollbackPlan::from_actions(rollback_actions),
        })
    }

    pub fn preflight_activities_csv_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_activities_csv_with_row_numbers(file_content.as_slice())?;
        self.preflight_activity_rows(rows)
    }

    pub fn preflight_activities_csv_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("activity")?;
        let rows = parse_activities_csv_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.preflight_activity_rows(rows)
    }

    pub fn preflight_activities_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_activities_json_with_row_numbers(file_content.as_slice())?;
        self.preflight_activity_rows(rows)
    }

    pub fn preflight_activities_json_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("activity")?;
        let rows = parse_activities_json_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.preflight_activity_rows(rows)
    }

    fn preflight_activity_rows(
        &self,
        rows: Vec<(usize, ActivityCsvRow)>,
    ) -> CrmResult<ImportPreflightReport> {
        let custom_targets = self.custom_field_export_targets("activity")?;
        let custom_rows = rows
            .iter()
            .map(|(row_number, row)| (*row_number, row.custom_fields.clone()))
            .collect::<Vec<_>>();
        validate_import_custom_fields(&custom_rows, &custom_targets)?;

        Ok(import_preflight_report(
            "activities",
            rows.len(),
            Vec::new(),
        ))
    }

    pub fn export_activities_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_activity_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_activities_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_activities_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_activity_rows()?;
        let count = rows.len() as u32;
        write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_activity_rows(&self) -> CrmResult<Vec<ActivityCsvRow>> {
        let custom_targets = self.custom_field_import_targets("activity")?;
        let field_targets = custom_targets
            .iter()
            .map(|(target, field_def_id)| (field_def_id.clone(), target.clone()))
            .collect::<BTreeMap<_, _>>();
        let custom_values = self
            .list_custom_field_values_for_type("activity")?
            .into_iter()
            .map(|value| ((value.entity_id, value.field_def_id), value.value))
            .collect::<BTreeMap<_, _>>();
        let activities = storage::activities::list_activities(&self.db.conn)?;
        Ok(activities
            .iter()
            .map(|activity| ActivityCsvRow {
                activity_type: activity.activity_type.clone(),
                title: activity.title.clone(),
                description: Some(activity.description.clone()).filter(|s| !s.is_empty()),
                due_date: activity.due_date.clone(),
                completed: Some(activity.completed),
                contact_id: activity.contact_id.clone(),
                deal_id: activity.deal_id.clone(),
                custom_fields: field_targets
                    .iter()
                    .map(|(field_def_id, target)| {
                        (
                            target.clone(),
                            custom_values
                                .get(&(activity.id.clone(), field_def_id.clone()))
                                .cloned()
                                .unwrap_or_default(),
                        )
                    })
                    .collect(),
            })
            .collect())
    }

    pub fn import_notes_csv(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_notes_csv_with_options(file_path, ImportOptions::default())
    }

    pub fn import_notes_csv_with_options(
        &mut self,
        file_path: &str,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_notes_csv_with_row_numbers(file_content.as_slice())?;
        self.import_note_rows(rows)
    }

    pub fn import_notes_csv_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_notes_csv_with_mapping_and_options(file_path, mapping, ImportOptions::default())
    }

    pub fn import_notes_csv_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_notes_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_note_rows(rows)
    }

    pub fn import_notes_json(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_notes_json_with_options(file_path, ImportOptions::default())
    }

    pub fn import_notes_json_with_options(
        &mut self,
        file_path: &str,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_notes_json_with_row_numbers(file_content.as_slice())?;
        self.import_note_rows(rows)
    }

    pub fn import_notes_json_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_notes_json_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_notes_json_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_notes_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_note_rows(rows)
    }

    pub fn preview_notes_json_import(&self, file_path: &str) -> CrmResult<JsonImportPreview> {
        let file_content = fs::read(file_path)?;
        preview_notes_json_import(file_content.as_slice())
    }

    fn import_note_rows(&mut self, rows: Vec<(usize, NoteCsvRow)>) -> CrmResult<ImportResult> {
        let mut created = 0u32;
        let merged = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();
        let mut rollback_actions = Vec::new();

        for (row_number, row) in rows {
            let (entity_type, entity_id, content) =
                match validate_note_import_row(&self.db.conn, row_number, &row) {
                    Ok(values) => values,
                    Err(e) => {
                        errors.push(format!(
                            "Row {}: {} ({})",
                            row_number,
                            e,
                            note_row_label(&row)
                        ));
                        skipped += 1;
                        continue;
                    }
                };

            match self.create_note(entity_type, entity_id, content) {
                Ok(note) => {
                    rollback_actions.push(import_rollback::ImportRollbackAction::created_note(
                        row_number, &note,
                    ));
                    if let Err(e) = storage::audit::record_audit(
                        &self.db.conn,
                        ACTOR_IMPORT,
                        None,
                        "import_row",
                        Some("note"),
                        Some(&note.id),
                        None,
                        None,
                        &self.device_id,
                    ) {
                        errors.push(format!("Row {}: audit record failed: {}", row_number, e));
                    }
                    created += 1;
                }
                Err(e) => {
                    errors.push(format!(
                        "Row {}: {} ({})",
                        row_number,
                        e,
                        note_row_label(&row)
                    ));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            merged,
            skipped,
            errors,
            rollback_plan: import_rollback::ImportRollbackPlan::from_actions(rollback_actions),
        })
    }

    pub fn preflight_notes_csv_import(&self, file_path: &str) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_notes_csv_with_row_numbers(file_content.as_slice())?;
        self.preflight_note_rows(rows)
    }

    pub fn preflight_notes_csv_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_notes_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_note_rows(rows)
    }

    pub fn preflight_notes_json_import(&self, file_path: &str) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_notes_json_with_row_numbers(file_content.as_slice())?;
        self.preflight_note_rows(rows)
    }

    pub fn preflight_notes_json_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_notes_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_note_rows(rows)
    }

    fn preflight_note_rows(
        &self,
        rows: Vec<(usize, NoteCsvRow)>,
    ) -> CrmResult<ImportPreflightReport> {
        for (row_number, row) in &rows {
            validate_note_import_row(&self.db.conn, *row_number, row)
                .map_err(|e| CrmError::InvalidInput(format!("Row {}: {}", row_number, e)))?;
        }

        Ok(import_preflight_report("notes", rows.len(), Vec::new()))
    }

    pub fn export_notes_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_note_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_notes_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_notes_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_note_rows()?;
        let count = rows.len() as u32;
        write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_note_rows(&self) -> CrmResult<Vec<NoteCsvRow>> {
        Ok(self
            .list_notes()?
            .into_iter()
            .map(|note| NoteCsvRow {
                entity_type: note.entity_type,
                entity_id: note.entity_id,
                content: note.content,
            })
            .collect())
    }

    pub fn import_tag_definitions_csv(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_tag_definitions_csv_with_options(file_path, ImportOptions::default())
    }

    pub fn import_tag_definitions_csv_with_options(
        &mut self,
        file_path: &str,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_definitions_csv_with_row_numbers(file_content.as_slice())?;
        self.import_tag_definition_rows(rows)
    }

    pub fn import_tag_definitions_csv_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_tag_definitions_csv_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_tag_definitions_csv_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_definitions_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_tag_definition_rows(rows)
    }

    pub fn import_tag_definitions_json(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_tag_definitions_json_with_options(file_path, ImportOptions::default())
    }

    pub fn import_tag_definitions_json_with_options(
        &mut self,
        file_path: &str,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_definitions_json_with_row_numbers(file_content.as_slice())?;
        self.import_tag_definition_rows(rows)
    }

    pub fn import_tag_definitions_json_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_tag_definitions_json_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_tag_definitions_json_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_definitions_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_tag_definition_rows(rows)
    }

    pub fn preview_tag_definitions_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<JsonImportPreview> {
        let file_content = fs::read(file_path)?;
        preview_tag_definitions_json_import(file_content.as_slice())
    }

    fn import_tag_definition_rows(
        &mut self,
        rows: Vec<(usize, TagDefinitionCsvRow)>,
    ) -> CrmResult<ImportResult> {
        let mut existing_by_name = self
            .list_tags()?
            .into_iter()
            .map(|tag| (tag.name.clone(), tag))
            .collect::<BTreeMap<_, _>>();
        let mut created = 0u32;
        let merged = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();
        let mut rollback_actions = Vec::new();

        for (row_number, row) in rows {
            let (name, color) = match validate_tag_definition_import_row(&row) {
                Ok(values) => values,
                Err(e) => {
                    errors.push(format!("Row {}: {} ({})", row_number, e, row.name));
                    skipped += 1;
                    continue;
                }
            };

            if existing_by_name.contains_key(&name) {
                skipped += 1;
                continue;
            }

            match self.create_tag(name.clone(), color) {
                Ok(tag) => {
                    existing_by_name.insert(tag.name.clone(), tag.clone());
                    rollback_actions.push(
                        import_rollback::ImportRollbackAction::created_tag_definition(
                            row_number, &tag,
                        ),
                    );
                    if let Err(e) = storage::audit::record_audit(
                        &self.db.conn,
                        ACTOR_IMPORT,
                        None,
                        "import_row",
                        Some("tag"),
                        Some(&tag.id),
                        None,
                        None,
                        &self.device_id,
                    ) {
                        errors.push(format!("Row {}: audit record failed: {}", row_number, e));
                    }
                    created += 1;
                }
                Err(e) => {
                    errors.push(format!("Row {}: {} ({})", row_number, e, row.name));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            merged,
            skipped,
            errors,
            rollback_plan: import_rollback::ImportRollbackPlan::from_actions(rollback_actions),
        })
    }

    pub fn preflight_tag_definitions_csv_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_definitions_csv_with_row_numbers(file_content.as_slice())?;
        self.preflight_tag_definition_rows(rows)
    }

    pub fn preflight_tag_definitions_csv_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_definitions_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_tag_definition_rows(rows)
    }

    pub fn preflight_tag_definitions_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_definitions_json_with_row_numbers(file_content.as_slice())?;
        self.preflight_tag_definition_rows(rows)
    }

    pub fn preflight_tag_definitions_json_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_definitions_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_tag_definition_rows(rows)
    }

    fn preflight_tag_definition_rows(
        &self,
        rows: Vec<(usize, TagDefinitionCsvRow)>,
    ) -> CrmResult<ImportPreflightReport> {
        for (row_number, row) in &rows {
            validate_tag_definition_import_row(row)
                .map_err(|e| CrmError::InvalidInput(format!("Row {}: {}", row_number, e)))?;
        }

        Ok(import_preflight_report(
            "tag_definitions",
            rows.len(),
            Vec::new(),
        ))
    }

    pub fn export_tag_definitions_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_tag_definition_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_tag_definitions_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_tag_definitions_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_tag_definition_rows()?;
        let count = rows.len() as u32;
        write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_tag_definition_rows(&self) -> CrmResult<Vec<TagDefinitionCsvRow>> {
        Ok(self
            .list_tags()?
            .into_iter()
            .map(|tag| TagDefinitionCsvRow {
                name: tag.name,
                color: Some(tag.color),
            })
            .collect())
    }

    pub fn import_custom_field_definitions_csv(
        &mut self,
        file_path: &str,
    ) -> CrmResult<ImportResult> {
        self.import_custom_field_definitions_csv_with_options(file_path, ImportOptions::default())
    }

    pub fn import_custom_field_definitions_csv_with_options(
        &mut self,
        file_path: &str,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_custom_field_definitions_csv_with_row_numbers(file_content.as_slice())?;
        self.import_custom_field_definition_rows(rows)
    }

    pub fn import_custom_field_definitions_csv_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_custom_field_definitions_csv_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_custom_field_definitions_csv_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows =
            parse_custom_field_definitions_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_custom_field_definition_rows(rows)
    }

    pub fn import_custom_field_definitions_json(
        &mut self,
        file_path: &str,
    ) -> CrmResult<ImportResult> {
        self.import_custom_field_definitions_json_with_options(file_path, ImportOptions::default())
    }

    pub fn import_custom_field_definitions_json_with_options(
        &mut self,
        file_path: &str,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_custom_field_definitions_json_with_row_numbers(file_content.as_slice())?;
        self.import_custom_field_definition_rows(rows)
    }

    pub fn import_custom_field_definitions_json_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_custom_field_definitions_json_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_custom_field_definitions_json_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows =
            parse_custom_field_definitions_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_custom_field_definition_rows(rows)
    }

    pub fn preview_custom_field_definitions_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<JsonImportPreview> {
        let file_content = fs::read(file_path)?;
        preview_custom_field_definitions_json_import(file_content.as_slice())
    }

    fn import_custom_field_definition_rows(
        &mut self,
        rows: Vec<(usize, CustomFieldDefinitionCsvRow)>,
    ) -> CrmResult<ImportResult> {
        let mut existing_by_key = self
            .list_custom_field_defs(None)?
            .into_iter()
            .map(|definition| {
                (
                    custom_field_definition_import_key(
                        &definition.entity_type,
                        &definition.field_name,
                    ),
                    definition,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut created = 0u32;
        let merged = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();
        let mut rollback_actions = Vec::new();

        for (row_number, row) in rows {
            let values = match validate_custom_field_definition_import_row(&row) {
                Ok(values) => values,
                Err(e) => {
                    errors.push(format!("Row {}: {} ({})", row_number, e, row.field_name));
                    skipped += 1;
                    continue;
                }
            };
            let key = custom_field_definition_import_key(&values.entity_type, &values.field_name);

            if let Some(existing) = existing_by_key.get(&key) {
                if custom_field_definition_matches_import(existing, &values) {
                    skipped += 1;
                } else {
                    errors.push(format!(
                        "Row {}: custom field definition already exists with different shape ({}:{})",
                        row_number, values.entity_type, values.field_name
                    ));
                    skipped += 1;
                }
                continue;
            }

            match self.create_custom_field_def(
                values.entity_type.clone(),
                values.field_name.clone(),
                values.field_type.clone(),
                values.field_options.clone(),
                Some(values.sort_order),
            ) {
                Ok(definition) => {
                    existing_by_key.insert(key, definition.clone());
                    rollback_actions.push(
                        import_rollback::ImportRollbackAction::created_custom_field_definition(
                            row_number,
                            &definition,
                        ),
                    );
                    if let Err(e) = storage::audit::record_audit(
                        &self.db.conn,
                        ACTOR_IMPORT,
                        None,
                        "import_row",
                        Some("custom_field_def"),
                        Some(&definition.id),
                        None,
                        None,
                        &self.device_id,
                    ) {
                        errors.push(format!("Row {}: audit record failed: {}", row_number, e));
                    }
                    created += 1;
                }
                Err(e) => {
                    errors.push(format!("Row {}: {} ({})", row_number, e, row.field_name));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            merged,
            skipped,
            errors,
            rollback_plan: import_rollback::ImportRollbackPlan::from_actions(rollback_actions),
        })
    }

    pub fn preflight_custom_field_definitions_csv_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_custom_field_definitions_csv_with_row_numbers(file_content.as_slice())?;
        self.preflight_custom_field_definition_rows(rows)
    }

    pub fn preflight_custom_field_definitions_csv_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows =
            parse_custom_field_definitions_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_custom_field_definition_rows(rows)
    }

    pub fn preflight_custom_field_definitions_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_custom_field_definitions_json_with_row_numbers(file_content.as_slice())?;
        self.preflight_custom_field_definition_rows(rows)
    }

    pub fn preflight_custom_field_definitions_json_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows =
            parse_custom_field_definitions_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_custom_field_definition_rows(rows)
    }

    fn preflight_custom_field_definition_rows(
        &self,
        rows: Vec<(usize, CustomFieldDefinitionCsvRow)>,
    ) -> CrmResult<ImportPreflightReport> {
        let existing_by_key = self
            .list_custom_field_defs(None)?
            .into_iter()
            .map(|definition| {
                (
                    custom_field_definition_import_key(
                        &definition.entity_type,
                        &definition.field_name,
                    ),
                    definition,
                )
            })
            .collect::<BTreeMap<_, _>>();

        for (row_number, row) in &rows {
            let values = validate_custom_field_definition_import_row(row)
                .map_err(|e| CrmError::InvalidInput(format!("Row {}: {}", row_number, e)))?;
            let key = custom_field_definition_import_key(&values.entity_type, &values.field_name);
            if let Some(existing) = existing_by_key.get(&key) {
                if !custom_field_definition_matches_import(existing, &values) {
                    return Err(CrmError::InvalidInput(format!(
                        "Row {}: custom field definition already exists with different shape ({}:{})",
                        row_number, values.entity_type, values.field_name
                    )));
                }
            }
        }

        Ok(import_preflight_report(
            "custom_field_definitions",
            rows.len(),
            Vec::new(),
        ))
    }

    pub fn export_custom_field_definitions_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_custom_field_definition_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_custom_field_definitions_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_custom_field_definitions_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_custom_field_definition_rows()?;
        let count = rows.len() as u32;
        write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_custom_field_definition_rows(&self) -> CrmResult<Vec<CustomFieldDefinitionCsvRow>> {
        Ok(self
            .list_custom_field_defs(None)?
            .into_iter()
            .map(|definition| CustomFieldDefinitionCsvRow {
                entity_type: definition.entity_type,
                field_name: definition.field_name,
                field_type: definition.field_type,
                field_options: definition.field_options,
                sort_order: Some(definition.sort_order.to_string()),
            })
            .collect())
    }

    pub fn import_tag_links_csv(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_tag_links_csv_with_options(file_path, ImportOptions::default())
    }

    pub fn import_tag_links_csv_with_options(
        &mut self,
        file_path: &str,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_links_csv_with_row_numbers(file_content.as_slice())?;
        self.import_tag_link_rows(rows)
    }

    pub fn import_tag_links_csv_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_tag_links_csv_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_tag_links_csv_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_links_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_tag_link_rows(rows)
    }

    pub fn import_tag_links_json(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_tag_links_json_with_options(file_path, ImportOptions::default())
    }

    pub fn import_tag_links_json_with_options(
        &mut self,
        file_path: &str,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_links_json_with_row_numbers(file_content.as_slice())?;
        self.import_tag_link_rows(rows)
    }

    pub fn import_tag_links_json_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_tag_links_json_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_tag_links_json_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        _options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_links_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.import_tag_link_rows(rows)
    }

    pub fn preview_tag_links_json_import(&self, file_path: &str) -> CrmResult<JsonImportPreview> {
        let file_content = fs::read(file_path)?;
        preview_tag_links_json_import(file_content.as_slice())
    }

    fn import_tag_link_rows(
        &mut self,
        rows: Vec<(usize, TagLinkCsvRow)>,
    ) -> CrmResult<ImportResult> {
        let mut created = 0u32;
        let merged = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();
        let mut rollback_actions = Vec::new();

        for (row_number, row) in rows {
            let (entity_type, entity_id, tag_id) =
                match validate_tag_link_import_row(&self.db.conn, &row) {
                    Ok(values) => values,
                    Err(e) => {
                        errors.push(format!(
                            "Row {}: {} ({})",
                            row_number,
                            e,
                            tag_link_row_label(&row)
                        ));
                        skipped += 1;
                        continue;
                    }
                };

            match storage::tags::active_tag_link_exists(
                &self.db.conn,
                &entity_type,
                &entity_id,
                &tag_id,
            ) {
                Ok(true) => {
                    skipped += 1;
                    continue;
                }
                Ok(false) => {}
                Err(e) => {
                    errors.push(format!(
                        "Row {}: {} ({})",
                        row_number,
                        e,
                        tag_link_row_label(&row)
                    ));
                    skipped += 1;
                    continue;
                }
            }

            match self.apply_tag_to_entity(entity_type.clone(), entity_id.clone(), tag_id.clone()) {
                Ok(()) => {
                    let target_link = storage::tags::get_active_target_tag_link(
                        &self.db.conn,
                        &entity_type,
                        &entity_id,
                        &tag_id,
                    )?
                    .ok_or_else(|| {
                        CrmError::Database(
                            "Created tag link snapshot could not be found".to_string(),
                        )
                    })?;
                    rollback_actions.push(import_rollback::ImportRollbackAction::created_tag_link(
                        row_number,
                        &target_link,
                    ));
                    let audit_entity_id =
                        tag_link_rollback_entity_id(&entity_type, &entity_id, &tag_id);
                    if let Err(e) = storage::audit::record_audit(
                        &self.db.conn,
                        ACTOR_IMPORT,
                        None,
                        "import_row",
                        Some("tag_link"),
                        Some(&audit_entity_id),
                        None,
                        None,
                        &self.device_id,
                    ) {
                        errors.push(format!("Row {}: audit record failed: {}", row_number, e));
                    }
                    created += 1;
                }
                Err(e) => {
                    errors.push(format!(
                        "Row {}: {} ({})",
                        row_number,
                        e,
                        tag_link_row_label(&row)
                    ));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            merged,
            skipped,
            errors,
            rollback_plan: import_rollback::ImportRollbackPlan::from_actions(rollback_actions),
        })
    }

    pub fn preflight_tag_links_csv_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_links_csv_with_row_numbers(file_content.as_slice())?;
        self.preflight_tag_link_rows(rows)
    }

    pub fn preflight_tag_links_csv_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_links_csv_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_tag_link_rows(rows)
    }

    pub fn preflight_tag_links_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_links_json_with_row_numbers(file_content.as_slice())?;
        self.preflight_tag_link_rows(rows)
    }

    pub fn preflight_tag_links_json_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_tag_links_json_with_mapping(file_content.as_slice(), &mapping)?;
        self.preflight_tag_link_rows(rows)
    }

    fn preflight_tag_link_rows(
        &self,
        rows: Vec<(usize, TagLinkCsvRow)>,
    ) -> CrmResult<ImportPreflightReport> {
        for (row_number, row) in &rows {
            validate_tag_link_import_row(&self.db.conn, row)
                .map_err(|e| CrmError::InvalidInput(format!("Row {}: {}", row_number, e)))?;
        }

        Ok(import_preflight_report("tag_links", rows.len(), Vec::new()))
    }

    pub fn export_tag_links_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_tag_link_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_tag_links_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_tag_links_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_tag_link_rows()?;
        let count = rows.len() as u32;
        write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_tag_link_rows(&self) -> CrmResult<Vec<TagLinkCsvRow>> {
        Ok(storage::tags::list_active_tag_links(&self.db.conn)?
            .into_iter()
            .map(|link| TagLinkCsvRow {
                entity_type: link.entity_type,
                entity_id: link.entity_id,
                tag_id: link.tag_id,
            })
            .collect())
    }

    pub fn import_organizations_csv(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_organizations_csv_with_options(file_path, ImportOptions::default())
    }

    pub fn import_organizations_csv_with_options(
        &mut self,
        file_path: &str,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_csv_with_row_numbers(file_content.as_slice())?;
        self.import_organization_rows(rows, options)
    }

    pub fn import_organizations_csv_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_organizations_csv_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_organizations_csv_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("organization")?;
        let rows = parse_organizations_csv_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.import_organization_rows(rows, options)
    }

    pub fn import_organizations_json(&mut self, file_path: &str) -> CrmResult<ImportResult> {
        self.import_organizations_json_with_options(file_path, ImportOptions::default())
    }

    pub fn import_organizations_json_with_options(
        &mut self,
        file_path: &str,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_json_with_row_numbers(file_content.as_slice())?;
        self.import_organization_rows(rows, options)
    }

    pub fn import_organizations_json_with_mapping(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportResult> {
        self.import_organizations_json_with_mapping_and_options(
            file_path,
            mapping,
            ImportOptions::default(),
        )
    }

    pub fn import_organizations_json_with_mapping_and_options(
        &mut self,
        file_path: &str,
        mapping: ImportColumnMapping,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("organization")?;
        let rows = parse_organizations_json_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.import_organization_rows(rows, options)
    }

    pub fn preview_organizations_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<JsonImportPreview> {
        let file_content = fs::read(file_path)?;
        preview_organizations_json_import(file_content.as_slice())
    }

    fn import_organization_rows(
        &mut self,
        rows: Vec<(usize, OrganizationCsvRow)>,
        options: ImportOptions,
    ) -> CrmResult<ImportResult> {
        let custom_targets = self.custom_field_import_targets("organization")?;
        let custom_rows = rows
            .iter()
            .map(|(row_number, row)| (*row_number, row.custom_fields.clone()))
            .collect::<Vec<_>>();
        validate_import_custom_fields(&custom_rows, &custom_targets)?;

        let mut created = 0u32;
        let mut merged = 0u32;
        let mut skipped = 0u32;
        let mut errors = Vec::new();
        let mut rollback_actions = Vec::new();

        for (row_number, row) in rows {
            if options.merge_duplicates {
                match self.find_unique_organization_import_match(&row) {
                    Ok(Some(organization)) => {
                        match self.merge_organization_import_row(
                            row_number,
                            &organization.id,
                            &row,
                            &custom_targets,
                        ) {
                            Ok(rollback_action) => {
                                if let Some(action) = rollback_action {
                                    rollback_actions.push(action);
                                }
                                if let Err(e) = storage::audit::record_audit(
                                    &self.db.conn,
                                    ACTOR_IMPORT,
                                    None,
                                    "import_row_merge",
                                    Some("organization"),
                                    Some(&organization.id),
                                    None,
                                    None,
                                    &self.device_id,
                                ) {
                                    errors.push(format!(
                                        "Row {}: audit record failed: {}",
                                        row_number, e
                                    ));
                                }
                                merged += 1;
                                continue;
                            }
                            Err(e) => {
                                errors.push(format!("Row {}: {} ({})", row_number, e, row.name));
                                skipped += 1;
                                continue;
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        errors.push(format!("Row {}: {} ({})", row_number, e, row.name));
                        skipped += 1;
                        continue;
                    }
                }
            }

            match self.create_organization(
                row.name.clone(),
                row.email.clone(),
                row.phone.clone(),
                row.website.clone(),
                row.address_line1.clone(),
                row.address_line2.clone(),
                row.city.clone(),
                row.region.clone(),
                row.country.clone(),
                row.postal_code.clone(),
                row.description.clone(),
            ) {
                Ok(organization) => {
                    match custom_field_import_updates(&row.custom_fields, &custom_targets).and_then(
                        |updates| {
                            self.apply_custom_field_import_updates(&organization.id, &updates)?;
                            self.custom_field_snapshot("organization", &organization.id)
                        },
                    ) {
                        Ok(custom_fields) => {
                            rollback_actions.push(
                                import_rollback::ImportRollbackAction::created_organization(
                                    row_number,
                                    &organization,
                                    custom_fields,
                                ),
                            );
                            if let Err(e) = storage::audit::record_audit(
                                &self.db.conn,
                                ACTOR_IMPORT,
                                None,
                                "import_row",
                                Some("organization"),
                                Some(&organization.id),
                                None,
                                None,
                                &self.device_id,
                            ) {
                                errors.push(format!(
                                    "Row {}: audit record failed: {}",
                                    row_number, e
                                ));
                            }
                            created += 1;
                        }
                        Err(e) => {
                            errors.push(format!("Row {}: {} ({})", row_number, e, row.name));
                            skipped += 1;
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Row {}: {} ({})", row_number, e, row.name));
                    skipped += 1;
                }
            }
        }

        Ok(ImportResult {
            created,
            merged,
            skipped,
            errors,
            rollback_plan: import_rollback::ImportRollbackPlan::from_actions(rollback_actions),
        })
    }

    fn find_unique_organization_import_match(
        &self,
        row: &OrganizationCsvRow,
    ) -> CrmResult<Option<storage::organizations::Organization>> {
        let mut matches = BTreeMap::new();

        let name = row.name.trim().to_string();
        if !name.is_empty() {
            for organization in
                storage::organizations::find_active_organizations_by_name(&self.db.conn, &name)?
            {
                matches.insert(organization.id.clone(), organization);
            }
        }

        if let Some(email) = trimmed_optional(&row.email) {
            for organization in
                storage::organizations::find_active_organizations_by_email(&self.db.conn, &email)?
            {
                matches.insert(organization.id.clone(), organization);
            }
        }

        if let Some(phone) = trimmed_optional(&row.phone) {
            for organization in
                storage::organizations::find_active_organizations_by_phone(&self.db.conn, &phone)?
            {
                matches.insert(organization.id.clone(), organization);
            }
        }

        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.into_values().next()),
            _ => Err(CrmError::InvalidInput(format!(
                "duplicate auto-merge skipped because multiple organizations match: {}",
                matches.keys().cloned().collect::<Vec<_>>().join(", ")
            ))),
        }
    }

    fn merge_organization_import_row(
        &mut self,
        row_number: usize,
        organization_id: &str,
        row: &OrganizationCsvRow,
        custom_targets: &BTreeMap<String, String>,
    ) -> CrmResult<Option<import_rollback::ImportRollbackAction>> {
        let existing = self.get_organization(organization_id)?;
        let before_custom_fields = self.custom_field_snapshot("organization", organization_id)?;
        let incoming_custom_fields =
            custom_field_import_updates(&row.custom_fields, custom_targets)?;
        let custom_updates =
            custom_field_auto_merge_updates(&before_custom_fields, &incoming_custom_fields);
        let email = fill_blank_option(&existing.email, row.email.as_ref()).map(Some);
        let phone = fill_blank_option(&existing.phone, row.phone.as_ref()).map(Some);
        let website = fill_blank_option(&existing.website, row.website.as_ref()).map(Some);
        let address_line1 =
            fill_blank_option(&existing.address_line1, row.address_line1.as_ref()).map(Some);
        let address_line2 =
            fill_blank_option(&existing.address_line2, row.address_line2.as_ref()).map(Some);
        let city = fill_blank_option(&existing.city, row.city.as_ref()).map(Some);
        let region = fill_blank_option(&existing.region, row.region.as_ref()).map(Some);
        let country = fill_blank_option(&existing.country, row.country.as_ref()).map(Some);
        let postal_code =
            fill_blank_option(&existing.postal_code, row.postal_code.as_ref()).map(Some);
        let description =
            fill_blank_option(&existing.description, row.description.as_ref()).map(Some);

        if email.is_none()
            && phone.is_none()
            && website.is_none()
            && address_line1.is_none()
            && address_line2.is_none()
            && city.is_none()
            && region.is_none()
            && country.is_none()
            && postal_code.is_none()
            && description.is_none()
            && custom_updates.is_empty()
        {
            return Ok(None);
        }

        let updated = if email.is_some()
            || phone.is_some()
            || website.is_some()
            || address_line1.is_some()
            || address_line2.is_some()
            || city.is_some()
            || region.is_some()
            || country.is_some()
            || postal_code.is_some()
            || description.is_some()
        {
            self.update_organization(
                organization_id,
                None,
                email,
                phone,
                website,
                address_line1,
                address_line2,
                city,
                region,
                country,
                postal_code,
                description,
            )?
        } else {
            existing.clone()
        };
        self.apply_custom_field_import_updates(organization_id, &custom_updates)?;
        let post_custom_fields = self.custom_field_snapshot("organization", organization_id)?;
        Ok(import_rollback::ImportRollbackAction::merged_organization(
            row_number,
            &existing,
            &updated,
            before_custom_fields,
            post_custom_fields,
        ))
    }

    pub fn preflight_organizations_csv_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_csv_with_row_numbers(file_content.as_slice())?;
        self.preflight_organization_rows(rows)
    }

    pub fn preflight_organizations_csv_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("organization")?;
        let rows = parse_organizations_csv_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.preflight_organization_rows(rows)
    }

    pub fn preflight_organizations_json_import(
        &self,
        file_path: &str,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let rows = parse_organizations_json_with_row_numbers(file_content.as_slice())?;
        self.preflight_organization_rows(rows)
    }

    pub fn preflight_organizations_json_import_with_mapping(
        &self,
        file_path: &str,
        mapping: ImportColumnMapping,
    ) -> CrmResult<ImportPreflightReport> {
        let file_content = fs::read(file_path)?;
        let custom_targets = self.custom_field_import_target_keys("organization")?;
        let rows = parse_organizations_json_with_mapping_targets(
            file_content.as_slice(),
            &mapping,
            &custom_targets,
        )?;
        self.preflight_organization_rows(rows)
    }

    fn preflight_organization_rows(
        &self,
        rows: Vec<(usize, OrganizationCsvRow)>,
    ) -> CrmResult<ImportPreflightReport> {
        let custom_targets = self.custom_field_export_targets("organization")?;
        let custom_rows = rows
            .iter()
            .map(|(row_number, row)| (*row_number, row.custom_fields.clone()))
            .collect::<Vec<_>>();
        validate_import_custom_fields(&custom_rows, &custom_targets)?;

        let mut warnings = Vec::new();

        for (row_number, row) in &rows {
            let name = row.name.trim().to_string();
            for organization in
                storage::organizations::find_active_organizations_by_name(&self.db.conn, &name)?
            {
                warnings.push(import_duplicate_warning(
                    "organizations",
                    *row_number,
                    "name",
                    &name,
                    "organization",
                    &organization.id,
                    &organization_display_label(&organization),
                    format!("Name '{}' matches existing organization", name),
                ));
            }

            if let Some(email) = trimmed_optional(&row.email) {
                for organization in storage::organizations::find_active_organizations_by_email(
                    &self.db.conn,
                    &email,
                )? {
                    warnings.push(import_duplicate_warning(
                        "organizations",
                        *row_number,
                        "email",
                        &email,
                        "organization",
                        &organization.id,
                        &organization_display_label(&organization),
                        format!("Email '{}' matches existing organization", email),
                    ));
                }
            }

            if let Some(phone) = trimmed_optional(&row.phone) {
                for organization in storage::organizations::find_active_organizations_by_phone(
                    &self.db.conn,
                    &phone,
                )? {
                    warnings.push(import_duplicate_warning(
                        "organizations",
                        *row_number,
                        "phone",
                        &phone,
                        "organization",
                        &organization.id,
                        &organization_display_label(&organization),
                        format!("Phone '{}' matches existing organization", phone),
                    ));
                }
            }
        }

        Ok(import_preflight_report(
            "organizations",
            rows.len(),
            warnings,
        ))
    }

    pub fn export_organizations_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_organization_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_organizations_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_organizations_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_organization_rows()?;
        let count = rows.len() as u32;
        write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_organization_rows(&self) -> CrmResult<Vec<OrganizationCsvRow>> {
        let custom_targets = self.custom_field_import_targets("organization")?;
        let field_targets = custom_targets
            .iter()
            .map(|(target, field_def_id)| (field_def_id.clone(), target.clone()))
            .collect::<BTreeMap<_, _>>();
        let custom_values = self
            .list_custom_field_values_for_type("organization")?
            .into_iter()
            .map(|value| ((value.entity_id, value.field_def_id), value.value))
            .collect::<BTreeMap<_, _>>();
        let organizations = storage::organizations::list_organizations(&self.db.conn)?;
        Ok(organizations
            .iter()
            .map(|organization| OrganizationCsvRow {
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
                description: organization.description.clone(),
                custom_fields: field_targets
                    .iter()
                    .map(|(field_def_id, target)| {
                        (
                            target.clone(),
                            custom_values
                                .get(&(organization.id.clone(), field_def_id.clone()))
                                .cloned()
                                .unwrap_or_default(),
                        )
                    })
                    .collect(),
            })
            .collect())
    }

    fn set_activity_completion(&mut self, id: &str, completed: bool) -> CrmResult<Activity> {
        let before = storage::activities::get_activity(&self.db.conn, id)?;
        if before.completed == completed {
            return Ok(before);
        }
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let activity = if completed {
            storage::activities::mark_complete(&tx, id)?
        } else {
            storage::activities::mark_incomplete(&tx, id)?
        };
        storage::sync::record_change(
            &tx,
            "activity",
            id,
            "completed",
            Some(if before.completed { "1" } else { "0" }),
            Some(if completed { "1" } else { "0" }),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            if completed {
                "mark_complete"
            } else {
                "mark_incomplete"
            },
            Some("activity"),
            Some(id),
            Some(&before),
            Some(&activity),
            &device_id,
        )?;
        tx.commit()?;
        Ok(activity)
    }
}

fn write_json_export<T>(file_path: &str, rows: &[T]) -> CrmResult<()>
where
    T: Serialize,
{
    let json = serde_json::to_string_pretty(rows)?;
    fs::write(file_path, format!("{json}\n"))?;
    Ok(())
}

fn load_or_create_device_id(db: &storage::Database) -> CrmResult<String> {
    match storage::settings::get_setting(db.connection(), "device_id")? {
        Some(setting) if !setting.value.is_empty() => Ok(setting.value),
        _ => {
            let new_id = new_uuid();
            storage::settings::set_setting(db.connection(), "device_id", &new_id)?;
            Ok(new_id)
        }
    }
}

// Audit records intentionally keep before/after context explicit at call sites.
#[allow(clippy::too_many_arguments)]
pub(super) fn record_audit_json<TBefore, TAfter>(
    conn: &Connection,
    actor_type: &str,
    action: &str,
    entity_type: Option<&str>,
    entity_id: Option<&str>,
    before: Option<TBefore>,
    after: Option<TAfter>,
    device_id: &str,
) -> InternalCrmResult<AuditLogEntry>
where
    TBefore: Serialize,
    TAfter: Serialize,
{
    let before_json = before
        .map(|value| serde_json::to_string(&value))
        .transpose()
        .map_err(CrmError::from)?;
    let after_json = after
        .map(|value| serde_json::to_string(&value))
        .transpose()
        .map_err(CrmError::from)?;

    storage::audit::record_audit(
        conn,
        actor_type,
        None,
        action,
        entity_type,
        entity_id,
        before_json.as_deref(),
        after_json.as_deref(),
        device_id,
    )
}

fn parse_bool(value: Option<&str>) -> bool {
    matches!(value, Some("true") | Some("1"))
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|trimmed| !trimmed.is_empty())
}

fn normalize_optional_update_string(value: Option<Option<String>>) -> Option<Option<String>> {
    value.map(normalize_optional_string)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn create_activity_in_transaction(
    conn: &Connection,
    device_id: &str,
    activity_type: &str,
    title: &str,
    description: Option<&str>,
    due_date: Option<&str>,
    contact_id: Option<&str>,
    deal_id: Option<&str>,
) -> CrmResult<Activity> {
    activity_engine::validate_activity_for_create(title, activity_type)?;
    if let Some(contact_id) = contact_id {
        storage::contacts::get_contact(conn, contact_id)?;
    }
    if let Some(deal_id) = deal_id {
        storage::deals::get_deal(conn, deal_id)?;
    }

    let activity = storage::activities::create_activity(
        conn,
        activity_type,
        title,
        description.unwrap_or(""),
        due_date,
        contact_id,
        deal_id,
        device_id,
    )?;
    create_activity_links_for_legacy_mirrors(conn, &activity, device_id)?;
    storage::sync::record_change(
        conn,
        "activity",
        &activity.id,
        "__create__",
        None,
        Some(&activity.id),
        device_id,
    )?;
    record_audit_json(
        conn,
        ACTOR_DESKTOP_APP,
        "create",
        Some("activity"),
        Some(&activity.id),
        None::<&()>,
        Some(&activity),
        device_id,
    )?;
    Ok(activity)
}

fn trimmed_optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|trimmed| !trimmed.is_empty())
        .map(str::to_string)
}

fn fill_blank_string(existing: &str, incoming: Option<&String>) -> Option<String> {
    if !existing.trim().is_empty() {
        return None;
    }

    incoming
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn fill_blank_option(existing: &Option<String>, incoming: Option<&String>) -> Option<String> {
    if existing
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return None;
    }

    incoming
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn fill_zero_value(existing: f64, incoming: Option<&String>) -> Option<f64> {
    if existing != 0.0 {
        return None;
    }

    incoming
        .and_then(|value| value.trim().parse::<f64>().ok())
        .filter(|value| *value != 0.0)
}

fn custom_field_targets(definitions: Vec<CustomFieldDefinition>) -> BTreeMap<String, String> {
    let mut name_counts = BTreeMap::new();
    for definition in &definitions {
        *name_counts
            .entry(definition.field_name.clone())
            .or_insert(0usize) += 1;
    }

    definitions
        .into_iter()
        .map(|definition| {
            let duplicate_name = name_counts
                .get(&definition.field_name)
                .copied()
                .unwrap_or_default()
                > 1;
            let target = custom_field_target(
                &definition.field_name,
                duplicate_name.then_some(definition.id.as_str()),
            );
            (target, definition.id)
        })
        .collect()
}

fn custom_field_target(field_name: &str, field_id: Option<&str>) -> String {
    let escaped_field_name = escape_custom_field_name(field_name);
    match field_id {
        Some(id) => format!("{CUSTOM_FIELD_PREFIX}{escaped_field_name}#{id}"),
        None => format!("{CUSTOM_FIELD_PREFIX}{escaped_field_name}"),
    }
}

fn escape_custom_field_name(field_name: &str) -> String {
    let mut escaped = String::with_capacity(field_name.len());
    for character in field_name.chars() {
        match character {
            '%' => escaped.push_str("%25"),
            '#' => escaped.push_str("%23"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn validate_import_custom_fields(
    rows: &[(usize, BTreeMap<String, String>)],
    targets: &BTreeMap<String, String>,
) -> CrmResult<()> {
    for (row_number, custom_fields) in rows {
        for target in custom_fields.keys() {
            if !targets.contains_key(target) {
                return Err(CrmError::InvalidInput(format!(
                    "Row {} maps to unsupported custom field '{}'",
                    row_number, target
                )));
            }
        }
    }

    Ok(())
}

fn custom_field_import_updates(
    row_custom_fields: &BTreeMap<String, String>,
    targets: &BTreeMap<String, String>,
) -> CrmResult<BTreeMap<String, String>> {
    let mut updates = BTreeMap::new();

    for (target, value) in row_custom_fields {
        let Some(field_def_id) = targets.get(target) else {
            return Err(CrmError::InvalidInput(format!(
                "Unsupported custom field target '{}'",
                target
            )));
        };

        let value = value.trim();
        if !value.is_empty() {
            updates.insert(field_def_id.clone(), value.to_string());
        }
    }

    Ok(updates)
}

fn custom_field_auto_merge_updates(
    existing: &BTreeMap<String, String>,
    incoming: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    incoming
        .iter()
        .filter(|(field_def_id, value)| {
            !value.trim().is_empty()
                && existing
                    .get(*field_def_id)
                    .map(|existing| existing.trim().is_empty())
                    .unwrap_or(true)
        })
        .map(|(field_def_id, value)| (field_def_id.clone(), value.trim().to_string()))
        .collect()
}

fn import_preflight_report(
    entity_type: &str,
    total_rows: usize,
    warnings: Vec<ImportDuplicateWarning>,
) -> ImportPreflightReport {
    ImportPreflightReport {
        entity_type: entity_type.to_string(),
        total_rows: total_rows as u32,
        duplicate_warning_count: warnings.len() as u32,
        warnings,
    }
}

fn validate_note_import_row(
    conn: &Connection,
    _row_number: usize,
    row: &NoteCsvRow,
) -> CrmResult<(String, String, String)> {
    let entity_type = normalize_note_import_entity_type(&row.entity_type)?;
    let entity_id = normalize_required_note_import_field(&row.entity_id, "entity_id")?;
    let content = normalize_required_note_import_field(&row.content, "content")?;
    ensure_note_import_entity_exists(conn, &entity_type, &entity_id)?;
    Ok((entity_type, entity_id, content))
}

fn normalize_note_import_entity_type(value: &str) -> CrmResult<String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "contact" => Ok("contact".to_string()),
        "organization" => Ok("organization".to_string()),
        "deal" => Ok("deal".to_string()),
        "activity" => Ok("activity".to_string()),
        "" => Err(CrmError::InvalidInput(
            "entity_type is required".to_string(),
        )),
        other => Err(CrmError::InvalidInput(format!(
            "Unsupported entity_type '{}'",
            other
        ))),
    }
}

fn normalize_required_note_import_field(value: &str, field: &str) -> CrmResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CrmError::InvalidInput(format!("{} is required", field)));
    }
    Ok(trimmed.to_string())
}

fn ensure_note_import_entity_exists(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> CrmResult<()> {
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

fn note_row_label(row: &NoteCsvRow) -> String {
    let entity_type = row.entity_type.trim();
    let entity_id = row.entity_id.trim();
    if entity_type.is_empty() && entity_id.is_empty() {
        "note".to_string()
    } else {
        format!("{}:{}", entity_type, entity_id)
    }
}

fn validate_tag_definition_import_row(
    row: &TagDefinitionCsvRow,
) -> CrmResult<(String, Option<String>)> {
    let name = normalize_required_note_import_field(&row.name, "name")?;
    let color = row
        .color
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    Ok((name, color))
}

struct CustomFieldDefinitionImportValues {
    entity_type: String,
    field_name: String,
    field_type: String,
    field_options: Option<String>,
    sort_order: i32,
}

fn validate_custom_field_definition_import_row(
    row: &CustomFieldDefinitionCsvRow,
) -> CrmResult<CustomFieldDefinitionImportValues> {
    const ENTITY_TYPES: [&str; 4] = ["contact", "deal", "activity", "organization"];
    const FIELD_TYPES: [&str; 5] = ["text", "number", "date", "boolean", "select"];

    let entity_type = row.entity_type.trim().to_ascii_lowercase();
    if !ENTITY_TYPES.contains(&entity_type.as_str()) {
        return Err(CrmError::InvalidInput(format!(
            "Invalid entity_type '{}'. Expected one of: {}",
            row.entity_type,
            ENTITY_TYPES.join(", ")
        )));
    }

    let field_name = normalize_required_note_import_field(&row.field_name, "field_name")?;
    let field_type = row.field_type.trim().to_ascii_lowercase();
    if !FIELD_TYPES.contains(&field_type.as_str()) {
        return Err(CrmError::InvalidInput(format!(
            "Invalid field_type '{}'. Expected one of: {}",
            row.field_type,
            FIELD_TYPES.join(", ")
        )));
    }

    let field_options = row
        .field_options
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    validate_custom_field_definition_options(&field_type, field_options.as_deref())?;

    let sort_order = match row
        .sort_order
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => value.parse::<i32>().map_err(|_| {
            CrmError::InvalidInput(format!("sort_order must be an integer, got '{}'", value))
        })?,
        None => 0,
    };

    Ok(CustomFieldDefinitionImportValues {
        entity_type,
        field_name,
        field_type,
        field_options,
        sort_order,
    })
}

fn validate_custom_field_definition_options(
    field_type: &str,
    field_options: Option<&str>,
) -> CrmResult<()> {
    if field_type == "select" {
        let raw = field_options.ok_or_else(|| {
            CrmError::InvalidInput("field_options is required for select custom fields".to_string())
        })?;
        validate_custom_field_definition_options_json(raw)?;
        return Ok(());
    }

    if let Some(raw) = field_options {
        validate_custom_field_definition_options_json(raw)?;
    }

    Ok(())
}

fn validate_custom_field_definition_options_json(raw: &str) -> CrmResult<()> {
    let parsed: serde_json::Value = serde_json::from_str(raw)?;
    let items = parsed
        .as_array()
        .ok_or_else(|| CrmError::InvalidInput("field_options must be a JSON array".to_string()))?;

    if items.is_empty() {
        return Err(CrmError::InvalidInput(
            "field_options must include at least one option".to_string(),
        ));
    }

    if !items.iter().all(|item| item.is_string()) {
        return Err(CrmError::InvalidInput(
            "field_options entries must all be strings".to_string(),
        ));
    }

    Ok(())
}

fn custom_field_definition_import_key(entity_type: &str, field_name: &str) -> String {
    format!("{}:{}", entity_type, field_name)
}

fn custom_field_definition_matches_import(
    definition: &CustomFieldDefinition,
    values: &CustomFieldDefinitionImportValues,
) -> bool {
    definition.entity_type == values.entity_type
        && definition.field_name == values.field_name
        && definition.field_type == values.field_type
        && definition.field_options == values.field_options
        && definition.sort_order == values.sort_order
}

fn validate_tag_link_import_row(
    conn: &Connection,
    row: &TagLinkCsvRow,
) -> CrmResult<(String, String, String)> {
    let entity_type = normalize_note_import_entity_type(&row.entity_type)?;
    let entity_id = normalize_required_note_import_field(&row.entity_id, "entity_id")?;
    let tag_id = normalize_required_note_import_field(&row.tag_id, "tag_id")?;
    ensure_note_import_entity_exists(conn, &entity_type, &entity_id)?;
    storage::tags::get_tag(conn, &tag_id)?;
    Ok((entity_type, entity_id, tag_id))
}

fn tag_link_row_label(row: &TagLinkCsvRow) -> String {
    let entity_type = row.entity_type.trim();
    let entity_id = row.entity_id.trim();
    let tag_id = row.tag_id.trim();
    if entity_type.is_empty() && entity_id.is_empty() && tag_id.is_empty() {
        "tag link".to_string()
    } else {
        tag_link_rollback_entity_id(entity_type, entity_id, tag_id)
    }
}

fn tag_link_rollback_entity_id(entity_type: &str, entity_id: &str, tag_id: &str) -> String {
    format!("{}:{}:{}", entity_type, entity_id, tag_id)
}

#[allow(clippy::too_many_arguments)]
fn import_duplicate_warning(
    entity_type: &str,
    row_number: usize,
    match_type: &str,
    csv_value: &str,
    existing_entity_type: &str,
    existing_entity_id: &str,
    existing_display_label: &str,
    reason: String,
) -> ImportDuplicateWarning {
    ImportDuplicateWarning {
        entity_type: entity_type.to_string(),
        row_number: row_number as u32,
        match_type: match_type.to_string(),
        csv_value: csv_value.to_string(),
        existing_entity_type: existing_entity_type.to_string(),
        existing_entity_id: existing_entity_id.to_string(),
        existing_display_label: existing_display_label.to_string(),
        reason,
    }
}

fn contact_display_label(contact: &storage::contacts::Contact) -> String {
    let name = [contact.first_name.as_str(), contact.last_name.as_str()]
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if !name.trim().is_empty() {
        name
    } else if !contact.org_name.trim().is_empty() {
        contact.org_name.clone()
    } else if !contact.email.trim().is_empty() {
        contact.email.clone()
    } else if !contact.phone.trim().is_empty() {
        contact.phone.clone()
    } else {
        contact.id.clone()
    }
}

fn organization_display_label(organization: &storage::organizations::Organization) -> String {
    if organization.name.trim().is_empty() {
        organization.id.clone()
    } else {
        organization.name.clone()
    }
}

fn deal_display_label(deal: &storage::deals::Deal) -> String {
    if deal.title.trim().is_empty() {
        deal.id.clone()
    } else {
        deal.title.clone()
    }
}

fn sync_status_from_settings(
    sync_enabled: &str,
    sync_url: &str,
    last_sync_at: Option<String>,
    pending_changes: u32,
) -> CrmResult<SyncStatus> {
    if !parse_bool(Some(sync_enabled)) {
        return Ok(SyncStatus {
            state: "idle".to_string(),
            last_sync_at,
            error_message: None,
            pending_changes,
        });
    }

    if sync_url.trim().is_empty() {
        return Ok(SyncStatus {
            state: "error".to_string(),
            last_sync_at,
            error_message: Some("Sync URL is not configured.".to_string()),
            pending_changes,
        });
    }

    Ok(SyncStatus {
        state: "not_implemented".to_string(),
        last_sync_at,
        error_message: Some(
            "Multi-device sync transport is not implemented yet. Local changes are recorded to the changelog for future sync.".to_string(),
        ),
        pending_changes,
    })
}
