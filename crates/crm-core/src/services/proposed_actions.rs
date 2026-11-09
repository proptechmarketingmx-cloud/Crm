use std::{fs, io::BufWriter};

use crate::audit::{ACTOR_DESKTOP_APP, ACTOR_MCP_CLIENT};
use crate::result::CrmResult;
use crate::storage::{
    self,
    activities::{Activity, ActivityLinkEntityType},
    proposed_actions::ProposedAction,
};
use crate::utils::{
    csv::{write_proposed_actions_csv, ProposedActionCsvRow},
    errors::CrmError,
};
use serde::Deserialize;

use super::activity_relationships::add_activity_link_in_transaction;
use super::external_client_permissions::{
    ensure_external_client_draft_permission, external_client_entity_scope,
    required_external_client_field,
};
use super::{create_activity_in_transaction, record_audit_json, CrmCore};

const CREATE_ACTIVITY_DRAFT_TOOL: &str = "create_activity_draft";
const CREATE_ACTIVITY_COMPATIBLE_ACTION_TYPE: &str = "create_activity";

impl CrmCore {
    pub fn list_pending_proposed_actions(&self) -> CrmResult<Vec<ProposedAction>> {
        storage::proposed_actions::list_pending_proposed_actions(&self.db.conn)
    }

    pub fn export_proposed_actions_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_proposed_action_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_proposed_actions_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_proposed_actions_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_proposed_action_rows()?;
        let count = rows.len() as u32;
        super::write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_proposed_action_rows(&self) -> CrmResult<Vec<ProposedActionCsvRow>> {
        Ok(
            storage::proposed_actions::list_all_proposed_actions(&self.db.conn)?
                .into_iter()
                .map(proposed_action_export_row)
                .collect(),
        )
    }

    pub fn approve_proposed_action(&mut self, id: String) -> CrmResult<ProposedAction> {
        self.decide_proposed_action(id, ProposedActionDecision::Approve)
    }

    pub fn reject_proposed_action(&mut self, id: String) -> CrmResult<ProposedAction> {
        self.decide_proposed_action(id, ProposedActionDecision::Reject)
    }

    // Preserve the placeholder API shape until external-client execution is implemented.
    #[allow(clippy::too_many_arguments)]
    pub fn create_external_proposed_action_stub(
        &mut self,
        client_id: Option<String>,
        action_type: String,
        tool_name: String,
        entity_type: Option<String>,
        entity_id: Option<String>,
        input_json: String,
        proposed_output_json: Option<String>,
    ) -> CrmResult<ProposedAction> {
        let client_id = client_id
            .as_deref()
            .map(|id| required_external_client_field("client_id", id))
            .transpose()?;
        let tool_name = if client_id.is_some() {
            required_external_client_field("tool_name", &tool_name)?
        } else {
            tool_name
        };
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        if let Some(client_id) = client_id.as_deref() {
            if let Err(error) = ensure_external_client_draft_permission(
                &tx,
                ACTOR_MCP_CLIENT,
                &device_id,
                client_id,
                &tool_name,
                external_client_entity_scope(entity_type.as_deref(), entity_id.as_deref()),
            ) {
                tx.commit()?;
                return Err(error);
            }
        }
        let proposed_action = storage::proposed_actions::create_proposed_action(
            &tx,
            client_id.as_deref(),
            &action_type,
            &tool_name,
            entity_type.as_deref(),
            entity_id.as_deref(),
            &input_json,
            proposed_output_json.as_deref(),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_MCP_CLIENT,
            "propose_action",
            Some("proposed_action"),
            Some(&proposed_action.id),
            None::<&()>,
            Some(&proposed_action),
            &device_id,
        )?;
        tx.commit()?;
        Ok(proposed_action)
    }

    fn decide_proposed_action(
        &mut self,
        id: String,
        decision: ProposedActionDecision,
    ) -> CrmResult<ProposedAction> {
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let before = storage::proposed_actions::get_proposed_action(&tx, &id)?;
        let proposed_action = match decision {
            ProposedActionDecision::Approve => {
                let before_action = before.as_ref().ok_or_else(|| {
                    CrmError::NotFound(format!("Proposed action '{}' was not found", id))
                })?;
                if before_action.status != "pending" {
                    return Err(CrmError::InvalidInput(format!(
                        "Proposed action '{}' must be pending before it can be approved or rejected; current status is '{}'",
                        id, before_action.status
                    )));
                }
                execute_create_activity_draft(&tx, before_action, &device_id)?;
                let executed = storage::proposed_actions::execute_proposed_action(&tx, &id)?;
                record_audit_json(
                    &tx,
                    ACTOR_DESKTOP_APP,
                    "execute_proposed_action",
                    Some("proposed_action"),
                    Some(&executed.id),
                    before.as_ref(),
                    Some(&executed),
                    &device_id,
                )?;
                executed
            }
            ProposedActionDecision::Reject => {
                storage::proposed_actions::reject_proposed_action(&tx, &id)?
            }
        };
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            decision.audit_action(),
            Some("proposed_action"),
            Some(&proposed_action.id),
            before.as_ref(),
            Some(&proposed_action),
            &device_id,
        )?;
        tx.commit()?;
        Ok(proposed_action)
    }
}

