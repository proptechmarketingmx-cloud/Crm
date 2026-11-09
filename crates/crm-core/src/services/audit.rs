use std::fs;
use std::io::BufWriter;

use crate::result::CrmResult;
use crate::storage::{self, audit::AuditLogEntry};
use crate::utils::csv::{write_audit_log_csv, AuditLogCsvRow};

use super::CrmCore;

impl CrmCore {
    pub fn list_recent_audit_log(&self, limit: u32) -> CrmResult<Vec<AuditLogEntry>> {
        storage::audit::list_recent_audit_log(&self.db.conn, limit)
    }

    pub fn export_audit_log_csv(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_audit_log_rows()?;
        let count = rows.len() as u32;
        let file = fs::File::create(file_path)?;
        write_audit_log_csv(BufWriter::new(file), &rows)?;
        Ok(count)
    }

    pub fn export_audit_log_json(&self, file_path: &str) -> CrmResult<u32> {
        let rows = self.export_audit_log_rows()?;
        let count = rows.len() as u32;
        super::write_json_export(file_path, &rows)?;
        Ok(count)
    }

    fn export_audit_log_rows(&self) -> CrmResult<Vec<AuditLogCsvRow>> {
        Ok(storage::audit::list_all_audit_log(&self.db.conn)?
            .into_iter()
            .map(|entry| AuditLogCsvRow {
                id: entry.id,
                actor_type: entry.actor_type,
                actor_id: entry.actor_id,
                action: entry.action,
                entity_type: entry.entity_type,
                entity_id: entry.entity_id,
                before_json: entry.before_json,
                after_json: entry.after_json,
                created_at: entry.created_at,
                device_id: entry.device_id,
            })
            .collect())
    }
}
