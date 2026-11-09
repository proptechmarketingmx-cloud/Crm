use std::{fs, io::BufWriter};

use rusqlite::Connection;
use serde::Serialize;

use crate::audit::{ACTOR_DESKTOP_APP, ACTOR_MCP_CLIENT};
use crate::permissions::{
    evaluate_tool_draft_permission, evaluate_tool_read_permission, ExternalClientPermissionMode,
    ToolPermissionEvaluation,
};
use crate::result::CrmResult;
use crate::storage::audit::AuditLogEntry;
use crate::storage::{
    self, external_client_permissions::ExternalClientPermission, external_clients::ExternalClient,
};
use crate::utils::{
    csv::{write_external_client_permissions_csv, ExternalClientPermissionCsvRow},
    errors::CrmError,
};

use super::{record_audit_json, CrmCore};

const EVALUATE_EXTERNAL_CLIENT_READ_PERMISSION_ACTION: &str =
    "evaluate_external_client_read_permission";
const EVALUATE_EXTERNAL_CLIENT_DRAFT_PERMISSION_ACTION: &str =
    "evaluate_external_client_draft_permission";
const RECORD_EXTERNAL_CLIENT_TOOL_RESULT_ACTION: &str = "record_external_client_tool_result";
const EXTERNAL_CLIENT_ENTITY_TYPE: &str = "external_client";

impl CrmCore {
    pub fn list_external_client_permissions(
        &self,
        client_id: &str,
    ) -> CrmResult<Vec<ExternalClientPermission>> {
        let client_id = required_external_client_field("client_id", client_id)?;
        require_existing_external_client(&self.db.conn, &client_id)?;

        storage::external_client_permissions::list_permissions_for_client(&self.db.conn, &client_id)
    }

    pub fn export_external_client_permissions_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_external_client_permission_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_external_client_permissions_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_external_client_permissions_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_external_client_permission_rows()?;
        let count = rows.len() as u32;
        super::write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_external_client_permission_rows(
        &self,
    ) -> CrmResult<Vec<ExternalClientPermissionCsvRow>> {
        Ok(
            storage::external_client_permissions::list_all_permissions_for_export(&self.db.conn)?
                .into_iter()
                .map(external_client_permission_export_row)
                .collect(),
        )
    }

    pub fn upsert_external_client_tool_permission(
        &mut self,
        client_id: &str,
        tool_name: &str,
        can_read: bool,
        can_write: bool,
        requires_confirmation: bool,
    ) -> CrmResult<ExternalClientPermission> {
        let client_id = required_external_client_field("client_id", client_id)?;
        let tool_name = required_external_client_field("tool_name", tool_name)?;
        validate_initial_permission_write(can_write, requires_confirmation)?;

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        require_initial_external_client_mode(&tx, &client_id)?;
        let before = storage::external_client_permissions::get_permission_for_tool(
            &tx, &client_id, &tool_name,
        )?;
        let permission = storage::external_client_permissions::upsert_permission_for_tool(
            &tx,
            &client_id,
            &tool_name,
            can_read,
            can_write,
            requires_confirmation,
        )?;

        if before.as_ref() != Some(&permission) {
            let sync_field = if before.is_some() {
                "__update__"
            } else {
                "__create__"
            };
            storage::sync::record_change(
                &tx,
                "external_client_permission",
                &permission.id,
                sync_field,
                before.as_ref().map(|old| old.id.as_str()),
                Some(&permission.id),
                &device_id,
            )?;
            record_audit_json(
                &tx,
                ACTOR_DESKTOP_APP,
                "upsert_external_client_permission",
                Some("external_client_permission"),
                Some(&permission.id),
                before.as_ref(),
                Some(&permission),
                &device_id,
            )?;
        }

        tx.commit()?;
        Ok(permission)
    }

    pub fn evaluate_external_client_tool_read_permission(
        &self,
        client_id: &str,
        tool_name: &str,
    ) -> CrmResult<ToolPermissionEvaluation> {
        let client_id = required_external_client_field("client_id", client_id)?;
        let tool_name = required_external_client_field("tool_name", tool_name)?;

        let tx = self.db.conn.unchecked_transaction()?;
        let outcome = evaluate_external_client_permission_with_audit(
            &tx,
            ACTOR_DESKTOP_APP,
            &self.device_id,
            ExternalClientAccessKind::Read,
            &client_id,
            &tool_name,
            None,
        );
        tx.commit()?;
        outcome
    }

    pub fn evaluate_external_client_draft_permission(
        &self,
        client_id: &str,
        tool_name: &str,
    ) -> CrmResult<ToolPermissionEvaluation> {
        let client_id = required_external_client_field("client_id", client_id)?;
        let tool_name = required_external_client_field("tool_name", tool_name)?;

        let tx = self.db.conn.unchecked_transaction()?;
        let outcome = evaluate_external_client_permission_with_audit(
            &tx,
            ACTOR_DESKTOP_APP,
            &self.device_id,
            ExternalClientAccessKind::Draft,
            &client_id,
            &tool_name,
            None,
        );
        tx.commit()?;
        outcome
    }