fn proposed_action_export_row(action: ProposedAction) -> ProposedActionCsvRow {
    let external_client_id = action.client_id.clone();
    let payload_json = action.input_json.clone();
    let decided_at = action
        .approved_at
        .clone()
        .or_else(|| action.rejected_at.clone());

    ProposedActionCsvRow {
        id: action.id,
        external_client_id,
        client_id: action.client_id,
        tool_name: action.tool_name,
        action_type: action.action_type,
        entity_type: action.entity_type,
        entity_id: action.entity_id,
        payload_json,
        input_json: action.input_json,
        proposed_output_json: action.proposed_output_json,
        status: action.status,
        created_at: action.created_at,
        decided_at,
        approved_at: action.approved_at,
        rejected_at: action.rejected_at,
        executed_at: action.executed_at,
        error_message: None,
        device_id: action.device_id,
    }
}

fn execute_create_activity_draft(
    conn: &rusqlite::Connection,
    proposed_action: &ProposedAction,
    device_id: &str,
) -> CrmResult<Activity> {
    validate_create_activity_draft_identity(proposed_action)?;

    let draft = CreateActivityDraftExecution::try_from(proposed_action)?;
    let activity = create_activity_in_transaction(
        conn,
        device_id,
        &draft.activity_type,
        &draft.title,
        draft.description.as_deref(),
        draft.due_date.as_deref(),
        draft.contact_id.as_deref(),
        draft.deal_id.as_deref(),
    )?;
    for organization_id in draft.organization_ids {
        add_activity_link_in_transaction(
            conn,
            &activity.id,
            ActivityLinkEntityType::Organization,
            &organization_id,
            device_id,
        )?;
    }
    Ok(activity)
}

fn validate_create_activity_draft_identity(proposed_action: &ProposedAction) -> CrmResult<()> {
    if proposed_action.tool_name != CREATE_ACTIVITY_DRAFT_TOOL {
        return Err(CrmError::InvalidInput(format!(
            "Unsupported proposed action tool for approval execution: tool_name='{}', action_type='{}'",
            proposed_action.tool_name, proposed_action.action_type
        )));
    }

    match proposed_action.action_type.as_str() {
        CREATE_ACTIVITY_DRAFT_TOOL | CREATE_ACTIVITY_COMPATIBLE_ACTION_TYPE => Ok(()),
        other => Err(CrmError::InvalidInput(format!(
            "Unsupported proposed action action_type '{}' for tool '{}'; expected '{}' or compatible '{}'",
            other,
            proposed_action.tool_name,
            CREATE_ACTIVITY_DRAFT_TOOL,
            CREATE_ACTIVITY_COMPATIBLE_ACTION_TYPE
        ))),
    }
}

#[derive(Debug)]
struct CreateActivityDraftExecution {
    title: String,
    activity_type: String,
    description: Option<String>,
    due_date: Option<String>,
    contact_id: Option<String>,
    deal_id: Option<String>,
    organization_ids: Vec<String>,
}

impl TryFrom<&ProposedAction> for CreateActivityDraftExecution {
    type Error = CrmError;

    fn try_from(proposed_action: &ProposedAction) -> Result<Self, Self::Error> {
        let input: CreateActivityDraftInput = serde_json::from_str(&proposed_action.input_json)?;
        let title = required_json_string("title", input.title.as_deref())?;
        let activity_type = optional_json_string(input.activity_type.as_deref())
            .unwrap_or_else(|| "task".to_string());
        let description = optional_json_string(input.description.as_deref());
        let due_date = optional_json_string(input.due_at.as_deref());

        let mut contact_id = None;
        let mut deal_id = None;
        let mut organization_ids = Vec::new();
        for linked_entity in input.linked_entities {
            let entity_type = ActivityLinkEntityType::try_from(linked_entity.entity_type.as_str())?;
            let entity_id = required_json_string(
                "linked_entities[].entity_id",
                Some(&linked_entity.entity_id),
            )?;
            match entity_type {
                ActivityLinkEntityType::Contact => {
                    if contact_id.replace(entity_id).is_some() {
                        return Err(CrmError::InvalidInput(
                            "create_activity_draft supports at most one linked contact".to_string(),
                        ));
                    }
                }
                ActivityLinkEntityType::Deal => {
                    if deal_id.replace(entity_id).is_some() {
                        return Err(CrmError::InvalidInput(
                            "create_activity_draft supports at most one linked deal".to_string(),
                        ));
                    }
                }
                ActivityLinkEntityType::Organization => organization_ids.push(entity_id),
            }
        }

        Ok(Self {
            title,
            activity_type,
            description,
            due_date,
            contact_id,
            deal_id,
            organization_ids,
        })
    }
}

#[derive(Debug, Deserialize)]
struct CreateActivityDraftInput {
    title: Option<String>,
    activity_type: Option<String>,
    description: Option<String>,
    due_at: Option<String>,
    #[serde(default)]
    linked_entities: Vec<CreateActivityDraftLinkedEntityInput>,
}

#[derive(Debug, Deserialize)]
struct CreateActivityDraftLinkedEntityInput {
    entity_type: String,
    entity_id: String,
}

fn required_json_string(field: &str, value: Option<&str>) -> CrmResult<String> {
    optional_json_string(value).ok_or_else(|| {
        CrmError::InvalidInput(format!("create_activity_draft {} is required", field))
    })
}

fn optional_json_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|trimmed| !trimmed.is_empty())
        .map(str::to_string)
}

#[derive(Debug, Clone, Copy)]
enum ProposedActionDecision {
    Approve,
    Reject,
}

impl ProposedActionDecision {
    fn audit_action(self) -> &'static str {
        match self {
            ProposedActionDecision::Approve => "approve_proposed_action",
            ProposedActionDecision::Reject => "reject_proposed_action",
        }
    }
}
