use std::collections::HashMap;

use crate::audit::ACTOR_DESKTOP_APP;
use crate::result::CrmResult;
use crate::storage::{self, settings::Setting};

use super::{record_audit_json, CrmCore};

impl CrmCore {
    pub fn get_settings(&self) -> CrmResult<HashMap<String, String>> {
        let all = storage::settings::get_all_settings(&self.db.conn)?;
        Ok(all.into_iter().map(|s| (s.key, s.value)).collect())
    }

    pub fn get_setting(&self, key: &str) -> CrmResult<Option<Setting>> {
        storage::settings::get_setting(&self.db.conn, key)
    }

    const ALLOWED_SETTING_KEYS: &[&str] = &[
        "language",
        "currency",
        "theme",
        "date_format",
        "sync_enabled",
        "sync_url",
        "notifications_enabled",
        "reminder_lead_minutes",
        "email_integration_enabled",
        "smtp_host",
        "smtp_port",
        "smtp_username",
        "smtp_password",
        "smtp_from",
        "imap_host",
        "imap_port",
        "imap_username",
        "imap_password",
    ];

    pub fn update_setting(&mut self, key: String, value: String) -> CrmResult<Setting> {
        if !Self::ALLOWED_SETTING_KEYS.contains(&key.as_str()) {
            return Err(crate::utils::errors::CrmError::InvalidInput(format!(
                "Unknown setting key '{key}'"
            )));
        }
        let before = storage::settings::get_setting(&self.db.conn, &key)?;
        let device_id = self.device_id.clone();
        let tx = self.db.conn.unchecked_transaction()?;
        let setting = storage::settings::set_setting(&tx, &key, &value)?;
        storage::sync::record_change(&tx, "setting", &key, &key, None, Some(&value), &device_id)?;
        record_audit_json(
            &tx,
            ACTOR_DESKTOP_APP,
            "update",
            Some("setting"),
            Some(&key),
            before.as_ref(),
            Some(&setting),
            &device_id,
        )?;
        tx.commit()?;
        Ok(setting)
    }
}