    pub fn record_external_client_tool_result(
        &self,
        client_id: &str,
        tool_name: &str,
        access_kind: &str,
        result_count: u32,
        entity_type: Option<&str>,
        entity_id: Option<&str>,
    ) -> CrmResult<AuditLogEntry> {
        let client_id = required_external_client_field("client_id", client_id)?;
        let tool_name = required_external_client_field("tool_name", tool_name)?;
        let access_kind = ExternalClientResultAccessKind::parse(access_kind)?;
        require_existing_external_client(&self.db.conn, &client_id)?;
        let entity_scope = external_client_entity_scope(entity_type, entity_id);
        let context = ExternalClientToolResultAudit {
            client_id: &client_id,
            tool_name: &tool_name,
            access_kind: access_kind.as_str(),
            status: "succeeded",
            result_count,
            entity_scope,
        };

        record_audit_json(
            &self.db.conn,
            ACTOR_MCP_CLIENT,
            RECORD_EXTERNAL_CLIENT_TOOL_RESULT_ACTION,
            Some(EXTERNAL_CLIENT_ENTITY_TYPE),
            Some(&client_id),
            None::<&()>,
            Some(&context),
            &self.device_id,
        )
    }
}

fn external_client_permission_export_row(
    permission: ExternalClientPermission,
) -> ExternalClientPermissionCsvRow {
    ExternalClientPermissionCsvRow {
        id: permission.id,
        client_id: permission.client_id,
        tool_name: permission.tool_name,
        can_read: permission.can_read,
        can_write: permission.can_write,
        requires_confirmation: permission.requires_confirmation,
        created_at: permission.created_at,
        updated_at: permission.updated_at,
    }
}

pub(super) fn ensure_external_client_draft_permission(
    conn: &Connection,
    actor_type: &str,
    device_id: &str,
    client_id: &str,
    tool_name: &str,
    entity_scope: Option<ExternalClientEntityScope<'_>>,
) -> CrmResult<ToolPermissionEvaluation> {
    let evaluation = evaluate_external_client_permission_with_audit(
        conn,
        actor_type,
        device_id,
        ExternalClientAccessKind::Draft,
        client_id,
        tool_name,
        entity_scope,
    )?;
    if evaluation.allowed {
        return Ok(evaluation);
    }

    Err(CrmError::InvalidInput(format!(
        "External client '{}' may not create draft proposed actions for tool '{}': {}",
        client_id,
        tool_name,
        evaluation.reason.as_str()
    )))
}

pub(super) fn required_external_client_field(field: &str, value: &str) -> CrmResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CrmError::InvalidInput(format!(
            "External client {} is required",
            field
        )));
    }

    Ok(trimmed.to_string())
}

pub(super) fn external_client_entity_scope<'a>(
    entity_type: Option<&'a str>,
    entity_id: Option<&'a str>,
) -> Option<ExternalClientEntityScope<'a>> {
    if entity_type.is_none() && entity_id.is_none() {
        return None;
    }

    Some(ExternalClientEntityScope {
        entity_type,
        entity_id,
    })
}

fn evaluate_external_client_permission_with_audit(
    conn: &Connection,
    actor_type: &str,
    device_id: &str,
    access_kind: ExternalClientAccessKind,
    client_id: &str,
    tool_name: &str,
    entity_scope: Option<ExternalClientEntityScope<'_>>,
) -> CrmResult<ToolPermissionEvaluation> {
    let result =
        evaluate_external_client_permission_decision(conn, access_kind, client_id, tool_name);

    let audit_result = record_external_client_permission_evaluation_audit(
        conn,
        actor_type,
        device_id,
        access_kind,
        client_id,
        tool_name,
        entity_scope,
        result.as_ref(),
    );

    match (result, audit_result) {
        (Ok(evaluation), Ok(_)) => Ok(evaluation),
        (Ok(_), Err(audit_error)) => Err(audit_error),
        (Err(evaluation_error), _) => Err(evaluation_error),
    }
}

fn evaluate_external_client_permission_decision(
    conn: &Connection,
    access_kind: ExternalClientAccessKind,
    client_id: &str,
    tool_name: &str,
) -> CrmResult<ToolPermissionEvaluation> {
    let mode = external_client_mode_for_evaluation(conn, client_id)?;
    let permission =
        storage::external_client_permissions::get_permission_for_tool(conn, client_id, tool_name)?;

    let grant = permission.as_ref().map(ExternalClientPermission::grant);
    Ok(match access_kind {
        ExternalClientAccessKind::Read => evaluate_tool_read_permission(mode, tool_name, grant),
        ExternalClientAccessKind::Draft => evaluate_tool_draft_permission(mode, tool_name, grant),
    })
}

#[allow(clippy::too_many_arguments)]
fn record_external_client_permission_evaluation_audit(
    conn: &Connection,
    actor_type: &str,
    device_id: &str,
    access_kind: ExternalClientAccessKind,
    client_id: &str,
    tool_name: &str,
    entity_scope: Option<ExternalClientEntityScope<'_>>,
    result: Result<&ToolPermissionEvaluation, &CrmError>,
) -> CrmResult<()> {
    let context = ExternalClientPermissionEvaluationAudit::from_result(
        client_id,
        tool_name,
        access_kind,
        entity_scope,
        result,
    );
    record_audit_json(
        conn,
        actor_type,
        access_kind.audit_action(),
        Some(EXTERNAL_CLIENT_ENTITY_TYPE),
        Some(client_id),
        None::<&()>,
        Some(&context),
        device_id,
    )?;
    Ok(())
}

