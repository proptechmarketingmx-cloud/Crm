use crm_core::services::DashboardStats;
use tauri::State;

use crate::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    let core = super::lock_core(&state)?;
    core.get_dashboard_stats().map_err(|e| e.to_string())
}
