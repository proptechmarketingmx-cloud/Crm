use std::{fs, io::BufWriter};

use crate::audit::ACTOR_DESKTOP_APP;
use crate::permissions::ExternalClientPermissionMode;
use crate::result::CrmResult;
use crate::storage::{self, external_clients::ExternalClient};
use crate::utils::csv::{write_external_clients_csv, ExternalClientCsvRow};
use crate::utils::errors::CrmError;

use super::CrmCore;
use super::{external_client_permissions::required_external_client_field, record_audit_json};

impl CrmCore {
    pub fn list_external_clients(&self) -> CrmResult<Vec<ExternalClient>> {
        storage::external_clients::list_external_clients(&self.db.conn)
    }

    pub fn create_external_client_placeholder(
        &mut self,
        name: &str,
        client_type: &str,
    ) -> CrmResult<ExternalClient> {
        let name = required_external_client_field("name", name)?;
        let client_type = required_external_client_field("client_type", client_type)?;

        storage::external_clients::create_external_client_placeholder(
            &self.db.conn,
            &name,
            &client_type,
            &self.device_id,
        )
    }

    pub fn update_external_client_activation(
        &mut self,
        client_id: &str,
        enabled: bool,
        permission_mode: &str,
    ) -> CrmResult<ExternalClient> {
        let client_id = required_external_client_field("client_id", client_id)?;
        let permission_mode = required_external_client_field("permission_mode", permission_mode)?;
        let mode = validate_activation_mode(enabled, &permission_mode)?;
        let before =
            storage::external_clients::get_active_external_client(&self.db.conn, &client_id)?
                .ok_or_else(|| external_client_not_found(&client_id))?;

        if before.enabled == enabled && before.permission_mode == mode.as_str() {
            return Ok(before);
        }

        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let client = storage::external_clients::update_external_client_activation(
            &tx,
            &client_id,
            enabled,
            mode.as_str(),
        )?
        .ok_or_else(|| external_client_not_found(&client_id))?;

        storage::sync::record_change(
            &tx,
            "external_client",
            &client.id,
            "__update__",
            Some(&before.id),
            Some(&client.id),
            &device_id,
        )?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "update_external_client_activation",
            Some("external_client"),
            Some(&client.id),
            Some(&before),
            Some(&client),
            &device_id,
        )?;

        tx.commit()?;
        Ok(client)
    }

    pub fn export_external_clients_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_external_client_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_external_clients_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_external_clients_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_external_client_rows()?;
        let count = rows.len() as u32;
        super::write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_external_client_rows(&self) -> CrmResult<Vec<ExternalClientCsvRow>> {
        Ok(
            storage::external_clients::list_external_clients_for_export(&self.db.conn)?
                .into_iter()
                .map(external_client_export_row)
                .collect(),
        )
    }
}

fn validate_activation_mode(
    enabled: bool,
    permission_mode: &str,
) -> CrmResult<ExternalClientPermissionMode> {
    let mode = ExternalClientPermissionMode::from_storage_value(permission_mode).ok_or_else(|| {
        CrmError::InvalidInput(format!(
            "External client permission mode '{}' is not supported by the activation review surface",
            permission_mode
        ))
    })?;

    if !mode.is_supported_initial_mode() {
        return Err(CrmError::InvalidInput(format!(
            "External client permission mode '{}' is reserved for future support and cannot be activated in this sprint",
            permission_mode
        )));
    }

    match (enabled, mode) {
        (false, ExternalClientPermissionMode::Disabled) => Ok(mode),
        (
            true,
            ExternalClientPermissionMode::ReadOnly | ExternalClientPermissionMode::DraftOnly,
        ) => Ok(mode),
        (false, _) => Err(CrmError::InvalidInput(
            "Disabled external clients must use permission mode 'disabled'".to_string(),
        )),
        (true, ExternalClientPermissionMode::Disabled) => Err(CrmError::InvalidInput(
            "Enabled external clients must use permission mode 'read_only' or 'draft_only'"
                .to_string(),
        )),
        (true, _) => Err(CrmError::InvalidInput(
            "External client permission mode is not supported by the activation review surface"
                .to_string(),
        )),
    }
}

fn external_client_not_found(client_id: &str) -> CrmError {
    CrmError::NotFound(format!(
        "External client '{}' was not found or has been deleted",
        client_id
    ))
}

fn external_client_export_row(client: ExternalClient) -> ExternalClientCsvRow {
    ExternalClientCsvRow {
        id: client.id,
        name: client.name,
        client_type: client.client_type,
        permission_mode: client.permission_mode,
        enabled: client.enabled,
        created_at: client.created_at,
        updated_at: client.updated_at,
        deleted_at: client.deleted_at,
        device_id: client.device_id,
    }
}
