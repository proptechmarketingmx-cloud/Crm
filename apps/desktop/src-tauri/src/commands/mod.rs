//! Desktop IPC commands.
//!
//! Command arguments keep Rust `snake_case` names on the JavaScript side
//! (`rename_all = "snake_case"`). Tauri's default camelCase conversion does not
//! match the existing frontend invoke payloads.

use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering;
use std::sync::MutexGuard;

use crm_core::CrmCore;
use tauri::State;

use crate::AppState;

pub mod activity_commands;
pub mod audit_pending_commands;
pub mod backup_commands;
pub mod contact_commands;
pub mod custom_field_commands;
pub mod dashboard_commands;
pub mod deal_commands;
pub mod email_commands;
pub mod external_client_commands;
pub mod import_export;
pub mod link_commands;
pub mod note_commands;
pub mod organization_commands;
pub(crate) mod path_guard;
pub mod report_commands;
pub mod saved_view_commands;
pub mod search_commands;
pub mod settings_commands;
pub mod sync_commands;
pub mod tag_commands;

pub(crate) struct CoreGuard<'a> {
    guard: MutexGuard<'a, Option<CrmCore>>,
}

impl Deref for CoreGuard<'_> {
    type Target = CrmCore;

    fn deref(&self) -> &Self::Target {
        self.guard
            .as_ref()
            .expect("CrmCore slot should be populated after lock_core succeeds")
    }
}

impl DerefMut for CoreGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard
            .as_mut()
            .expect("CrmCore slot should be populated after lock_core succeeds")
    }
}

pub(crate) fn lock_core<'a>(state: &'a State<'_, AppState>) -> Result<CoreGuard<'a>, String> {
    let mut guard = state
        .core
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    if guard.is_none() {
        if state.needs_reopen.swap(false, Ordering::SeqCst) {
            match CrmCore::open(&state.data_dir) {
                Ok(core) => *guard = Some(core),
                Err(e) => {
                    state.needs_reopen.store(true, Ordering::SeqCst);
                    return Err(format!(
                        "CrmCore failed to reopen after restore: {}. Please restart the app.",
                        e
                    ));
                }
            }
        } else {
            return Err("CrmCore is unavailable during local restore".to_string());
        }
    }
    Ok(CoreGuard { guard })
}
