//! Tauri IPC commands for reporting and analytics metrics.

use tauri::State;

use crm_core::storage::reporting::{ActivityFunnelReport, PipelineConversionReport};

use crate::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_pipeline_conversion_report(
    state: State<'_, AppState>,
) -> Result<PipelineConversionReport, String> {
    let core = super::lock_core(&state)?;
    core.get_pipeline_conversion_report()
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_activity_funnel_report(
    state: State<'_, AppState>,
) -> Result<ActivityFunnelReport, String> {
    let core = super::lock_core(&state)?;
    core.get_activity_funnel_report().map_err(|e| e.to_string())
}
