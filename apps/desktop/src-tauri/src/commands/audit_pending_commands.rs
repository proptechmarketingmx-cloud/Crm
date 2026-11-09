use crm_core::storage::{audit::AuditLogEntry, proposed_actions::ProposedAction};
use tauri::State;

use crate::{commands::lock_core, AppState};

const DEFAULT_AUDIT_LOG_LIMIT: u32 = 100;
const MAX_AUDIT_LOG_LIMIT: u32 = 500;

#[tauri::command(rename_all = "snake_case")]
pub async fn list_recent_audit_log(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<AuditLogEntry>, String> {
    let core = lock_core(&state)?;
    core.list_recent_audit_log(audit_log_limit(limit))
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn list_pending_proposed_actions(
    state: State<'_, AppState>,
) -> Result<Vec<ProposedAction>, String> {
    let core = lock_core(&state)?;
    core.list_pending_proposed_actions()
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn approve_proposed_action(
    state: State<'_, AppState>,
    id: String,
) -> Result<ProposedAction, String> {
    let mut core = lock_core(&state)?;
    core.approve_proposed_action(id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn reject_proposed_action(
    state: State<'_, AppState>,
    id: String,
) -> Result<ProposedAction, String> {
    let mut core = lock_core(&state)?;
    core.reject_proposed_action(id).map_err(|e| e.to_string())
}

fn audit_log_limit(limit: Option<u32>) -> u32 {
    limit
        .unwrap_or(DEFAULT_AUDIT_LOG_LIMIT)
        .clamp(1, MAX_AUDIT_LOG_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::{audit_log_limit, DEFAULT_AUDIT_LOG_LIMIT, MAX_AUDIT_LOG_LIMIT};

    #[test]
    fn audit_log_limit_defaults_and_clamps_to_storage_bounds() {
        assert_eq!(audit_log_limit(None), DEFAULT_AUDIT_LOG_LIMIT);
        assert_eq!(audit_log_limit(Some(0)), 1);
        assert_eq!(audit_log_limit(Some(25)), 25);
        assert_eq!(audit_log_limit(Some(750)), MAX_AUDIT_LOG_LIMIT);
    }
}