fn require_existing_external_client(
    conn: &Connection,
    client_id: &str,
) -> CrmResult<ExternalClient> {
    storage::external_clients::get_active_external_client(conn, client_id)?.ok_or_else(|| {
        CrmError::NotFound(format!(
            "External client '{}' was not found or has been deleted",
            client_id
        ))
    })
}

#[derive(Debug, Clone, Copy, Serialize)]
pub(super) struct ExternalClientEntityScope<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_id: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
enum ExternalClientAccessKind {
    Read,
    Draft,
}

impl ExternalClientAccessKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Draft => "draft",
        }
    }

    fn audit_action(self) -> &'static str {
        match self {
            Self::Read => EVALUATE_EXTERNAL_CLIENT_READ_PERMISSION_ACTION,
            Self::Draft => EVALUATE_EXTERNAL_CLIENT_DRAFT_PERMISSION_ACTION,
        }
    }
}

#[derive(Debug, Serialize)]
struct ExternalClientPermissionEvaluationAudit<'a> {
    client_id: &'a str,
    tool_name: &'a str,
    access_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<&'static str>,
    allowed: bool,
    reason: &'static str,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_scope: Option<ExternalClientEntityScope<'a>>,
}

impl<'a> ExternalClientPermissionEvaluationAudit<'a> {
    fn from_result(
        client_id: &'a str,
        tool_name: &'a str,
        access_kind: ExternalClientAccessKind,
        entity_scope: Option<ExternalClientEntityScope<'a>>,
        result: Result<&ToolPermissionEvaluation, &CrmError>,
    ) -> Self {
        match result {
            Ok(evaluation) => Self {
                client_id,
                tool_name,
                access_kind: access_kind.as_str(),
                mode: Some(evaluation.mode.as_str()),
                allowed: evaluation.allowed,
                reason: evaluation.reason.as_str(),
                status: if evaluation.allowed {
                    "allowed"
                } else {
                    "denied"
                },
                entity_scope,
            },
            Err(CrmError::NotFound(_)) => Self {
                client_id,
                tool_name,
                access_kind: access_kind.as_str(),
                mode: None,
                allowed: false,
                reason: "client_not_found_or_deleted",
                status: "error",
                entity_scope,
            },
            Err(_) => Self {
                client_id,
                tool_name,
                access_kind: access_kind.as_str(),
                mode: None,
                allowed: false,
                reason: "evaluation_error",
                status: "error",
                entity_scope,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ExternalClientResultAccessKind {
    Read,
    Draft,
}

impl ExternalClientResultAccessKind {
    fn parse(value: &str) -> CrmResult<Self> {
        match value.trim() {
            "read" => Ok(Self::Read),
            "draft" => Ok(Self::Draft),
            other => Err(CrmError::InvalidInput(format!(
                "External client tool result access kind '{}' is unsupported",
                other
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Draft => "draft",
        }
    }
}

#[derive(Debug, Serialize)]
struct ExternalClientToolResultAudit<'a> {
    client_id: &'a str,
    tool_name: &'a str,
    access_kind: &'static str,
    status: &'static str,
    result_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_scope: Option<ExternalClientEntityScope<'a>>,
}

fn require_initial_external_client_mode(
    conn: &Connection,
    client_id: &str,
) -> CrmResult<ExternalClientPermissionMode> {
    let client = require_existing_external_client(conn, client_id)?;
    let mode = parse_external_client_mode(&client)?;
    if !mode.is_supported_initial_mode() {
        return Err(CrmError::InvalidInput(format!(
            "External client '{}' has future permission mode '{}' that is not enabled in this sprint",
            client_id,
            mode.as_str()
        )));
    }

    Ok(mode)
}

fn external_client_mode_for_evaluation(
    conn: &Connection,
    client_id: &str,
) -> CrmResult<ExternalClientPermissionMode> {
    let client = require_existing_external_client(conn, client_id)?;
    if !client.enabled {
        return Ok(ExternalClientPermissionMode::Disabled);
    }

    parse_external_client_mode(&client)
}

fn parse_external_client_mode(client: &ExternalClient) -> CrmResult<ExternalClientPermissionMode> {
    ExternalClientPermissionMode::from_storage_value(&client.permission_mode).ok_or_else(|| {
        CrmError::InvalidInput(format!(
            "External client '{}' has unsupported permission mode '{}'",
            client.id, client.permission_mode
        ))
    })
}

fn validate_initial_permission_write(
    can_write: bool,
    requires_confirmation: bool,
) -> CrmResult<()> {
    if can_write && !requires_confirmation {
        return Err(CrmError::InvalidInput(
            "External client write permissions must require confirmation in this sprint"
                .to_string(),
        ));
    }

    Ok(())
}
