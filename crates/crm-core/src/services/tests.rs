use std::collections::HashMap;

use rusqlite::params;

use super::{CrmCore, ImportOptions, TagColorUpdate};
use crate::{
    permissions::ToolPermissionDecisionReason,
    storage::{external_clients::ExternalClient, proposed_actions::ProposedAction},
    utils::{csv::ImportColumnMapping, datetime::now_iso8601, errors::CrmError, uuid::new_uuid},
};

fn open_test_core() -> (CrmCore, std::path::PathBuf) {
    let path = std::env::temp_dir().join(format!("900crm-core-test-{}", new_uuid()));
    let core = CrmCore::open(&path).expect("test core should open");
    (core, path)
}

fn count(core: &CrmCore, sql: &str) -> i64 {
    core.db
        .conn
        .query_row(sql, [], |row| row.get(0))
        .expect("count query should succeed")
}

fn count_custom_field_audit_action(core: &CrmCore, value_id: &str, action: &str) -> i64 {
    core.db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'custom_field_value' AND entity_id = ?1 AND action = ?2",
            params![value_id, action],
            |row| row.get(0),
        )
        .expect("custom field audit count query should succeed")
}

fn count_custom_field_set_sync(core: &CrmCore, value_id: &str, new_value: &str) -> i64 {
    core.db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'custom_field_value' AND entity_id = ?1 AND field_name = 'value' AND new_value = ?2",
            params![value_id, new_value],
            |row| row.get(0),
        )
        .expect("custom field set sync count query should succeed")
}

fn count_custom_field_delete_sync(core: &CrmCore, value_id: &str, old_value: &str) -> i64 {
    core.db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'custom_field_value' AND entity_id = ?1 AND field_name = '__delete__' AND old_value = ?2 AND new_value IS NULL",
            params![value_id, old_value],
            |row| row.get(0),
        )
        .expect("custom field delete sync count query should succeed")
}

fn import_mapping(pairs: &[(&str, Option<&str>)]) -> ImportColumnMapping {
    pairs
        .iter()
        .map(|(source, target)| ((*source).to_string(), target.map(str::to_string)))
        .collect()
}

fn read_json_export(path: &std::path::Path) -> Vec<serde_json::Value> {
    let json = std::fs::read_to_string(path).expect("JSON export should read");
    assert!(
        json.contains("\n  {"),
        "JSON export should be pretty-printed"
    );
    serde_json::from_str::<Vec<serde_json::Value>>(&json).expect("JSON export should parse")
}

fn read_csv_export(path: &std::path::Path) -> Vec<HashMap<String, String>> {
    let mut reader = csv::Reader::from_path(path).expect("CSV export should read");
    let headers = reader
        .headers()
        .expect("CSV export should include headers")
        .iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

    reader
        .records()
        .map(|record| {
            let record = record.expect("CSV export row should parse");
            headers
                .iter()
                .cloned()
                .zip(record.iter().map(str::to_string))
                .collect()
        })
        .collect()
}

fn create_test_proposed_action(core: &mut CrmCore, title: &str) -> ProposedAction {
    create_test_proposed_action_with_input(core, format!(r#"{{"title":"{}"}}"#, title))
}

fn create_test_proposed_action_with_input(
    core: &mut CrmCore,
    input_json: String,
) -> ProposedAction {
    create_test_proposed_action_with_identity(
        core,
        "create_activity",
        "create_activity_draft",
        input_json,
    )
}

fn create_test_proposed_action_with_identity(
    core: &mut CrmCore,
    action_type: &str,
    tool_name: &str,
    input_json: String,
) -> ProposedAction {
    core.create_external_proposed_action_stub(
        None,
        action_type.to_string(),
        tool_name.to_string(),
        Some("activity".to_string()),
        None,
        input_json,
        None,
    )
    .expect("proposed action should be created")
}

fn create_test_external_client_with_mode(
    core: &mut CrmCore,
    permission_mode: &str,
) -> ExternalClient {
    let client = core
        .create_external_client_placeholder("Test MCP Client", "mcp")
        .expect("external client placeholder should be created");
    core.db
        .conn
        .execute(
            "UPDATE external_clients SET permission_mode = ?1, enabled = 1, updated_at = ?2 WHERE id = ?3",
            params![permission_mode, now_iso8601(), &client.id],
        )
        .expect("external client permission mode should update");

    ExternalClient {
        permission_mode: permission_mode.to_string(),
        enabled: true,
        ..client
    }
}

fn create_client_proposed_action(
    core: &mut CrmCore,
    client_id: &str,
    tool_name: &str,
) -> ProposedAction {
    core.create_external_proposed_action_stub(
        Some(client_id.to_string()),
        "create_activity".to_string(),
        tool_name.to_string(),
        Some("activity".to_string()),
        None,
        r#"{"title":"External draft"}"#.to_string(),
        None,
    )
    .expect("client proposed action should be created")
}

fn count_permission_rows(core: &CrmCore, client_id: &str, tool_name: &str) -> i64 {
    core.db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM external_client_permissions WHERE client_id = ?1 AND tool_name = ?2",
            params![client_id, tool_name],
            |row| row.get(0),
        )
        .expect("permission row count should query")
}

fn count_permission_audit_rows(core: &CrmCore, permission_id: &str) -> i64 {
    core.db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'upsert_external_client_permission' AND entity_type = 'external_client_permission' AND entity_id = ?1",
            params![permission_id],
            |row| row.get(0),
        )
        .expect("permission audit row count should query")
}

fn count_permission_sync_rows(core: &CrmCore, permission_id: &str) -> i64 {
    core.db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'external_client_permission' AND entity_id = ?1",
            params![permission_id],
            |row| row.get(0),
        )
        .expect("permission sync row count should query")
}

fn count_external_client_activation_audit_rows(core: &CrmCore, client_id: &str) -> i64 {
    core.db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'update_external_client_activation' AND entity_type = 'external_client' AND entity_id = ?1",
            params![client_id],
            |row| row.get(0),
        )
        .expect("external client activation audit row count should query")
}

fn count_external_client_sync_rows(core: &CrmCore, client_id: &str) -> i64 {
    core.db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'external_client' AND entity_id = ?1 AND field_name = '__update__'",
            params![client_id],
            |row| row.get(0),
        )
        .expect("external client sync row count should query")
}

fn count_external_client_evaluation_audit_rows(
    core: &CrmCore,
    action: &str,
    actor_type: &str,
    client_id: &str,
) -> i64 {
    core.db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = ?1 AND actor_type = ?2 AND entity_type = 'external_client' AND entity_id = ?3",
            params![action, actor_type, client_id],
            |row| row.get(0),
        )
        .expect("external client evaluation audit row count should query")
}

fn external_client_evaluation_audit_context(
    core: &CrmCore,
    action: &str,
    actor_type: &str,
    client_id: &str,
) -> serde_json::Value {
    let (entity_type, entity_id, after_json): (Option<String>, Option<String>, Option<String>) =
        core.db
            .conn
            .query_row(
                "SELECT entity_type, entity_id, after_json FROM audit_log WHERE action = ?1 AND actor_type = ?2 AND entity_id = ?3 ORDER BY created_at DESC, id DESC LIMIT 1",
                params![action, actor_type, client_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("external client evaluation audit row should query");

    assert_eq!(entity_type.as_deref(), Some("external_client"));
    assert_eq!(entity_id.as_deref(), Some(client_id));
    serde_json::from_str(
        after_json
            .as_deref()
            .expect("audit after_json should exist"),
    )
    .expect("evaluation audit context should parse")
}

fn external_client_tool_result_audit_context(
    core: &CrmCore,
    client_id: &str,
    tool_name: &str,
) -> serde_json::Value {
    let (entity_type, entity_id, after_json): (Option<String>, Option<String>, Option<String>) =
        core.db
            .conn
            .query_row(
                "SELECT entity_type, entity_id, after_json FROM audit_log WHERE action = 'record_external_client_tool_result' AND actor_type = 'mcp_client' AND entity_id = ?1 AND after_json LIKE ?2 ORDER BY created_at DESC, id DESC LIMIT 1",
                params![client_id, format!("%\"tool_name\":\"{tool_name}\"%")],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("external client tool result audit row should query");

    assert_eq!(entity_type.as_deref(), Some("external_client"));
    assert_eq!(entity_id.as_deref(), Some(client_id));
    serde_json::from_str(
        after_json
            .as_deref()
            .expect("tool result audit after_json should exist"),
    )
    .expect("tool result audit context should parse")
}

fn assert_permission_denial(error: CrmError, reason: &str) {
    match error {
        CrmError::InvalidInput(message) => {
            assert!(message.contains(reason), "{message}");
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}

#[test]
fn create_local_backup_writes_snapshot_and_metadata() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Backup".to_string()),
            Some("Tester".to_string()),
            None,
            Some("backup@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created before backup");

    let backup_dir = path.join("backup");
    let backup = core
        .create_local_backup(&backup_dir)
        .expect("backup should be created");

    assert_eq!(backup.metadata.backup_format_version, 1);
    assert_eq!(backup.metadata.app_version, env!("CARGO_PKG_VERSION"));
    assert_eq!(
        backup.metadata.schema_version,
        crate::storage::Database::current_schema_version()
    );
    assert_eq!(backup.metadata.device_id, core.device_id());
    assert_eq!(backup.metadata.database_file, "900crm.db");
    assert!(std::path::Path::new(&backup.database_path).is_file());
    assert!(std::path::Path::new(&backup.metadata_path).is_file());

    let backup_conn =
        rusqlite::Connection::open(&backup.database_path).expect("backup database should open");
    let backed_up_contact_count: i64 = backup_conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE id = ?1",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("backup contact count should query");
    assert_eq!(backed_up_contact_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn backup_metadata_validation_accepts_current_schema() {
    let (core, path) = open_test_core();
    let backup_dir = path.join("backup");
    let backup = core
        .create_local_backup(&backup_dir)
        .expect("backup should be created");

    let validation = core
        .validate_local_backup(&backup_dir)
        .expect("backup should validate");

    assert_eq!(validation.metadata, backup.metadata);
    assert_eq!(validation.database_path, backup.database_path);
    assert_eq!(validation.metadata_path, backup.metadata_path);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn restore_requires_explicit_confirmation() {
    let (core, path) = open_test_core();
    let backup_dir = path.join("backup");
    core.create_local_backup(&backup_dir)
        .expect("backup should be created");

    let restore_dir = path.join("restore-target");
    let err = CrmCore::restore_local_backup_to_app_data(&restore_dir, &backup_dir, false)
        .expect_err("restore should require explicit confirmation");

    match err {
        crate::utils::errors::CrmError::InvalidInput(message) => {
            assert!(message.contains("requires explicit confirmation"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }
    assert!(!restore_dir.join("900crm.db").exists());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn invalid_restore_is_rejected_before_applying() {
    let (source_core, source_path) = open_test_core();
    let backup_dir = source_path.join("backup");
    let backup = source_core
        .create_local_backup(&backup_dir)
        .expect("backup should be created");

    let mut metadata: super::backup::LocalBackupMetadata = serde_json::from_slice(
        &std::fs::read(&backup.metadata_path).expect("metadata should read"),
    )
    .expect("metadata should parse");
    metadata.schema_version = crate::storage::Database::current_schema_version() + 1;
    std::fs::write(
        &backup.metadata_path,
        serde_json::to_vec_pretty(&metadata).expect("metadata should serialize"),
    )
    .expect("metadata should be overwritten");

    let (mut target_core, target_path) = open_test_core();
    let existing = target_core
        .create_contact(
            Some("person".to_string()),
            Some("Existing".to_string()),
            Some("Target".to_string()),
            None,
            Some("existing@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("target contact should be created");
    drop(target_core);

    let err = CrmCore::restore_local_backup_to_app_data(&target_path, &backup_dir, true)
        .expect_err("future-schema backup should be rejected");
    match err {
        crate::utils::errors::CrmError::InvalidInput(message) => {
            assert!(message.contains("newer than supported schema version"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let target_core = CrmCore::open(&target_path).expect("target core should still open");
    let preserved_count: i64 = target_core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE id = ?1",
            params![existing.id],
            |row| row.get(0),
        )
        .expect("preserved target contact count should query");
    assert_eq!(preserved_count, 1);

    drop(source_core);
    drop(target_core);
    let _ = std::fs::remove_dir_all(source_path);
    let _ = std::fs::remove_dir_all(target_path);
}

#[test]
fn corrupt_backup_database_is_rejected_by_integrity_validation() {
    let (core, path) = open_test_core();
    let backup_dir = path.join("backup");
    let backup = core
        .create_local_backup(&backup_dir)
        .expect("backup should be created");

    std::fs::write(&backup.database_path, b"not a sqlite database")
        .expect("backup database should be overwritten with corrupt bytes");

    let err = core
        .validate_local_backup(&backup_dir)
        .expect_err("corrupt backup should be rejected");
    match err {
        crate::utils::errors::CrmError::InvalidInput(message) => {
            assert!(
                message.contains("invalid") || message.contains("integrity check failed"),
                "unexpected message: {message}"
            );
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let leftover_validation_files = std::fs::read_dir(&backup_dir)
        .expect("backup dir should be readable")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("integrity-check")
        })
        .count();
    assert_eq!(leftover_validation_files, 0);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn failed_confirmed_restore_removes_temporary_restore_file() {
    let (core, path) = open_test_core();
    let backup_dir = path.join("backup");
    core.create_local_backup(&backup_dir)
        .expect("backup should be created");

    let restore_dir = path.join("restore-target");
    let conflicting_database_dir = restore_dir.join("900crm.db");
    std::fs::create_dir_all(&conflicting_database_dir)
        .expect("conflicting database directory should be created");

    let err = CrmCore::restore_local_backup_to_app_data(&restore_dir, &backup_dir, true)
        .expect_err("restore should fail when target database path is a directory");
    match err {
        crate::utils::errors::CrmError::Io(_) => {}
        other => panic!("expected Io restore failure, got {other:?}"),
    }

    assert!(
        !restore_dir.join("900crm.db.restore_tmp").exists(),
        "failed restore should remove temporary database copy"
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn activity_stats_query_counts_completed_overdue_and_due_today() {
    let (mut core, path) = open_test_core();
    let today = now_iso8601()[..10].to_string();

    core.create_activity(
        "task".to_string(),
        "Due today".to_string(),
        None,
        Some(format!("{today}T23:59:59Z")),
        None,
        None,
    )
    .expect("due-today activity should be created");
    core.create_activity(
        "task".to_string(),
        "Past due".to_string(),
        None,
        Some("2000-01-01T00:00:00Z".to_string()),
        None,
        None,
    )
    .expect("overdue activity should be created");
    let completed = core
        .create_activity(
            "task".to_string(),
            "Completed".to_string(),
            None,
            Some("2000-01-01T00:00:00Z".to_string()),
            None,
            None,
        )
        .expect("completed activity should be created");
    core.mark_activity_complete(&completed.id)
        .expect("activity should be marked complete");

    let stats = crate::crm_engine::activities::get_activity_stats(&core.db.conn)
        .expect("activity stats should query");
    assert_eq!(stats.total, 3);
    assert_eq!(stats.completed, 1);
    assert_eq!(stats.overdue, 1);
    assert_eq!(stats.due_today, 1);
    assert_eq!(stats.pending, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn date_only_activity_stats_follow_the_local_calendar_day() {
    let (mut core, path) = open_test_core();
    let today = chrono::Local::now().date_naive();
    let yesterday = (today - chrono::Duration::days(1)).to_string();
    let today = today.to_string();

    core.create_activity(
        "visit".to_string(),
        "Due today date-only".to_string(),
        None,
        Some(today),
        None,
        None,
    )
    .expect("date-only today activity should be created");
    core.create_activity(
        "sms".to_string(),
        "Yesterday date-only".to_string(),
        None,
        Some(yesterday),
        None,
        None,
    )
    .expect("date-only yesterday activity should be created");

    let stats = crate::crm_engine::activities::get_activity_stats(&core.db.conn)
        .expect("activity stats should query");
    assert_eq!(stats.total, 2);
    assert_eq!(stats.completed, 0);
    assert_eq!(stats.overdue, 1);
    assert_eq!(stats.due_today, 1);
    assert_eq!(stats.pending, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn duplicate_detection_uses_contact_repository_queries() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            Some("amina@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");

    let email_input = crate::crm_engine::contacts::ContactInput {
        contact_type: Some("person".to_string()),
        first_name: Some("Other".to_string()),
        last_name: Some("Person".to_string()),
        org_name: None,
        email: Some("AMINA@example.com".to_string()),
        phone: None,
        address: None,
        city: None,
        country: None,
        org_id: None,
        notes: None,
    };
    let email_matches =
        crate::crm_engine::contacts::find_duplicate_candidates(&core.db.conn, &email_input)
            .expect("email duplicate detection should query");
    assert_eq!(email_matches.len(), 1);
    assert_eq!(email_matches[0].contact.id, contact.id);
    assert_eq!(email_matches[0].score, 1.0);

    let name_input = crate::crm_engine::contacts::ContactInput {
        contact_type: Some("person".to_string()),
        first_name: Some("Amina".to_string()),
        last_name: Some("Diallo".to_string()),
        org_name: None,
        email: None,
        phone: None,
        address: None,
        city: None,
        country: None,
        org_id: None,
        notes: None,
    };
    let name_matches =
        crate::crm_engine::contacts::find_duplicate_candidates(&core.db.conn, &name_input)
            .expect("name duplicate detection should query");
    assert_eq!(name_matches.len(), 1);
    assert_eq!(name_matches[0].contact.id, contact.id);
    assert_eq!(name_matches[0].score, 0.9);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn contact_duplicate_candidates_flag_email_and_phone_pairs_without_writes() {
    let (mut core, path) = open_test_core();

    let email_target = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Target".to_string()),
            None,
            Some("Ada.Exact@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("email target should be created");
    let email_source = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Source".to_string()),
            None,
            Some(" Ada.Exact@example.com ".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("email source should be created");
    let case_only_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Case".to_string()),
            Some("Only".to_string()),
            None,
            Some("ada.exact@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("case-only email contact should be created");
    let phone_target = core
        .create_contact(
            Some("person".to_string()),
            Some("Grace".to_string()),
            Some("Target".to_string()),
            None,
            None,
            Some("+15550100".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("phone target should be created");
    let phone_source = core
        .create_contact(
            Some("person".to_string()),
            Some("Grace".to_string()),
            Some("Source".to_string()),
            None,
            None,
            Some(" +15550100 ".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("phone source should be created");
    let blank_contact_a = core
        .create_contact(
            Some("person".to_string()),
            Some("Blank".to_string()),
            Some("One".to_string()),
            None,
            Some("   ".to_string()),
            Some("   ".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("blank contact should be created");
    let blank_contact_b = core
        .create_contact(
            Some("person".to_string()),
            Some("Blank".to_string()),
            Some("Two".to_string()),
            None,
            Some("   ".to_string()),
            Some("   ".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("blank contact should be created");
    let deleted_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Deleted".to_string()),
            Some("Duplicate".to_string()),
            None,
            Some("Ada.Exact@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("deleted duplicate should be created");
    core.delete_contact(&deleted_contact.id)
        .expect("deleted duplicate should be soft-deleted");

    let contact_count_before = count(&core, "SELECT COUNT(*) FROM contacts");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let candidates = core
        .list_contact_duplicate_candidates()
        .expect("duplicate candidates should load");

    assert_eq!(candidates.len(), 2);

    let email_candidate = candidates
        .iter()
        .find(|candidate| candidate.match_type == "email")
        .expect("email duplicate candidate should exist");
    assert_eq!(email_candidate.source_id, email_source.id);
    assert_eq!(email_candidate.source_display_label, "Ada Source");
    assert_eq!(email_candidate.target_id, email_target.id);
    assert_eq!(email_candidate.target_display_label, "Ada Target");
    assert_eq!(email_candidate.matched_value, "Ada.Exact@example.com");
    assert_eq!(
        email_candidate.reason,
        "Same email address: Ada.Exact@example.com"
    );

    let phone_candidate = candidates
        .iter()
        .find(|candidate| candidate.match_type == "phone")
        .expect("phone duplicate candidate should exist");
    assert_eq!(phone_candidate.source_id, phone_source.id);
    assert_eq!(phone_candidate.source_display_label, "Grace Source");
    assert_eq!(phone_candidate.target_id, phone_target.id);
    assert_eq!(phone_candidate.target_display_label, "Grace Target");
    assert_eq!(phone_candidate.matched_value, "+15550100");
    assert_eq!(phone_candidate.reason, "Same phone number: +15550100");

    for ignored_id in [
        blank_contact_a.id.as_str(),
        blank_contact_b.id.as_str(),
        case_only_contact.id.as_str(),
        deleted_contact.id.as_str(),
    ] {
        assert!(candidates.iter().all(|candidate| {
            candidate.source_id != ignored_id && candidate.target_id != ignored_id
        }));
    }
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM contacts"),
        contact_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn contact_duplicate_candidates_report_pair_once_when_email_and_phone_match() {
    let (mut core, path) = open_test_core();

    let target = core
        .create_contact(
            Some("person".to_string()),
            Some("Pair".to_string()),
            Some("Target".to_string()),
            None,
            Some("Pair@example.com".to_string()),
            Some("+15550200".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("target should be created");
    let source = core
        .create_contact(
            Some("person".to_string()),
            Some("Pair".to_string()),
            Some("Source".to_string()),
            None,
            Some(" Pair@example.com ".to_string()),
            Some(" +15550200 ".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("source should be created");

    let candidates = core
        .list_contact_duplicate_candidates()
        .expect("duplicate candidates should load");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].match_type, "email");
    assert_eq!(candidates[0].matched_value, "Pair@example.com");
    assert_eq!(candidates[0].source_id, source.id);
    assert_eq!(candidates[0].target_id, target.id);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn pipeline_age_query_ignores_closed_deals() {
    let (mut core, path) = open_test_core();

    let active = core
        .create_deal(
            "Current active deal".to_string(),
            Some(100.0),
            None,
            Some("Lead".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("active deal should be created");
    let closed = core
        .create_deal(
            "Old closed deal".to_string(),
            Some(100.0),
            None,
            Some("Closed Won".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("closed deal should be created");

    core.db
        .conn
        .execute(
            "UPDATE deals SET created_at = '2000-01-01T00:00:00Z' WHERE id = ?1",
            params![closed.id],
        )
        .expect("closed deal timestamp should update");

    let age = crate::storage::deals::get_average_active_deal_age_days(&core.db.conn)
        .expect("average active deal age should query");
    assert!(
        age < 1.0,
        "closed deal age should not affect active average, got {age}"
    );

    let active_lookup = core
        .get_deal(&active.id)
        .expect("active deal should remain");
    assert_eq!(active_lookup.stage, "Lead");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

fn seed_global_search_entities(core: &mut CrmCore) {
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Clinic".to_string()),
            Some("Lead".to_string()),
            Some("Clinic Partners".to_string()),
            Some("clinic@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
    core.create_organization(
        "Clinic Partners".to_string(),
        Some("partners@clinic.example".to_string()),
        None,
        None,
        None,
        None,
        Some("Lagos".to_string()),
        None,
        Some("Nigeria".to_string()),
        None,
        Some("Regional clinic group".to_string()),
    )
    .expect("organization should be created");
    core.create_deal(
        "Clinic expansion".to_string(),
        Some(2500.0),
        None,
        Some("Proposal".to_string()),
        None,
        None,
        None,
        None,
        Some("Expansion project".to_string()),
    )
    .expect("deal should be created");
    core.create_activity(
        "call".to_string(),
        "Call clinic".to_string(),
        Some("Discuss rollout".to_string()),
        None,
        None,
        None,
    )
    .expect("activity should be created");
    let tag = core
        .create_tag("Clinic Priority".to_string(), None)
        .expect("tag should be created");
    core.create_note(
        "contact".to_string(),
        contact.id,
        "Clinic onboarding note".to_string(),
    )
    .expect("note should be created");
    assert_eq!(tag.name, "Clinic Priority");
}

#[test]
fn global_search_returns_all_supported_entity_types() {
    let (mut core, path) = open_test_core();
    seed_global_search_entities(&mut core);

    let results = core
        .global_search("Clinic", Some(20))
        .expect("global search should query");
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Contact));
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Organization));
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Deal));
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Activity));
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Note));
    assert!(results
        .iter()
        .any(|r| r.entity_type == crate::search::SearchEntityType::Tag));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn global_search_respects_blank_query_and_limit_behavior() {
    let (mut core, path) = open_test_core();
    seed_global_search_entities(&mut core);

    let blank = core
        .global_search("   ", Some(20))
        .expect("blank global search should succeed");
    assert!(blank.is_empty());

    let zero = core
        .global_search("Clinic", Some(0))
        .expect("zero-limit global search should succeed");
    assert!(zero.is_empty());

    let limited = core
        .global_search("Clinic", Some(2))
        .expect("limited global search should succeed");
    assert_eq!(limited.len(), 2);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v10_creates_global_search_fts_tables() {
    let (core, path) = open_test_core();

    for table_name in [
        "organizations_fts",
        "deals_fts",
        "activities_fts",
        "notes_fts",
        "tags_fts",
    ] {
        let sql = format!(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '{}'",
            table_name
        );
        assert_eq!(count(&core, &sql), 1, "{table_name} should exist");
    }

    assert_eq!(
        core.db
            .schema_version()
            .expect("schema version should read"),
        crate::storage::Database::current_schema_version()
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn global_search_uses_fts_for_non_contact_entities() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "HelioVector Partners".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("orbital clinics".to_string()),
        )
        .expect("organization should create");
    let deal = core
        .create_deal(
            "NeuroHarbor rollout".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("specialized expansion".to_string()),
        )
        .expect("deal should create");
    let activity = core
        .create_activity(
            "call".to_string(),
            "CalderaSignal planning".to_string(),
            Some("coordinate launch".to_string()),
            None,
            None,
            None,
        )
        .expect("activity should create");
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Fiona".to_string()),
            Some("Search".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should create");
    let note = core
        .create_note(
            "contact".to_string(),
            contact.id,
            "QuasarLedger onboarding note".to_string(),
        )
        .expect("note should create");
    let tag = core
        .create_tag("ZenithRoute".to_string(), None)
        .expect("tag should create");

    assert_fts_result(
        &core,
        "HelioVector",
        crate::search::SearchEntityType::Organization,
        &organization.id,
    );
    assert_fts_result(
        &core,
        "NeuroHarbor",
        crate::search::SearchEntityType::Deal,
        &deal.id,
    );
    assert_fts_result(
        &core,
        "CalderaSignal",
        crate::search::SearchEntityType::Activity,
        &activity.id,
    );
    assert_fts_result(
        &core,
        "QuasarLedger",
        crate::search::SearchEntityType::Note,
        &note.id,
    );
    assert_fts_result(
        &core,
        "ZenithRoute",
        crate::search::SearchEntityType::Tag,
        &tag.id,
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn global_search_fts_updates_and_soft_deletes_stay_current() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "OldOrbit Health".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should create");
    let deal = core
        .create_deal(
            "OldOrbit deal".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("deal should create");
    let activity = core
        .create_activity(
            "task".to_string(),
            "OldOrbit task".to_string(),
            None,
            None,
            None,
            None,
        )
        .expect("activity should create");
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Update".to_string()),
            Some("Target".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should create");
    let note = core
        .create_note(
            "contact".to_string(),
            contact.id,
            "OldOrbit note".to_string(),
        )
        .expect("note should create");
    let tag = core
        .create_tag("OldOrbit Tag".to_string(), None)
        .expect("tag should create");

    core.update_organization(
        &organization.id,
        Some("NewOrbit Health".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("organization should update");
    core.update_deal(
        &deal.id,
        Some("NewOrbit deal".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("deal should update");
    core.update_activity(
        &activity.id,
        None,
        Some("NewOrbit task".to_string()),
        None,
        None,
        None,
        None,
        None,
    )
    .expect("activity should update");
    core.update_note(&note.id, "NewOrbit note".to_string())
        .expect("note should update");
    core.update_tag(&tag.id, Some("NewOrbit Tag".to_string()), None)
        .expect("tag should update");

    let old_results = core
        .global_search("OldOrbit", Some(20))
        .expect("old query should search");
    assert!(old_results.is_empty());

    assert_fts_result(
        &core,
        "NewOrbit",
        crate::search::SearchEntityType::Organization,
        &organization.id,
    );
    assert_fts_result(
        &core,
        "NewOrbit",
        crate::search::SearchEntityType::Deal,
        &deal.id,
    );
    assert_fts_result(
        &core,
        "NewOrbit",
        crate::search::SearchEntityType::Activity,
        &activity.id,
    );
    assert_fts_result(
        &core,
        "NewOrbit",
        crate::search::SearchEntityType::Note,
        &note.id,
    );
    assert_fts_result(
        &core,
        "NewOrbit",
        crate::search::SearchEntityType::Tag,
        &tag.id,
    );

    core.delete_organization(&organization.id)
        .expect("organization should delete");
    core.delete_deal(&deal.id).expect("deal should delete");
    core.delete_activity(&activity.id)
        .expect("activity should delete");
    core.delete_note(&note.id).expect("note should delete");
    core.delete_tag(&tag.id).expect("tag should delete");

    let deleted_results = core
        .global_search("NewOrbit", Some(20))
        .expect("deleted query should search");
    assert!(deleted_results.is_empty());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn global_search_uses_text_fallback_when_fts_table_is_missing() {
    let (mut core, path) = open_test_core();

    let tag = core
        .create_tag("FallbackOnly".to_string(), None)
        .expect("tag should create");
    core.db
        .conn
        .execute("DROP TABLE tags_fts", [])
        .expect("tags fts table should drop for fallback test");

    let results = core
        .global_search("FallbackOnly", Some(20))
        .expect("fallback query should search");
    assert!(results.iter().any(|result| {
        result.entity_type == crate::search::SearchEntityType::Tag
            && result.entity_id == tag.id
            && result.match_field == "name"
    }));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

fn assert_fts_result(
    core: &CrmCore,
    query: &str,
    entity_type: crate::search::SearchEntityType,
    entity_id: &str,
) {
    let results = core
        .global_search(query, Some(20))
        .expect("global search should query");
    assert!(
        results.iter().any(|result| {
            result.entity_type == entity_type
                && result.entity_id == entity_id
                && result.match_field == "fts"
        }),
        "expected fts result for {entity_type:?} {entity_id} in {results:?}"
    );
}

#[test]
fn create_contact_writes_contact_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            Some("amina@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            Some("Imported from field office".to_string()),
        )
        .expect("contact should be created");

    let contact_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE id = ?1 AND deleted_at IS NULL",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("contact count should query");
    assert_eq!(contact_count, 1);

    let audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'create' AND entity_type = 'contact' AND entity_id = ?1",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("audit count should query");
    assert_eq!(audit_count, 1);

    let sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'contact' AND entity_id = ?1 AND field_name = '__create__'",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("sync count should query");
    assert_eq!(sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn create_proposed_action_writes_proposed_action_and_audit() {
    let (mut core, path) = open_test_core();

    let proposed_action = core
        .create_external_proposed_action_stub(
            None,
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Follow up"}"#.to_string(),
            None,
        )
        .expect("proposed action should be created");

    let proposed_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM proposed_actions WHERE id = ?1 AND status = 'pending'",
            params![proposed_action.id],
            |row| row.get(0),
        )
        .expect("proposed action count should query");
    assert_eq!(proposed_count, 1);

    let audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'propose_action' AND entity_type = 'proposed_action' AND entity_id = ?1",
            params![proposed_action.id],
            |row| row.get(0),
        )
        .expect("audit count should query");
    assert_eq!(audit_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_permission_default_denies_without_rows() {
    let (mut core, path) = open_test_core();
    let client = create_test_external_client_with_mode(&mut core, "disabled");

    let permissions = core
        .list_external_client_permissions(&client.id)
        .expect("disabled client permissions should list");
    assert!(permissions.is_empty());

    let read = core
        .evaluate_external_client_tool_read_permission(&client.id, "contacts.search")
        .expect("disabled client read evaluation should succeed");
    assert!(!read.allowed);
    assert_eq!(read.reason, ToolPermissionDecisionReason::ClientDisabled);

    let draft = core
        .evaluate_external_client_draft_permission(&client.id, "create_activity_draft")
        .expect("disabled client draft evaluation should succeed");
    assert!(!draft.allowed);
    assert_eq!(draft.reason, ToolPermissionDecisionReason::ClientDisabled);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn read_permission_evaluation_records_audit_context_without_sync_changelog() {
    let (mut core, path) = open_test_core();
    let client = create_test_external_client_with_mode(&mut core, "read_only");

    core.upsert_external_client_tool_permission(&client.id, "contacts.search", true, false, true)
        .expect("read permission should upsert");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let allowed = core
        .evaluate_external_client_tool_read_permission(&client.id, "contacts.search")
        .expect("allowed read evaluation should succeed");
    assert!(allowed.allowed);

    let allowed_context = external_client_evaluation_audit_context(
        &core,
        "evaluate_external_client_read_permission",
        "desktop_app",
        &client.id,
    );
    assert_eq!(
        allowed_context["client_id"].as_str(),
        Some(client.id.as_str())
    );
    assert_eq!(
        allowed_context["tool_name"].as_str(),
        Some("contacts.search")
    );
    assert_eq!(allowed_context["access_kind"].as_str(), Some("read"));
    assert_eq!(allowed_context["mode"].as_str(), Some("read_only"));
    assert_eq!(allowed_context["allowed"].as_bool(), Some(true));
    assert_eq!(allowed_context["reason"].as_str(), Some("allowed"));
    assert_eq!(allowed_context["status"].as_str(), Some("allowed"));
    assert!(allowed_context.get("entity_scope").is_none());

    let denied = core
        .evaluate_external_client_tool_read_permission(&client.id, "contacts.detail")
        .expect("denied read evaluation should still succeed");
    assert!(!denied.allowed);
    assert_eq!(
        denied.reason,
        ToolPermissionDecisionReason::MissingToolPermission
    );

    let denied_context = external_client_evaluation_audit_context(
        &core,
        "evaluate_external_client_read_permission",
        "desktop_app",
        &client.id,
    );
    assert_eq!(
        denied_context["tool_name"].as_str(),
        Some("contacts.detail")
    );
    assert_eq!(denied_context["access_kind"].as_str(), Some("read"));
    assert_eq!(denied_context["mode"].as_str(), Some("read_only"));
    assert_eq!(denied_context["allowed"].as_bool(), Some(false));
    assert_eq!(
        denied_context["reason"].as_str(),
        Some("missing_tool_permission")
    );
    assert_eq!(denied_context["status"].as_str(), Some("denied"));
    assert_eq!(
        count_external_client_evaluation_audit_rows(
            &core,
            "evaluate_external_client_read_permission",
            "desktop_app",
            &client.id
        ),
        2
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn draft_permission_evaluation_records_audit_context_without_sync_changelog() {
    let (mut core, path) = open_test_core();
    let client = create_test_external_client_with_mode(&mut core, "draft_only");

    core.upsert_external_client_tool_permission(
        &client.id,
        "create_activity_draft",
        true,
        true,
        true,
    )
    .expect("draft permission should upsert");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let allowed = core
        .evaluate_external_client_draft_permission(&client.id, "create_activity_draft")
        .expect("allowed draft evaluation should succeed");
    assert!(allowed.allowed);

    let allowed_context = external_client_evaluation_audit_context(
        &core,
        "evaluate_external_client_draft_permission",
        "desktop_app",
        &client.id,
    );
    assert_eq!(
        allowed_context["client_id"].as_str(),
        Some(client.id.as_str())
    );
    assert_eq!(
        allowed_context["tool_name"].as_str(),
        Some("create_activity_draft")
    );
    assert_eq!(allowed_context["access_kind"].as_str(), Some("draft"));
    assert_eq!(allowed_context["mode"].as_str(), Some("draft_only"));
    assert_eq!(allowed_context["allowed"].as_bool(), Some(true));
    assert_eq!(allowed_context["reason"].as_str(), Some("allowed"));
    assert_eq!(allowed_context["status"].as_str(), Some("allowed"));
    assert!(allowed_context.get("entity_scope").is_none());

    let denied = core
        .evaluate_external_client_draft_permission(&client.id, "contacts.search")
        .expect("denied draft evaluation should still succeed");
    assert!(!denied.allowed);
    assert_eq!(
        denied.reason,
        ToolPermissionDecisionReason::MissingToolPermission
    );

    let denied_context = external_client_evaluation_audit_context(
        &core,
        "evaluate_external_client_draft_permission",
        "desktop_app",
        &client.id,
    );
    assert_eq!(
        denied_context["tool_name"].as_str(),
        Some("contacts.search")
    );
    assert_eq!(denied_context["access_kind"].as_str(), Some("draft"));
    assert_eq!(denied_context["mode"].as_str(), Some("draft_only"));
    assert_eq!(denied_context["allowed"].as_bool(), Some(false));
    assert_eq!(
        denied_context["reason"].as_str(),
        Some("missing_tool_permission")
    );
    assert_eq!(denied_context["status"].as_str(), Some("denied"));
    assert_eq!(
        count_external_client_evaluation_audit_rows(
            &core,
            "evaluate_external_client_draft_permission",
            "desktop_app",
            &client.id
        ),
        2
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_tool_result_records_audit_context_without_sync_changelog() {
    let (mut core, path) = open_test_core();
    let client = create_test_external_client_with_mode(&mut core, "read_only");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    core.record_external_client_tool_result(
        &client.id,
        "contacts.list",
        "read",
        2,
        Some("contact"),
        Some("contact-1"),
    )
    .expect("tool result audit should record");

    let context = external_client_tool_result_audit_context(&core, &client.id, "contacts.list");
    assert_eq!(context["client_id"].as_str(), Some(client.id.as_str()));
    assert_eq!(context["tool_name"].as_str(), Some("contacts.list"));
    assert_eq!(context["access_kind"].as_str(), Some("read"));
    assert_eq!(context["status"].as_str(), Some("succeeded"));
    assert_eq!(context["result_count"].as_u64(), Some(2));
    assert_eq!(
        context["entity_scope"]["entity_type"].as_str(),
        Some("contact")
    );
    assert_eq!(
        context["entity_scope"]["entity_id"].as_str(),
        Some("contact-1")
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    let err = core
        .record_external_client_tool_result(&client.id, "contacts.list", "write", 1, None, None)
        .expect_err("unsupported result access kind should reject");
    assert_permission_denial(err, "unsupported");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_evaluation_audits_unknown_and_deleted_client_attempts() {
    let (mut core, path) = open_test_core();
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let unknown = core
        .evaluate_external_client_tool_read_permission("missing-client", "contacts.search")
        .expect_err("unknown client read evaluation should fail");
    assert!(matches!(unknown, CrmError::NotFound(_)));

    let unknown_context = external_client_evaluation_audit_context(
        &core,
        "evaluate_external_client_read_permission",
        "desktop_app",
        "missing-client",
    );
    assert_eq!(
        unknown_context["client_id"].as_str(),
        Some("missing-client")
    );
    assert_eq!(
        unknown_context["tool_name"].as_str(),
        Some("contacts.search")
    );
    assert_eq!(unknown_context["access_kind"].as_str(), Some("read"));
    assert!(unknown_context.get("mode").is_none());
    assert_eq!(unknown_context["allowed"].as_bool(), Some(false));
    assert_eq!(
        unknown_context["reason"].as_str(),
        Some("client_not_found_or_deleted")
    );
    assert_eq!(unknown_context["status"].as_str(), Some("error"));

    let client = create_test_external_client_with_mode(&mut core, "draft_only");
    core.db
        .conn
        .execute(
            "UPDATE external_clients SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now_iso8601(), &client.id],
        )
        .expect("external client should be marked deleted");

    let deleted = core
        .evaluate_external_client_draft_permission(&client.id, "create_activity_draft")
        .expect_err("deleted client draft evaluation should fail");
    assert!(matches!(deleted, CrmError::NotFound(_)));

    let deleted_context = external_client_evaluation_audit_context(
        &core,
        "evaluate_external_client_draft_permission",
        "desktop_app",
        &client.id,
    );
    assert_eq!(
        deleted_context["client_id"].as_str(),
        Some(client.id.as_str())
    );
    assert_eq!(
        deleted_context["tool_name"].as_str(),
        Some("create_activity_draft")
    );
    assert_eq!(deleted_context["access_kind"].as_str(), Some("draft"));
    assert!(deleted_context.get("mode").is_none());
    assert_eq!(deleted_context["allowed"].as_bool(), Some(false));
    assert_eq!(
        deleted_context["reason"].as_str(),
        Some("client_not_found_or_deleted")
    );
    assert_eq!(deleted_context["status"].as_str(), Some("error"));
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn read_only_external_client_can_read_allowed_tool_but_cannot_create_draft() {
    let (mut core, path) = open_test_core();
    let client = create_test_external_client_with_mode(&mut core, "read_only");

    core.upsert_external_client_tool_permission(&client.id, "contacts.search", true, false, true)
        .expect("read-only tool permission should upsert");

    let read = core
        .evaluate_external_client_tool_read_permission(&client.id, "contacts.search")
        .expect("read-only client read evaluation should succeed");
    assert!(read.allowed);

    let draft = core
        .evaluate_external_client_draft_permission(&client.id, "contacts.search")
        .expect("read-only client draft evaluation should succeed");
    assert!(!draft.allowed);
    assert_eq!(draft.reason, ToolPermissionDecisionReason::WriteNotAllowed);

    let err = core
        .create_external_proposed_action_stub(
            Some(client.id.clone()),
            "create_activity".to_string(),
            "contacts.search".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Blocked"}"#.to_string(),
            None,
        )
        .expect_err("read-only client should not create proposed action");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("write_not_allowed"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn draft_only_external_client_requires_matching_confirmed_write_permission() {
    let (mut core, path) = open_test_core();
    let client = create_test_external_client_with_mode(&mut core, "draft_only");

    let missing = core
        .evaluate_external_client_draft_permission(&client.id, "create_activity_draft")
        .expect("draft-only missing permission evaluation should succeed");
    assert!(!missing.allowed);
    assert_eq!(
        missing.reason,
        ToolPermissionDecisionReason::MissingToolPermission
    );

    core.upsert_external_client_tool_permission(
        &client.id,
        "create_activity_draft",
        false,
        true,
        true,
    )
    .expect("draft permission should upsert");

    let allowed = core
        .evaluate_external_client_draft_permission(&client.id, "create_activity_draft")
        .expect("draft-only write permission evaluation should succeed");
    assert!(allowed.allowed);

    let proposed_action =
        create_client_proposed_action(&mut core, &client.id, "create_activity_draft");
    assert_eq!(
        proposed_action.client_id.as_deref(),
        Some(client.id.as_str())
    );
    assert_eq!(proposed_action.status, "pending");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_permission_methods_reject_unknown_and_deleted_clients() {
    let (mut core, path) = open_test_core();

    let unknown_list = core
        .list_external_client_permissions("missing-client")
        .expect_err("unknown client permission list should fail");
    assert!(matches!(unknown_list, CrmError::NotFound(_)));

    let unknown_upsert = core
        .upsert_external_client_tool_permission(
            "missing-client",
            "contacts.search",
            true,
            false,
            true,
        )
        .expect_err("unknown client permission upsert should fail");
    assert!(matches!(unknown_upsert, CrmError::NotFound(_)));

    let unknown_eval = core
        .evaluate_external_client_tool_read_permission("missing-client", "contacts.search")
        .expect_err("unknown client permission evaluation should fail");
    assert!(matches!(unknown_eval, CrmError::NotFound(_)));

    let client = create_test_external_client_with_mode(&mut core, "read_only");
    core.db
        .conn
        .execute(
            "UPDATE external_clients SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now_iso8601(), &client.id],
        )
        .expect("external client should be marked deleted");

    let deleted_list = core
        .list_external_client_permissions(&client.id)
        .expect_err("deleted client permission list should fail");
    assert!(matches!(deleted_list, CrmError::NotFound(_)));

    let deleted_upsert = core
        .upsert_external_client_tool_permission(&client.id, "contacts.search", true, false, true)
        .expect_err("deleted client permission upsert should fail");
    assert!(matches!(deleted_upsert, CrmError::NotFound(_)));

    let deleted_proposal = core
        .create_external_proposed_action_stub(
            Some(client.id.clone()),
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Blocked"}"#.to_string(),
            None,
        )
        .expect_err("deleted client proposed action should fail");
    assert!(matches!(deleted_proposal, CrmError::NotFound(_)));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_permission_upsert_is_idempotent_and_audits_changes() {
    let (mut core, path) = open_test_core();
    let client = create_test_external_client_with_mode(&mut core, "draft_only");

    let inserted = core
        .upsert_external_client_tool_permission(
            &client.id,
            "create_activity_draft",
            true,
            false,
            true,
        )
        .expect("permission should insert");
    assert_eq!(
        count_permission_rows(&core, &client.id, "create_activity_draft"),
        1
    );
    assert_eq!(count_permission_audit_rows(&core, &inserted.id), 1);
    assert_eq!(count_permission_sync_rows(&core, &inserted.id), 1);

    let same = core
        .upsert_external_client_tool_permission(
            &client.id,
            "create_activity_draft",
            true,
            false,
            true,
        )
        .expect("same permission should be idempotent");
    assert_eq!(same.id, inserted.id);
    assert_eq!(same.updated_at, inserted.updated_at);
    assert_eq!(
        count_permission_rows(&core, &client.id, "create_activity_draft"),
        1
    );
    assert_eq!(count_permission_audit_rows(&core, &inserted.id), 1);
    assert_eq!(count_permission_sync_rows(&core, &inserted.id), 1);

    let updated = core
        .upsert_external_client_tool_permission(
            &client.id,
            "create_activity_draft",
            true,
            true,
            true,
        )
        .expect("permission should update");
    assert_eq!(updated.id, inserted.id);
    assert!(updated.can_write);
    assert_eq!(
        count_permission_rows(&core, &client.id, "create_activity_draft"),
        1
    );
    assert_eq!(count_permission_audit_rows(&core, &inserted.id), 2);
    assert_eq!(count_permission_sync_rows(&core, &inserted.id), 2);

    let direct_write_err = core
        .upsert_external_client_tool_permission(
            &client.id,
            "create_activity_draft",
            true,
            true,
            false,
        )
        .expect_err("direct write permission should be rejected");
    match direct_write_err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("must require confirmation"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn create_external_proposed_action_enforces_client_permissions() {
    let (mut core, path) = open_test_core();

    let internal = core
        .create_external_proposed_action_stub(
            None,
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Internal draft"}"#.to_string(),
            None,
        )
        .expect("internal proposed action should still be allowed");
    assert_eq!(internal.client_id, None);

    let disabled_client = create_test_external_client_with_mode(&mut core, "disabled");
    core.upsert_external_client_tool_permission(
        &disabled_client.id,
        "create_activity_draft",
        true,
        true,
        true,
    )
    .expect("disabled client permission row can be configured but should not grant access");
    let disabled_err = core
        .create_external_proposed_action_stub(
            Some(disabled_client.id.clone()),
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Blocked"}"#.to_string(),
            None,
        )
        .expect_err("disabled client proposed action should fail");
    assert_permission_denial(disabled_err, "client_disabled");

    let read_only_client = create_test_external_client_with_mode(&mut core, "read_only");
    core.upsert_external_client_tool_permission(
        &read_only_client.id,
        "create_activity_draft",
        true,
        true,
        true,
    )
    .expect("read-only client permission row can be configured but should not grant draft access");
    let read_only_err = core
        .create_external_proposed_action_stub(
            Some(read_only_client.id.clone()),
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Blocked"}"#.to_string(),
            None,
        )
        .expect_err("read-only client proposed action should fail");
    assert_permission_denial(read_only_err, "write_not_allowed");

    let draft_without_row = create_test_external_client_with_mode(&mut core, "draft_only");
    let missing_err = core
        .create_external_proposed_action_stub(
            Some(draft_without_row.id.clone()),
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Blocked"}"#.to_string(),
            None,
        )
        .expect_err("draft-only client without permission row should fail");
    assert_permission_denial(missing_err, "missing_tool_permission");

    let draft_without_confirmation = create_test_external_client_with_mode(&mut core, "draft_only");
    let now = now_iso8601();
    core.db
        .conn
        .execute(
            r#"
            INSERT INTO external_client_permissions
                (id, client_id, tool_name, can_read, can_write,
                 requires_confirmation, created_at, updated_at)
            VALUES (?1, ?2, ?3, 1, 1, 0, ?4, ?4)
            "#,
            params![
                new_uuid(),
                &draft_without_confirmation.id,
                "create_activity_draft",
                now
            ],
        )
        .expect("unsafe permission row should insert for enforcement regression");
    let no_confirmation_err = core
        .create_external_proposed_action_stub(
            Some(draft_without_confirmation.id.clone()),
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Blocked"}"#.to_string(),
            None,
        )
        .expect_err("draft permission without confirmation should fail");
    assert_permission_denial(no_confirmation_err, "confirmation_not_required");

    let allowed_client = create_test_external_client_with_mode(&mut core, "draft_only");
    core.upsert_external_client_tool_permission(
        &allowed_client.id,
        "create_activity_draft",
        true,
        true,
        true,
    )
    .expect("confirmed draft permission should upsert");
    let allowed =
        create_client_proposed_action(&mut core, &allowed_client.id, "create_activity_draft");
    assert_eq!(
        allowed.client_id.as_deref(),
        Some(allowed_client.id.as_str())
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_proposed_action_stub_audits_denied_draft_permission_without_creating_action() {
    let (mut core, path) = open_test_core();
    let client = create_test_external_client_with_mode(&mut core, "draft_only");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");
    let proposed_count_before = count(&core, "SELECT COUNT(*) FROM proposed_actions");

    let denied = core
        .create_external_proposed_action_stub(
            Some(client.id.clone()),
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            Some("activity-123".to_string()),
            r#"{"title":"Blocked"}"#.to_string(),
            None,
        )
        .expect_err("missing draft permission should fail");
    assert_permission_denial(denied, "missing_tool_permission");

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM proposed_actions"),
        proposed_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    let context = external_client_evaluation_audit_context(
        &core,
        "evaluate_external_client_draft_permission",
        "mcp_client",
        &client.id,
    );
    assert_eq!(context["client_id"].as_str(), Some(client.id.as_str()));
    assert_eq!(context["tool_name"].as_str(), Some("create_activity_draft"));
    assert_eq!(context["access_kind"].as_str(), Some("draft"));
    assert_eq!(context["mode"].as_str(), Some("draft_only"));
    assert_eq!(context["allowed"].as_bool(), Some(false));
    assert_eq!(context["reason"].as_str(), Some("missing_tool_permission"));
    assert_eq!(context["status"].as_str(), Some("denied"));
    assert_eq!(
        context["entity_scope"]["entity_type"].as_str(),
        Some("activity")
    );
    assert_eq!(
        context["entity_scope"]["entity_id"].as_str(),
        Some("activity-123")
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_proposed_action_stub_audits_allowed_draft_permission_and_proposal() {
    let (mut core, path) = open_test_core();
    let client = create_test_external_client_with_mode(&mut core, "draft_only");
    core.upsert_external_client_tool_permission(
        &client.id,
        "create_activity_draft",
        true,
        true,
        true,
    )
    .expect("confirmed draft permission should upsert");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let proposed_action = core
        .create_external_proposed_action_stub(
            Some(client.id.clone()),
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            Some("activity-456".to_string()),
            r#"{"title":"Allowed"}"#.to_string(),
            None,
        )
        .expect("allowed client proposed action should be created");

    let context = external_client_evaluation_audit_context(
        &core,
        "evaluate_external_client_draft_permission",
        "mcp_client",
        &client.id,
    );
    assert_eq!(context["client_id"].as_str(), Some(client.id.as_str()));
    assert_eq!(context["tool_name"].as_str(), Some("create_activity_draft"));
    assert_eq!(context["access_kind"].as_str(), Some("draft"));
    assert_eq!(context["mode"].as_str(), Some("draft_only"));
    assert_eq!(context["allowed"].as_bool(), Some(true));
    assert_eq!(context["reason"].as_str(), Some("allowed"));
    assert_eq!(context["status"].as_str(), Some("allowed"));
    assert_eq!(
        context["entity_scope"]["entity_type"].as_str(),
        Some("activity")
    );
    assert_eq!(
        context["entity_scope"]["entity_id"].as_str(),
        Some("activity-456")
    );

    let propose_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'propose_action' AND actor_type = 'mcp_client' AND entity_type = 'proposed_action' AND entity_id = ?1",
            params![&proposed_action.id],
            |row| row.get(0),
        )
        .expect("proposed-action audit count should query");
    assert_eq!(propose_audit_count, 1);
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn future_external_client_permission_modes_cannot_read_draft_or_upsert_grants() {
    for mode in ["write_with_confirmation", "write_allowed"] {
        let (mut core, path) = open_test_core();
        let client = create_test_external_client_with_mode(&mut core, mode);

        let now = now_iso8601();
        core.db
            .conn
            .execute(
                r#"
                INSERT INTO external_client_permissions
                    (id, client_id, tool_name, can_read, can_write,
                     requires_confirmation, created_at, updated_at)
                VALUES (?1, ?2, 'future.tool', 1, 1, 1, ?3, ?3)
                "#,
                params![new_uuid(), &client.id, now],
            )
            .expect("future-mode permission row should insert for denial regression");

        let read = core
            .evaluate_external_client_tool_read_permission(&client.id, "future.tool")
            .expect("future mode read evaluation should succeed");
        assert!(!read.allowed, "mode {mode} should not allow reads");
        assert_eq!(
            read.reason,
            ToolPermissionDecisionReason::UnsupportedClientMode
        );

        let draft = core
            .evaluate_external_client_draft_permission(&client.id, "future.tool")
            .expect("future mode draft evaluation should succeed");
        assert!(!draft.allowed, "mode {mode} should not allow drafts");
        assert_eq!(
            draft.reason,
            ToolPermissionDecisionReason::UnsupportedClientMode
        );

        let upsert = core
            .upsert_external_client_tool_permission(&client.id, "future.tool", true, true, true)
            .expect_err("future mode client grants should not be configurable yet");
        match upsert {
            CrmError::InvalidInput(message) => {
                assert!(message.contains("future permission mode"), "{message}");
                assert!(message.contains(mode), "{message}");
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }

        drop(core);
        let _ = std::fs::remove_dir_all(path);
    }
}

#[test]
fn list_pending_proposed_actions_returns_only_pending_actions_in_created_order() {
    let (mut core, path) = open_test_core();

    let approved_action = core
        .create_external_proposed_action_stub(
            None,
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Done"}"#.to_string(),
            None,
        )
        .expect("approved proposed action should be created");
    core.db
        .conn
        .execute(
            "UPDATE proposed_actions SET status = 'approved', approved_at = ?1 WHERE id = ?2",
            params![now_iso8601(), &approved_action.id],
        )
        .expect("proposed action status should update");

    let first_pending = core
        .create_external_proposed_action_stub(
            None,
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"First pending"}"#.to_string(),
            None,
        )
        .expect("first pending proposed action should be created");
    let second_pending = core
        .create_external_proposed_action_stub(
            None,
            "create_activity".to_string(),
            "create_activity_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Second pending"}"#.to_string(),
            None,
        )
        .expect("second pending proposed action should be created");
    core.db
        .conn
        .execute(
            "UPDATE proposed_actions SET created_at = '2026-06-24T08:00:00Z' WHERE id = ?1",
            params![&first_pending.id],
        )
        .expect("first pending timestamp should update");
    core.db
        .conn
        .execute(
            "UPDATE proposed_actions SET created_at = '2026-06-24T09:00:00Z' WHERE id = ?1",
            params![&second_pending.id],
        )
        .expect("second pending timestamp should update");

    let pending = core
        .list_pending_proposed_actions()
        .expect("pending proposed actions should list");

    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].id, first_pending.id);
    assert_eq!(pending[1].id, second_pending.id);
    assert!(pending.iter().all(|action| action.status == "pending"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn approve_create_activity_draft_executes_activity_and_marks_proposed_action_executed() {
    let (mut core, path) = open_test_core();
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
    let organization = core
        .create_organization(
            "Nine Hundred Labs".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should be created");
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            None,
            Some(organization.id.clone()),
            None,
        )
        .expect("deal should be created");
    let input_json = serde_json::json!({
        "title": "Approve me",
        "activity_type": "call",
        "description": "Confirm next steps",
        "due_at": "2026-06-25T09:00:00Z",
        "linked_entities": [
            { "entity_type": "contact", "entity_id": contact.id.clone() },
            { "entity_type": "organization", "entity_id": organization.id.clone() },
            { "entity_type": "deal", "entity_id": deal.id.clone() }
        ]
    })
    .to_string();
    let proposed_action = create_test_proposed_action_with_input(&mut core, input_json);

    let approved = core
        .approve_proposed_action(proposed_action.id.clone())
        .expect("pending proposed action should approve and execute");

    assert_eq!(approved.id, proposed_action.id);
    assert_eq!(approved.status, "executed");
    assert!(approved.approved_at.is_some());
    assert_eq!(approved.rejected_at, None);
    assert!(approved.executed_at.is_some());

    let stored_activity: (
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = core
        .db
        .conn
        .query_row(
            "SELECT id, activity_type, title, due_date, contact_id, deal_id FROM activities WHERE title = 'Approve me'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
        )
        .expect("created activity should query");
    assert_eq!(stored_activity.1, "call");
    assert_eq!(stored_activity.2, "Approve me");
    assert_eq!(stored_activity.3.as_deref(), Some("2026-06-25T09:00:00Z"));
    assert_eq!(stored_activity.4.as_deref(), Some(contact.id.as_str()));
    assert_eq!(stored_activity.5.as_deref(), Some(deal.id.as_str()));

    let links = core
        .list_activity_links(&stored_activity.0)
        .expect("created activity links should list");
    assert_eq!(links.len(), 3);
    assert!(links.iter().any(|link| {
        link.entity_type == crate::storage::activities::ActivityLinkEntityType::Contact
            && link.entity_id == contact.id
    }));
    assert!(links.iter().any(|link| {
        link.entity_type == crate::storage::activities::ActivityLinkEntityType::Organization
            && link.entity_id == organization.id
    }));
    assert!(links.iter().any(|link| {
        link.entity_type == crate::storage::activities::ActivityLinkEntityType::Deal
            && link.entity_id == deal.id
    }));

    let audit_payloads: (Option<String>, Option<String>) = core
        .db
        .conn
        .query_row(
            "SELECT before_json, after_json FROM audit_log WHERE actor_type = 'desktop_app' AND action = 'approve_proposed_action' AND entity_type = 'proposed_action' AND entity_id = ?1",
            params![&approved.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("approve audit payload should query");
    assert!(audit_payloads
        .0
        .as_deref()
        .is_some_and(|payload| payload.contains(r#""status":"pending""#)));
    assert!(audit_payloads
        .1
        .as_deref()
        .is_some_and(|payload| payload.contains(r#""status":"executed""#)));
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'execute_proposed_action' AND entity_type = 'proposed_action'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'create' AND entity_type = 'activity'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn approve_create_activity_draft_invalid_input_rolls_back_pending_without_audit_or_activity() {
    let (mut core, path) = open_test_core();
    let proposed_action = create_test_proposed_action_with_input(
        &mut core,
        r#"{"activity_type":"task","description":"missing title"}"#.to_string(),
    );

    let err = core
        .approve_proposed_action(proposed_action.id.clone())
        .expect_err("invalid draft input should not approve");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("title"));
            assert!(message.contains("required"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let stored: (String, Option<String>, Option<String>) = core
        .db
        .conn
        .query_row(
            "SELECT status, approved_at, executed_at FROM proposed_actions WHERE id = ?1",
            params![&proposed_action.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("proposed action should query after failed approval");
    assert_eq!(stored.0, "pending");
    assert_eq!(stored.1, None);
    assert_eq!(stored.2, None);
    assert_eq!(count(&core, "SELECT COUNT(*) FROM activities"), 0);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action IN ('approve_proposed_action', 'execute_proposed_action')"
        ),
        0
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn approve_unsupported_proposed_action_stays_pending_without_audit_or_activity() {
    let (mut core, path) = open_test_core();
    let proposed_action = core
        .create_external_proposed_action_stub(
            None,
            "send_email".to_string(),
            "send_email_draft".to_string(),
            Some("activity".to_string()),
            None,
            r#"{"title":"Unsupported"}"#.to_string(),
            None,
        )
        .expect("unsupported proposed action should be created as pending");

    let err = core
        .approve_proposed_action(proposed_action.id.clone())
        .expect_err("unsupported proposed action should not approve");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("Unsupported proposed action"));
            assert!(message.contains("send_email_draft"));
            assert!(message.contains("send_email"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let stored: (String, Option<String>, Option<String>) = core
        .db
        .conn
        .query_row(
            "SELECT status, approved_at, executed_at FROM proposed_actions WHERE id = ?1",
            params![&proposed_action.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("unsupported proposed action should query after failed approval");
    assert_eq!(stored.0, "pending");
    assert_eq!(stored.1, None);
    assert_eq!(stored.2, None);
    assert_eq!(count(&core, "SELECT COUNT(*) FROM activities"), 0);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action IN ('approve_proposed_action', 'execute_proposed_action')"
        ),
        0
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn approve_mismatched_tool_name_stays_pending_without_audit_or_activity() {
    let (mut core, path) = open_test_core();
    let proposed_action = create_test_proposed_action_with_identity(
        &mut core,
        "create_activity_draft",
        "send_email_draft",
        r#"{"title":"Mismatched tool"}"#.to_string(),
    );

    let err = core
        .approve_proposed_action(proposed_action.id.clone())
        .expect_err("mismatched tool should not approve or execute");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("Unsupported proposed action tool"));
            assert!(message.contains("send_email_draft"));
            assert!(message.contains("create_activity_draft"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let stored: (String, Option<String>, Option<String>) = core
        .db
        .conn
        .query_row(
            "SELECT status, approved_at, executed_at FROM proposed_actions WHERE id = ?1",
            params![&proposed_action.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("mismatched tool proposed action should query after failed approval");
    assert_eq!(stored.0, "pending");
    assert_eq!(stored.1, None);
    assert_eq!(stored.2, None);
    assert_eq!(count(&core, "SELECT COUNT(*) FROM activities"), 0);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action IN ('approve_proposed_action', 'execute_proposed_action')"
        ),
        0
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn approve_mismatched_action_type_stays_pending_without_audit_or_activity() {
    let (mut core, path) = open_test_core();
    let proposed_action = create_test_proposed_action_with_identity(
        &mut core,
        "send_email",
        "create_activity_draft",
        r#"{"title":"Mismatched action"}"#.to_string(),
    );

    let err = core
        .approve_proposed_action(proposed_action.id.clone())
        .expect_err("mismatched action type should not approve or execute");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("Unsupported proposed action action_type"));
            assert!(message.contains("send_email"));
            assert!(message.contains("create_activity_draft"));
            assert!(message.contains("create_activity"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let stored: (String, Option<String>, Option<String>) = core
        .db
        .conn
        .query_row(
            "SELECT status, approved_at, executed_at FROM proposed_actions WHERE id = ?1",
            params![&proposed_action.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("mismatched action proposed action should query after failed approval");
    assert_eq!(stored.0, "pending");
    assert_eq!(stored.1, None);
    assert_eq!(stored.2, None);
    assert_eq!(count(&core, "SELECT COUNT(*) FROM activities"), 0);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action IN ('approve_proposed_action', 'execute_proposed_action')"
        ),
        0
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn reject_pending_proposed_action_marks_timestamp_and_audit_without_execution() {
    let (mut core, path) = open_test_core();
    let proposed_action = create_test_proposed_action(&mut core, "Reject me");

    let rejected = core
        .reject_proposed_action(proposed_action.id.clone())
        .expect("pending proposed action should reject");

    assert_eq!(rejected.id, proposed_action.id);
    assert_eq!(rejected.status, "rejected");
    assert_eq!(rejected.approved_at, None);
    assert!(rejected.rejected_at.is_some());
    assert_eq!(rejected.executed_at, None);
    assert_eq!(count(&core, "SELECT COUNT(*) FROM activities"), 0);

    let stored_executed_at: Option<String> = core
        .db
        .conn
        .query_row(
            "SELECT executed_at FROM proposed_actions WHERE id = ?1",
            params![&rejected.id],
            |row| row.get(0),
        )
        .expect("executed timestamp should query");
    assert_eq!(stored_executed_at, None);

    let audit_payloads: (Option<String>, Option<String>) = core
        .db
        .conn
        .query_row(
            "SELECT before_json, after_json FROM audit_log WHERE actor_type = 'desktop_app' AND action = 'reject_proposed_action' AND entity_type = 'proposed_action' AND entity_id = ?1",
            params![&rejected.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("reject audit payload should query");
    assert!(audit_payloads
        .0
        .as_deref()
        .is_some_and(|payload| payload.contains(r#""status":"pending""#)));
    assert!(audit_payloads
        .1
        .as_deref()
        .is_some_and(|payload| payload.contains(r#""status":"rejected""#)));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn proposed_action_decisions_reject_unknown_and_already_non_pending_actions() {
    let (mut core, path) = open_test_core();

    let unknown_err = core
        .approve_proposed_action("missing-proposed-action".to_string())
        .expect_err("unknown proposed action should be rejected");
    match unknown_err {
        CrmError::NotFound(message) => {
            assert!(message.contains("missing-proposed-action"));
        }
        other => panic!("expected NotFound, got {other:?}"),
    }

    let approved_action = create_test_proposed_action(&mut core, "Already executed");
    core.approve_proposed_action(approved_action.id.clone())
        .expect("first approval should execute");
    let approved_again_err = core
        .approve_proposed_action(approved_action.id.clone())
        .expect_err("already executed proposed action should be rejected");
    match approved_again_err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("must be pending"));
            assert!(message.contains("executed"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let executed_action = create_test_proposed_action(&mut core, "Already executed");
    core.db
        .conn
        .execute(
            "UPDATE proposed_actions SET status = 'executed', executed_at = ?1 WHERE id = ?2",
            params![now_iso8601(), &executed_action.id],
        )
        .expect("executed status should update");
    let executed_err = core
        .reject_proposed_action(executed_action.id.clone())
        .expect_err("already executed proposed action should be rejected");
    match executed_err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("must be pending"));
            assert!(message.contains("executed"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn stale_pending_decision_does_not_succeed_or_write_decision_audit() {
    let (mut core, path) = open_test_core();
    let proposed_action = create_test_proposed_action(&mut core, "Stale pending");

    let audit_count_before: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action IN ('approve_proposed_action', 'reject_proposed_action') AND entity_id = ?1",
            params![&proposed_action.id],
            |row| row.get(0),
        )
        .expect("decision audit count should query before stale decision");

    let err = crate::storage::proposed_actions::approve_proposed_action_after_test_status_change(
        &core.db.conn,
        &proposed_action.id,
        "executed",
    )
    .expect_err("stale pending decision should fail");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("no longer pending"));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let stored: (String, Option<String>, Option<String>, Option<String>) = core
        .db
        .conn
        .query_row(
            "SELECT status, approved_at, rejected_at, executed_at FROM proposed_actions WHERE id = ?1",
            params![&proposed_action.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("stale proposed action should query");
    assert_eq!(stored.0, "executed");
    assert_eq!(stored.1, None);
    assert_eq!(stored.2, None);
    assert_eq!(stored.3, None);

    let audit_count_after: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action IN ('approve_proposed_action', 'reject_proposed_action') AND entity_id = ?1",
            params![&proposed_action.id],
            |row| row.get(0),
        )
        .expect("decision audit count should query after stale decision");
    assert_eq!(audit_count_after, audit_count_before);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn list_recent_audit_log_returns_latest_entries_with_storage_limit_floor() {
    let (mut core, path) = open_test_core();

    let first_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("First".to_string()),
            Some("Contact".to_string()),
            None,
            Some("first.audit@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("first contact should be created");
    let second_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Second".to_string()),
            Some("Contact".to_string()),
            None,
            Some("second.audit@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("second contact should be created");

    core.db
        .conn
        .execute(
            "UPDATE audit_log SET created_at = '2026-06-24T08:00:00Z' WHERE entity_id = ?1",
            params![&first_contact.id],
        )
        .expect("first audit timestamp should update");
    core.db
        .conn
        .execute(
            "UPDATE audit_log SET created_at = '2026-06-24T09:00:00Z' WHERE entity_id = ?1",
            params![&second_contact.id],
        )
        .expect("second audit timestamp should update");

    let latest = core
        .list_recent_audit_log(1)
        .expect("recent audit log should list");
    assert_eq!(latest.len(), 1);
    assert_eq!(
        latest[0].entity_id.as_deref(),
        Some(second_contact.id.as_str())
    );

    let floored = core
        .list_recent_audit_log(0)
        .expect("zero limit should be floored by storage");
    assert_eq!(floored.len(), 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn export_audit_log_csv_and_json_include_all_rows_in_chronological_order_without_auditing_export() {
    let (mut core, path) = open_test_core();

    let first_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("First".to_string()),
            Some("Audit".to_string()),
            None,
            Some("first.audit.export@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("first contact should be created");
    let second_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Second".to_string()),
            Some("Audit".to_string()),
            None,
            Some("second.audit.export@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("second contact should be created");

    core.db
        .conn
        .execute(
            "UPDATE audit_log SET created_at = '2026-06-24T08:00:00Z' WHERE entity_id = ?1",
            params![&first_contact.id],
        )
        .expect("first audit timestamp should update");
    core.db
        .conn
        .execute(
            "UPDATE audit_log SET created_at = '2026-06-24T08:00:00Z' WHERE entity_id = ?1",
            params![&second_contact.id],
        )
        .expect("second audit timestamp should update");

    let first_audit_id: String = core
        .db
        .conn
        .query_row(
            "SELECT id FROM audit_log WHERE entity_id = ?1",
            params![&first_contact.id],
            |row| row.get(0),
        )
        .expect("first audit id should query");
    let second_audit_id: String = core
        .db
        .conn
        .query_row(
            "SELECT id FROM audit_log WHERE entity_id = ?1",
            params![&second_contact.id],
            |row| row.get(0),
        )
        .expect("second audit id should query");
    let mut expected_pairs = [
        (first_audit_id, first_contact.id.clone()),
        (second_audit_id, second_contact.id.clone()),
    ];
    expected_pairs.sort_by(|left, right| left.0.cmp(&right.0));
    let expected_order = expected_pairs
        .into_iter()
        .map(|(_, entity_id)| entity_id)
        .collect::<Vec<_>>();
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");

    let csv_export_path = path.join("audit-log-export.csv");
    let csv_count = core
        .export_audit_log_csv(
            csv_export_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("audit log CSV export should succeed");
    assert_eq!(csv_count, 2);
    let csv_rows = read_csv_export(&csv_export_path);
    assert_eq!(csv_rows.len(), 2);
    let csv_text = std::fs::read_to_string(&csv_export_path).expect("CSV export should read");
    assert!(csv_text.starts_with(
        "id,actor_type,actor_id,action,entity_type,entity_id,before_json,after_json,created_at,device_id\n"
    ));
    assert_eq!(
        csv_rows
            .iter()
            .map(|row| row
                .get("entity_id")
                .expect("entity_id should export")
                .clone())
            .collect::<Vec<_>>(),
        expected_order
    );

    let json_export_path = path.join("audit-log-export.json");
    let json_count = core
        .export_audit_log_json(
            json_export_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("audit log JSON export should succeed");
    assert_eq!(json_count, 2);
    let json_rows = read_json_export(&json_export_path);
    assert_eq!(
        json_rows
            .iter()
            .map(|row| row["entity_id"]
                .as_str()
                .expect("entity_id should export")
                .to_string())
            .collect::<Vec<_>>(),
        expected_order
    );
    assert_eq!(json_rows[0]["actor_type"], "desktop_app");
    assert_eq!(json_rows[0]["action"], "create");
    assert_eq!(json_rows[0]["entity_type"], "contact");
    assert_eq!(
        json_rows[0]["created_at"],
        serde_json::Value::String("2026-06-24T08:00:00Z".to_string())
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn export_proposed_actions_csv_and_json_include_all_rows_in_stable_order_without_mutation() {
    let (mut core, path) = open_test_core();

    let client = create_test_external_client_with_mode(&mut core, "draft_only");
    core.upsert_external_client_tool_permission(
        &client.id,
        "create_activity_draft",
        true,
        true,
        true,
    )
    .expect("client draft permission should be granted");

    let client_action =
        create_client_proposed_action(&mut core, &client.id, "create_activity_draft");
    let local_action = create_test_proposed_action_with_input(
        &mut core,
        r#"{"title":"Local pending"}"#.to_string(),
    );
    let rejected_action = create_test_proposed_action_with_input(
        &mut core,
        r#"{"title":"Rejected draft"}"#.to_string(),
    );
    core.reject_proposed_action(rejected_action.id.clone())
        .expect("proposed action should reject");

    core.db
        .conn
        .execute(
            "UPDATE proposed_actions SET created_at = '2026-06-24T09:00:00Z' WHERE id = ?1",
            params![&client_action.id],
        )
        .expect("client action timestamp should update");
    core.db
        .conn
        .execute(
            "UPDATE proposed_actions SET created_at = '2026-06-24T09:00:00Z' WHERE id = ?1",
            params![&local_action.id],
        )
        .expect("local action timestamp should update");
    core.db
        .conn
        .execute(
            "UPDATE proposed_actions SET created_at = '2026-06-24T08:00:00Z', rejected_at = '2026-06-24T10:00:00Z' WHERE id = ?1",
            params![&rejected_action.id],
        )
        .expect("rejected action timestamps should update");

    let mut tied_ids = [client_action.id.clone(), local_action.id.clone()];
    tied_ids.sort();
    let expected_order = vec![
        rejected_action.id.clone(),
        tied_ids[0].clone(),
        tied_ids[1].clone(),
    ];
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let csv_export_path = path.join("proposed-actions-export.csv");
    let csv_count = core
        .export_proposed_actions_csv(
            csv_export_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("proposed actions CSV export should succeed");
    assert_eq!(csv_count, 3);
    let csv_rows = read_csv_export(&csv_export_path);
    assert_eq!(csv_rows.len(), 3);
    let csv_text = std::fs::read_to_string(&csv_export_path).expect("CSV export should read");
    assert!(csv_text.starts_with(
        "id,external_client_id,client_id,tool_name,action_type,entity_type,entity_id,payload_json,input_json,proposed_output_json,status,created_at,decided_at,approved_at,rejected_at,executed_at,error_message,device_id\n"
    ));
    assert_eq!(
        csv_rows
            .iter()
            .map(|row| row.get("id").expect("id should export").clone())
            .collect::<Vec<_>>(),
        expected_order
    );
    let csv_client_row = csv_rows
        .iter()
        .find(|row| row.get("id") == Some(&client_action.id))
        .expect("client action should export");
    assert_eq!(csv_client_row.get("external_client_id"), Some(&client.id));
    assert_eq!(csv_client_row.get("client_id"), Some(&client.id));
    assert_eq!(
        csv_client_row.get("payload_json"),
        Some(&r#"{"title":"External draft"}"#.to_string())
    );
    assert_eq!(
        csv_client_row.get("payload_json"),
        csv_client_row.get("input_json")
    );
    assert_eq!(csv_client_row.get("error_message"), Some(&String::new()));
    let csv_rejected_row = csv_rows
        .iter()
        .find(|row| row.get("id") == Some(&rejected_action.id))
        .expect("rejected action should export");
    assert_eq!(
        csv_rejected_row.get("status"),
        Some(&"rejected".to_string())
    );
    assert_eq!(
        csv_rejected_row.get("decided_at"),
        Some(&"2026-06-24T10:00:00Z".to_string())
    );
    assert_eq!(
        csv_rejected_row.get("rejected_at"),
        Some(&"2026-06-24T10:00:00Z".to_string())
    );
    assert_eq!(csv_rejected_row.get("executed_at"), Some(&String::new()));

    let json_export_path = path.join("proposed-actions-export.json");
    let json_count = core
        .export_proposed_actions_json(
            json_export_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("proposed actions JSON export should succeed");
    assert_eq!(json_count, 3);
    let json_rows = read_json_export(&json_export_path);
    assert_eq!(
        json_rows
            .iter()
            .map(|row| row["id"].as_str().expect("id should export").to_string())
            .collect::<Vec<_>>(),
        expected_order
    );
    let json_client_row = json_rows
        .iter()
        .find(|row| row["id"] == client_action.id)
        .expect("client action should export");
    assert_eq!(json_client_row["external_client_id"], client.id);
    assert_eq!(json_client_row["client_id"], client.id);
    assert_eq!(
        json_client_row["payload_json"],
        r#"{"title":"External draft"}"#
    );
    assert_eq!(
        json_client_row["payload_json"],
        json_client_row["input_json"]
    );
    assert_eq!(json_client_row["error_message"], serde_json::Value::Null);
    let json_rejected_row = json_rows
        .iter()
        .find(|row| row["id"] == rejected_action.id)
        .expect("rejected action should export");
    assert_eq!(json_rejected_row["status"], "rejected");
    assert_eq!(json_rejected_row["decided_at"], "2026-06-24T10:00:00Z");
    assert_eq!(json_rejected_row["rejected_at"], "2026-06-24T10:00:00Z");
    assert_eq!(json_rejected_row["executed_at"], serde_json::Value::Null);

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn export_external_clients_csv_and_json_include_non_deleted_placeholders_without_mutation() {
    let (mut core, path) = open_test_core();

    core.create_contact(
        Some("person".to_string()),
        Some("External".to_string()),
        Some("Client Export".to_string()),
        None,
        Some("external.client.export@example.com".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("contact should create audit and sync baseline");

    let disabled = core
        .create_external_client_placeholder("Claude Desktop", "mcp")
        .expect("disabled external client placeholder should be created");
    let enabled = core
        .create_external_client_placeholder("Local Script", "script")
        .expect("enabled external client placeholder should be created");
    let deleted = core
        .create_external_client_placeholder("Deleted Integration", "mcp")
        .expect("deleted external client placeholder should be created");

    core.db
        .conn
        .execute(
            "UPDATE external_clients SET created_at = '2026-06-24T09:00:00Z', updated_at = '2026-06-24T09:15:00Z' WHERE id = ?1",
            params![&disabled.id],
        )
        .expect("disabled external client timestamps should update");
    core.db
        .conn
        .execute(
            "UPDATE external_clients SET permission_mode = 'read_only', enabled = 1, created_at = '2026-06-24T08:00:00Z', updated_at = '2026-06-24T08:15:00Z' WHERE id = ?1",
            params![&enabled.id],
        )
        .expect("enabled external client should update");
    core.db
        .conn
        .execute(
            "UPDATE external_clients SET created_at = '2026-06-24T07:00:00Z', updated_at = '2026-06-24T07:15:00Z', deleted_at = '2026-06-24T07:30:00Z' WHERE id = ?1",
            params![&deleted.id],
        )
        .expect("deleted external client should update");

    let expected_order = vec![enabled.id.clone(), disabled.id.clone()];
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let csv_export_path = path.join("external-clients-export.csv");
    let csv_count = core
        .export_external_clients_csv(
            csv_export_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("external clients CSV export should succeed");
    assert_eq!(csv_count, 2);
    let csv_rows = read_csv_export(&csv_export_path);
    assert_eq!(csv_rows.len(), 2);
    let csv_text = std::fs::read_to_string(&csv_export_path).expect("CSV export should read");
    assert!(csv_text.starts_with(
        "id,name,client_type,permission_mode,enabled,created_at,updated_at,deleted_at,device_id\n"
    ));
    assert_eq!(
        csv_rows
            .iter()
            .map(|row| row.get("id").expect("id should export").clone())
            .collect::<Vec<_>>(),
        expected_order
    );
    assert!(csv_rows
        .iter()
        .all(|row| row.get("id") != Some(&deleted.id)));
    let csv_disabled_row = csv_rows
        .iter()
        .find(|row| row.get("id") == Some(&disabled.id))
        .expect("disabled non-deleted placeholder should export");
    assert_eq!(
        csv_disabled_row.get("name"),
        Some(&"Claude Desktop".to_string())
    );
    assert_eq!(
        csv_disabled_row.get("permission_mode"),
        Some(&"disabled".to_string())
    );
    assert_eq!(csv_disabled_row.get("enabled"), Some(&"false".to_string()));
    assert_eq!(csv_disabled_row.get("deleted_at"), Some(&String::new()));
    let csv_enabled_row = csv_rows
        .iter()
        .find(|row| row.get("id") == Some(&enabled.id))
        .expect("enabled non-deleted client should export");
    assert_eq!(
        csv_enabled_row.get("name"),
        Some(&"Local Script".to_string())
    );
    assert_eq!(
        csv_enabled_row.get("client_type"),
        Some(&"script".to_string())
    );
    assert_eq!(
        csv_enabled_row.get("permission_mode"),
        Some(&"read_only".to_string())
    );
    assert_eq!(csv_enabled_row.get("enabled"), Some(&"true".to_string()));
    assert_eq!(
        csv_enabled_row.get("created_at"),
        Some(&"2026-06-24T08:00:00Z".to_string())
    );
    assert_eq!(csv_enabled_row.get("deleted_at"), Some(&String::new()));
    assert_eq!(
        csv_enabled_row.get("device_id"),
        Some(&core.device_id().to_string())
    );

    let json_export_path = path.join("external-clients-export.json");
    let json_count = core
        .export_external_clients_json(
            json_export_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("external clients JSON export should succeed");
    assert_eq!(json_count, 2);
    let json_rows = read_json_export(&json_export_path);
    assert_eq!(
        json_rows
            .iter()
            .map(|row| row["id"].as_str().expect("id should export").to_string())
            .collect::<Vec<_>>(),
        expected_order
    );
    assert!(json_rows.iter().all(|row| row["id"] != deleted.id));
    let json_disabled_row = json_rows
        .iter()
        .find(|row| row["id"] == disabled.id)
        .expect("disabled non-deleted placeholder should export");
    assert_eq!(json_disabled_row["name"], "Claude Desktop");
    assert_eq!(json_disabled_row["permission_mode"], "disabled");
    assert_eq!(json_disabled_row["enabled"], false);
    assert_eq!(json_disabled_row["deleted_at"], serde_json::Value::Null);
    let json_enabled_row = json_rows
        .iter()
        .find(|row| row["id"] == enabled.id)
        .expect("enabled non-deleted client should export");
    assert_eq!(json_enabled_row["name"], "Local Script");
    assert_eq!(json_enabled_row["client_type"], "script");
    assert_eq!(json_enabled_row["permission_mode"], "read_only");
    assert_eq!(json_enabled_row["enabled"], true);
    assert_eq!(json_enabled_row["created_at"], "2026-06-24T08:00:00Z");
    assert_eq!(json_enabled_row["updated_at"], "2026-06-24T08:15:00Z");
    assert_eq!(json_enabled_row["deleted_at"], serde_json::Value::Null);
    assert_eq!(json_enabled_row["device_id"], core.device_id());

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn export_external_client_permissions_csv_and_json_include_rows_in_stable_order_without_mutation() {
    let (mut core, path) = open_test_core();

    let read_client = create_test_external_client_with_mode(&mut core, "read_only");
    let draft_client = create_test_external_client_with_mode(&mut core, "draft_only");

    let read_permission = core
        .upsert_external_client_tool_permission(
            &read_client.id,
            "contacts.search",
            true,
            false,
            false,
        )
        .expect("read permission should insert");
    let lookup_permission = core
        .upsert_external_client_tool_permission(
            &read_client.id,
            "accounts.lookup",
            true,
            false,
            true,
        )
        .expect("second read permission should insert");
    let draft_permission = core
        .upsert_external_client_tool_permission(
            &draft_client.id,
            "create_activity_draft",
            true,
            true,
            true,
        )
        .expect("draft permission should insert");

    core.db
        .conn
        .execute(
            "UPDATE external_client_permissions SET created_at = '2026-06-24T09:00:00Z', updated_at = '2026-06-24T09:15:00Z' WHERE id = ?1",
            params![&read_permission.id],
        )
        .expect("read permission timestamps should update");
    core.db
        .conn
        .execute(
            "UPDATE external_client_permissions SET created_at = '2026-06-24T08:00:00Z', updated_at = '2026-06-24T08:15:00Z' WHERE id = ?1",
            params![&lookup_permission.id],
        )
        .expect("lookup permission timestamps should update");
    core.db
        .conn
        .execute(
            "UPDATE external_client_permissions SET created_at = '2026-06-24T07:00:00Z', updated_at = '2026-06-24T07:15:00Z' WHERE id = ?1",
            params![&draft_permission.id],
        )
        .expect("draft permission timestamps should update");

    let mut expected_order = vec![
        (
            read_client.id.clone(),
            "contacts.search".to_string(),
            "2026-06-24T09:00:00Z".to_string(),
            read_permission.id.clone(),
        ),
        (
            read_client.id.clone(),
            "accounts.lookup".to_string(),
            "2026-06-24T08:00:00Z".to_string(),
            lookup_permission.id.clone(),
        ),
        (
            draft_client.id.clone(),
            "create_activity_draft".to_string(),
            "2026-06-24T07:00:00Z".to_string(),
            draft_permission.id.clone(),
        ),
    ];
    expected_order.sort();
    let expected_ids = expected_order
        .into_iter()
        .map(|(_, _, _, id)| id)
        .collect::<Vec<_>>();
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let csv_export_path = path.join("external-client-permissions-export.csv");
    let csv_count = core
        .export_external_client_permissions_csv(
            csv_export_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("external client permissions CSV export should succeed");
    assert_eq!(csv_count, 3);
    let csv_rows = read_csv_export(&csv_export_path);
    assert_eq!(csv_rows.len(), 3);
    let csv_text = std::fs::read_to_string(&csv_export_path).expect("CSV export should read");
    assert!(csv_text.starts_with(
        "id,client_id,tool_name,can_read,can_write,requires_confirmation,created_at,updated_at\n"
    ));
    assert_eq!(
        csv_rows
            .iter()
            .map(|row| row.get("id").expect("id should export").clone())
            .collect::<Vec<_>>(),
        expected_ids.clone()
    );
    let csv_draft_row = csv_rows
        .iter()
        .find(|row| row.get("id") == Some(&draft_permission.id))
        .expect("draft permission should export");
    assert_eq!(csv_draft_row.get("client_id"), Some(&draft_client.id));
    assert_eq!(
        csv_draft_row.get("tool_name"),
        Some(&"create_activity_draft".to_string())
    );
    assert_eq!(csv_draft_row.get("can_read"), Some(&"true".to_string()));
    assert_eq!(csv_draft_row.get("can_write"), Some(&"true".to_string()));
    assert_eq!(
        csv_draft_row.get("requires_confirmation"),
        Some(&"true".to_string())
    );
    assert_eq!(
        csv_draft_row.get("updated_at"),
        Some(&"2026-06-24T07:15:00Z".to_string())
    );
    let csv_lookup_row = csv_rows
        .iter()
        .find(|row| row.get("id") == Some(&lookup_permission.id))
        .expect("lookup permission should export");
    assert_eq!(
        csv_lookup_row.get("requires_confirmation"),
        Some(&"true".to_string())
    );
    let csv_read_row = csv_rows
        .iter()
        .find(|row| row.get("id") == Some(&read_permission.id))
        .expect("read permission should export");
    assert_eq!(
        csv_read_row.get("requires_confirmation"),
        Some(&"false".to_string())
    );

    let json_export_path = path.join("external-client-permissions-export.json");
    let json_count = core
        .export_external_client_permissions_json(
            json_export_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("external client permissions JSON export should succeed");
    assert_eq!(json_count, 3);
    let json_rows = read_json_export(&json_export_path);
    assert_eq!(
        json_rows
            .iter()
            .map(|row| row["id"].as_str().expect("id should export").to_string())
            .collect::<Vec<_>>(),
        expected_ids
    );
    let json_draft_row = json_rows
        .iter()
        .find(|row| row["id"] == draft_permission.id)
        .expect("draft permission should export");
    assert_eq!(
        json_draft_row
            .as_object()
            .expect("permission row should be object")
            .len(),
        8
    );
    assert_eq!(json_draft_row["client_id"], draft_client.id);
    assert_eq!(json_draft_row["tool_name"], "create_activity_draft");
    assert_eq!(json_draft_row["can_read"], true);
    assert_eq!(json_draft_row["can_write"], true);
    assert_eq!(json_draft_row["requires_confirmation"], true);
    assert_eq!(json_draft_row["created_at"], "2026-06-24T07:00:00Z");
    assert_eq!(json_draft_row["updated_at"], "2026-06-24T07:15:00Z");
    assert!(json_draft_row.get("name").is_none());
    assert!(json_draft_row.get("permission_mode").is_none());

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn create_organization_writes_organization_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            Some("+123456".to_string()),
            None,
            None,
            None,
            Some("Lagos".to_string()),
            None,
            Some("NG".to_string()),
            None,
            Some("Regional partner".to_string()),
        )
        .expect("organization should be created");

    let organization_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM organizations WHERE id = ?1 AND deleted_at IS NULL",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("organization count should query");
    assert_eq!(organization_count, 1);

    let audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'create' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("audit count should query");
    assert_eq!(audit_count, 1);

    let sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = '__create__'",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("sync count should query");
    assert_eq!(sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_organizations_csv_creates_valid_rows_and_skips_blank_names() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("organizations.csv");
    std::fs::write(
        &csv_path,
        "name,email,phone,website,address_line1,address_line2,city,region,country,postal_code,description\n\
         Acme Health,hello@acme.example,+123456,https://acme.example,Dock 4,Suite 9,Lagos,Lagos State,NG,100001,Regional partner\n\
         ,blank@example.com,,,,,,,,,\n",
    )
    .expect("organization CSV fixture should write");

    let result = core
        .import_organizations_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("organization CSV import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());

    let organizations = core
        .list_organizations()
        .expect("organizations should list after import");
    assert_eq!(organizations.len(), 1);
    assert_eq!(organizations[0].name, "Acme Health");
    assert_eq!(
        organizations[0].email.as_deref(),
        Some("hello@acme.example")
    );
    assert_eq!(organizations[0].phone.as_deref(), Some("+123456"));
    assert_eq!(
        organizations[0].website.as_deref(),
        Some("https://acme.example")
    );
    assert_eq!(organizations[0].address_line1.as_deref(), Some("Dock 4"));
    assert_eq!(organizations[0].address_line2.as_deref(), Some("Suite 9"));
    assert_eq!(organizations[0].city.as_deref(), Some("Lagos"));
    assert_eq!(organizations[0].region.as_deref(), Some("Lagos State"));
    assert_eq!(organizations[0].country.as_deref(), Some("NG"));
    assert_eq!(organizations[0].postal_code.as_deref(), Some("100001"));
    assert_eq!(
        organizations[0].description.as_deref(),
        Some("Regional partner")
    );

    let import_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE actor_type = 'import' AND action = 'import_row' AND entity_type = 'organization'",
            [],
            |row| row.get(0),
        )
        .expect("organization import audit count should query");
    assert_eq!(import_audit_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_contacts_json_creates_rows_skips_blank_first_names_and_reports_row_errors() {
    let (mut core, path) = open_test_core();
    let json_path = path.join("contacts.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "first_name": "Ada",
    "last_name": "Lovelace",
    "org_name": "Analytical Engines",
    "email": "ada@example.com",
    "phone": "+15550123",
    "address": "1 Example Street",
    "city": "London",
    "country": "UK",
    "notes": "Prefers email"
  },
  {
    "first_name": "   ",
    "email": "blank@example.com"
  },
  {
    "first_name": "Invalid",
    "email": "invalid-email"
  }
]
"#,
    )
    .expect("contact JSON fixture should write");

    let result = core
        .import_contacts_json(json_path.to_str().expect("path should be valid UTF-8"))
        .expect("contact JSON import should complete with row-level errors");

    assert_eq!(result.created, 1);
    assert_eq!(result.skipped, 1);
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].contains("Row 4:"));
    assert!(result.errors[0].contains("invalid-email"));

    let imported: (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    ) = core
        .db
        .conn
        .query_row(
            "SELECT first_name, last_name, org_name, email, phone, address, city, country, notes FROM contacts WHERE email = ?1",
            params!["ada@example.com"],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            },
        )
        .expect("imported JSON contact should query");
    assert_eq!(
        imported,
        (
            "Ada".to_string(),
            "Lovelace".to_string(),
            "Analytical Engines".to_string(),
            "ada@example.com".to_string(),
            "+15550123".to_string(),
            "1 Example Street".to_string(),
            "London".to_string(),
            "UK".to_string(),
            "Prefers email".to_string(),
        )
    );

    let import_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE actor_type = 'import' AND action = 'import_row' AND entity_type = 'contact'",
            [],
            |row| row.get(0),
        )
        .expect("contact import audit count should query");
    assert_eq!(import_audit_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_contacts_with_auto_merge_fills_blank_fields_without_overwriting_and_creates_new_rows() {
    let (mut core, path) = open_test_core();
    let existing = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            None,
            Some("Existing address".to_string()),
            None,
            None,
            None,
            None,
        )
        .expect("existing contact should be created");

    let csv_path = path.join("contacts-auto-merge.csv");
    std::fs::write(
        &csv_path,
        "First,Last,Mail,Phone,Address,City,Notes\n\
         Imported,Overwrite,ADA@example.com,+15550123,Incoming address,London,Imported note\n\
         Grace,Hopper,grace@example.com,+15550124,,Arlington,\n",
    )
    .expect("contact auto-merge CSV fixture should write");

    let result = core
        .import_contacts_csv_with_mapping_and_options(
            csv_path.to_str().expect("path should be valid UTF-8"),
            import_mapping(&[
                ("First", Some("first_name")),
                ("Last", Some("last_name")),
                ("Mail", Some("email")),
                ("Phone", Some("phone")),
                ("Address", Some("address")),
                ("City", Some("city")),
                ("Notes", Some("notes")),
            ]),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("contact auto-merge import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.merged, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());
    assert_eq!(count(&core, "SELECT COUNT(*) FROM contacts"), 2);

    let merged = core
        .get_contact(&existing.id)
        .expect("merged contact should still exist");
    assert_eq!(merged.first_name, "Ada");
    assert_eq!(merged.last_name, "Lovelace");
    assert_eq!(merged.email, "ada@example.com");
    assert_eq!(merged.phone, "+15550123");
    assert_eq!(merged.address, "Existing address");
    assert_eq!(merged.city, "London");
    assert_eq!(merged.notes, "Imported note");
    assert!(merged.deleted_at.is_none());

    let import_merge_audit = count(
        &core,
        "SELECT COUNT(*) FROM audit_log WHERE actor_type = 'import' AND action = 'import_row_merge' AND entity_type = 'contact'",
    );
    assert_eq!(import_merge_audit, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_rollback_soft_deletes_created_contact_rows_once() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("contacts-created-rollback.csv");
    std::fs::write(
        &csv_path,
        "first_name,last_name,email\nAda,Lovelace,ada@example.com\n",
    )
    .expect("contact CSV fixture should write");

    let result = core
        .import_contacts_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("contact import should succeed");
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("created import should return a rollback plan");

    assert_eq!(rollback_plan.actions.len(), 1);
    assert_eq!(
        core.list_contacts(None)
            .expect("contacts list")
            .contacts
            .len(),
        1
    );

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("created row rollback should complete");

    assert_eq!(rollback.rolled_back, 1);
    assert_eq!(rollback.skipped, 0);
    assert!(rollback.errors.is_empty());
    assert_eq!(
        core.list_contacts(None)
            .expect("contacts list")
            .contacts
            .len(),
        0
    );

    let repeated = core
        .rollback_completed_import(&rollback_plan)
        .expect("repeated rollback should be safe");
    assert_eq!(repeated.rolled_back, 0);
    assert_eq!(repeated.skipped, 1);
    assert_eq!(repeated.errors[0].code, "not_found");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_rollback_soft_deletes_created_deal_and_organization_rows() {
    let (mut core, path) = open_test_core();
    let deal_json_path = path.join("deals-created-rollback.json");
    std::fs::write(
        &deal_json_path,
        r#"[
  {
    "title": "Acme Renewal",
    "value": "12500.50",
    "currency": "EUR",
    "stage": "Proposal",
    "expected_close": "2026-09-30",
    "notes": "Renewal path"
  }
]
"#,
    )
    .expect("deal JSON fixture should write");

    let deal_result = core
        .import_deals_json(deal_json_path.to_str().expect("path should be valid UTF-8"))
        .expect("deal import should succeed");
    let deal_rollback_plan = deal_result
        .rollback_plan
        .clone()
        .expect("created deal import should return a rollback plan");
    assert_eq!(deal_rollback_plan.actions.len(), 1);
    assert_eq!(core.list_deals().expect("deals list").len(), 1);

    let deal_rollback = core
        .rollback_completed_import(&deal_rollback_plan)
        .expect("created deal rollback should complete");
    assert_eq!(deal_rollback.rolled_back, 1);
    assert_eq!(deal_rollback.skipped, 0);
    assert!(deal_rollback.errors.is_empty());
    assert_eq!(core.list_deals().expect("deals list").len(), 0);

    let organization_csv_path = path.join("organizations-created-rollback.csv");
    std::fs::write(
        &organization_csv_path,
        "name,email,phone\nAcme Health,hello@acme.example,+2345550100\n",
    )
    .expect("organization CSV fixture should write");

    let organization_result = core
        .import_organizations_csv(
            organization_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("organization import should succeed");
    let organization_rollback_plan = organization_result
        .rollback_plan
        .clone()
        .expect("created organization import should return a rollback plan");
    assert_eq!(organization_rollback_plan.actions.len(), 1);
    assert_eq!(
        core.list_organizations().expect("organizations list").len(),
        1
    );

    let organization_rollback = core
        .rollback_completed_import(&organization_rollback_plan)
        .expect("created organization rollback should complete");
    assert_eq!(organization_rollback.rolled_back, 1);
    assert_eq!(organization_rollback.skipped, 0);
    assert!(organization_rollback.errors.is_empty());
    assert_eq!(
        core.list_organizations().expect("organizations list").len(),
        0
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_notes_csv_preflights_imports_exports_and_rolls_back_generic_notes() {
    let (mut core, path) = open_test_core();
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should create");
    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should create");
    let deal = core
        .create_deal(
            "Acme Renewal".to_string(),
            Some(12500.0),
            Some("USD".to_string()),
            Some("Lead".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("deal should create");
    let activity = core
        .create_activity(
            "call".to_string(),
            "Intro call".to_string(),
            None,
            None,
            None,
            None,
        )
        .expect("activity should create");

    let csv_path = path.join("notes.csv");
    std::fs::write(
        &csv_path,
        format!(
            "entity_type,entity_id,content\n\
             contact,{},Contact note\n\
             organization,{},Organization note\n\
             deal,{},Deal note\n\
             activity,{},Activity note\n",
            contact.id, organization.id, deal.id, activity.id
        ),
    )
    .expect("note CSV fixture should write");

    let preflight = core
        .preflight_notes_csv_import(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("note preflight should succeed");
    assert_eq!(preflight.entity_type, "notes");
    assert_eq!(preflight.total_rows, 4);
    assert_eq!(preflight.duplicate_warning_count, 0);
    assert!(preflight.warnings.is_empty());
    assert!(core.list_notes().expect("notes list").is_empty());

    let result = core
        .import_notes_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("note import should succeed");
    assert_eq!(result.created, 4);
    assert_eq!(result.merged, 0);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());
    assert_eq!(
        result
            .rollback_plan
            .as_ref()
            .expect("note import should return rollback plan")
            .actions
            .len(),
        4
    );

    let notes = core.list_notes().expect("notes should list");
    assert_eq!(notes.len(), 4);
    assert!(notes.iter().any(|note| note.content == "Contact note"));
    assert!(notes
        .iter()
        .any(|note| note.entity_type == "organization" && note.entity_id == organization.id));

    let csv_export_path = path.join("notes-export.csv");
    assert_eq!(
        core.export_notes_csv(
            csv_export_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("note CSV export should succeed"),
        4
    );
    let csv_rows = read_csv_export(&csv_export_path);
    assert_eq!(csv_rows.len(), 4);
    assert_eq!(csv_rows[0].keys().len(), 3);
    assert!(csv_rows
        .iter()
        .any(|row| row["entity_type"] == "contact" && row["content"] == "Contact note"));

    let json_export_path = path.join("notes-export.json");
    assert_eq!(
        core.export_notes_json(
            json_export_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("note JSON export should succeed"),
        4
    );
    let json_rows = read_json_export(&json_export_path);
    assert!(json_rows.iter().all(|row| row.get("id").is_none()));
    assert!(json_rows
        .iter()
        .any(|row| row["entity_type"] == "activity" && row["content"] == "Activity note"));

    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("note import should return rollback plan");
    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("note rollback should complete");
    assert_eq!(rollback.rolled_back, 4);
    assert_eq!(rollback.skipped, 0);
    assert!(rollback.errors.is_empty());
    assert!(core
        .list_notes()
        .expect("notes list after rollback")
        .is_empty());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn preflight_notes_csv_import_validates_active_targets_without_writes() {
    let (mut core, path) = open_test_core();
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Deleted".to_string()),
            Some("Contact".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should create");
    core.delete_contact(&contact.id)
        .expect("contact should be soft deleted");

    let csv_path = path.join("notes-invalid.csv");
    std::fs::write(
        &csv_path,
        format!(
            "entity_type,entity_id,content\ncontact,{},Cannot attach\n",
            contact.id
        ),
    )
    .expect("invalid note CSV fixture should write");

    let error = core
        .preflight_notes_csv_import(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect_err("preflight should reject deleted parent entity");
    assert!(error.to_string().contains("Row 2"));
    assert!(error.to_string().contains("not found"));
    assert!(core.list_notes().expect("notes should list").is_empty());

    let unsupported_path = path.join("notes-unsupported.csv");
    std::fs::write(
        &unsupported_path,
        "entity_type,entity_id,content\ninvoice,local-1,Unsupported\n",
    )
    .expect("unsupported note CSV fixture should write");
    let unsupported_error = core
        .preflight_notes_csv_import(
            unsupported_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect_err("preflight should reject unsupported entity type");
    assert!(unsupported_error
        .to_string()
        .contains("Unsupported entity_type 'invoice'"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_notes_json_with_mapping_uses_note_service_and_conflict_safe_rollback() {
    let (mut core, path) = open_test_core();
    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should create");
    let json_path = path.join("notes-mapped.json");
    std::fs::write(
        &json_path,
        format!(
            r#"[
  {{
    "Kind": "organization",
    "Target": "{}",
    "Body": "Mapped organization note"
  }}
]
"#,
            organization.id
        ),
    )
    .expect("mapped note JSON fixture should write");
    let mapping = import_mapping(&[
        ("Kind", Some("entity_type")),
        ("Target", Some("entity_id")),
        ("Body", Some("content")),
    ]);

    let preflight = core
        .preflight_notes_json_import_with_mapping(
            json_path.to_str().expect("path should be valid UTF-8"),
            mapping.clone(),
        )
        .expect("mapped note preflight should succeed");
    assert_eq!(preflight.entity_type, "notes");
    assert_eq!(preflight.duplicate_warning_count, 0);

    let result = core
        .import_notes_json_with_mapping(
            json_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped note import should succeed");
    assert_eq!(result.created, 1);
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("created note import should return rollback plan");
    let note_id = match &rollback_plan.actions[0] {
        super::import_rollback::ImportRollbackAction::Note { entity_id, .. } => entity_id.clone(),
        other => panic!("expected note rollback action, got {other:?}"),
    };
    let note = core.get_note(&note_id).expect("imported note should exist");
    assert_eq!(note.content, "Mapped organization note");
    assert_eq!(note.entity_type, "organization");
    assert_eq!(note.entity_id, organization.id);

    core.update_note(&note_id, "Edited after import".to_string())
        .expect("note should update after import");
    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("note rollback should complete with conflict skip");
    assert_eq!(rollback.rolled_back, 0);
    assert_eq!(rollback.skipped, 1);
    assert_eq!(rollback.errors[0].code, "conflict");
    assert_eq!(
        core.get_note(&note_id)
            .expect("conflicted note should remain")
            .content,
        "Edited after import"
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_rollback_restores_only_changed_contact_merge_fields() {
    let (mut core, path) = open_test_core();
    let existing = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            None,
            Some("Existing address".to_string()),
            None,
            None,
            None,
            None,
        )
        .expect("existing contact should be created");

    let csv_path = path.join("contacts-merge-rollback.csv");
    std::fs::write(
        &csv_path,
        "first_name,last_name,email,phone,address,city,notes\n\
         Imported,Overwrite,ADA@example.com,+15550123,Incoming address,London,Imported note\n",
    )
    .expect("contact auto-merge CSV fixture should write");

    let result = core
        .import_contacts_csv_with_options(
            csv_path.to_str().expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("contact auto-merge import should succeed");
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("merge import should return a rollback plan");

    let merged = core
        .get_contact(&existing.id)
        .expect("merged contact should exist");
    assert_eq!(merged.first_name, "Ada");
    assert_eq!(merged.last_name, "Lovelace");
    assert_eq!(merged.email, "ada@example.com");
    assert_eq!(merged.phone, "+15550123");
    assert_eq!(merged.address, "Existing address");
    assert_eq!(merged.city, "London");
    assert_eq!(merged.notes, "Imported note");

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("merge rollback should complete");
    assert_eq!(rollback.rolled_back, 1);
    assert_eq!(rollback.skipped, 0);
    assert!(rollback.errors.is_empty());

    let restored = core
        .get_contact(&existing.id)
        .expect("contact should still exist after rollback");
    assert_eq!(restored.first_name, "Ada");
    assert_eq!(restored.last_name, "Lovelace");
    assert_eq!(restored.email, "ada@example.com");
    assert_eq!(restored.phone, "");
    assert_eq!(restored.address, "Existing address");
    assert_eq!(restored.city, "");
    assert_eq!(restored.notes, "");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_contacts_duplicate_auto_merge_disabled_preserves_create_behavior() {
    let (mut core, path) = open_test_core();
    core.create_contact(
        Some("person".to_string()),
        Some("Ada".to_string()),
        Some("Lovelace".to_string()),
        None,
        Some("ada@example.com".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("existing contact should be created");

    let csv_path = path.join("contacts-no-auto-merge.csv");
    std::fs::write(
        &csv_path,
        "first_name,last_name,email,phone\nImported,Duplicate,ada@example.com,+15550123\n",
    )
    .expect("contact CSV fixture should write");

    let result = core
        .import_contacts_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("default contact import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.merged, 0);
    assert_eq!(result.skipped, 0);
    assert_eq!(count(&core, "SELECT COUNT(*) FROM contacts"), 2);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_contacts_with_auto_merge_skips_ambiguous_matches_with_row_error() {
    let (mut core, path) = open_test_core();
    core.create_contact(
        Some("person".to_string()),
        Some("Ada".to_string()),
        Some("Lovelace".to_string()),
        None,
        Some("ada@example.com".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("email-matched contact should be created");
    core.create_contact(
        Some("person".to_string()),
        Some("Grace".to_string()),
        Some("Hopper".to_string()),
        None,
        None,
        Some("+15550123".to_string()),
        None,
        None,
        None,
        None,
        None,
    )
    .expect("phone-matched contact should be created");

    let csv_path = path.join("contacts-ambiguous-auto-merge.csv");
    std::fs::write(
        &csv_path,
        "first_name,last_name,email,phone\nImported,Duplicate,ada@example.com,+15550123\n",
    )
    .expect("ambiguous contact CSV fixture should write");

    let result = core
        .import_contacts_csv_with_options(
            csv_path.to_str().expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("ambiguous contact auto-merge import should finish with row error");

    assert_eq!(result.created, 0);
    assert_eq!(result.merged, 0);
    assert_eq!(result.skipped, 1);
    assert_eq!(count(&core, "SELECT COUNT(*) FROM contacts"), 2);
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].starts_with("Row 2:"));
    assert!(
        result.errors[0].contains("duplicate auto-merge skipped because multiple contacts match:")
    );
    assert!(result.errors[0].contains("(Imported)"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_organizations_with_auto_merge_fills_blank_fields_without_overwriting() {
    let (mut core, path) = open_test_core();
    let existing = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            None,
            None,
            Some("Dock 4".to_string()),
            None,
            Some("Lagos".to_string()),
            None,
            None,
            None,
            None,
        )
        .expect("existing organization should be created");

    let json_path = path.join("organizations-auto-merge.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "Company": "acme health",
    "Inbox": "new@acme.example",
    "Telephone": "+2345550100",
    "Site": "https://acme.example",
    "Line 1": "Incoming address",
    "City": "Nairobi",
    "Description": "Imported description"
  }
]"#,
    )
    .expect("organization auto-merge JSON fixture should write");

    let result = core
        .import_organizations_json_with_mapping_and_options(
            json_path.to_str().expect("path should be valid UTF-8"),
            import_mapping(&[
                ("Company", Some("name")),
                ("Inbox", Some("email")),
                ("Telephone", Some("phone")),
                ("Site", Some("website")),
                ("Line 1", Some("address_line1")),
                ("City", Some("city")),
                ("Description", Some("description")),
            ]),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("organization auto-merge import should succeed");

    assert_eq!(result.created, 0);
    assert_eq!(result.merged, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());
    assert_eq!(count(&core, "SELECT COUNT(*) FROM organizations"), 1);

    let merged = core
        .get_organization(&existing.id)
        .expect("merged organization should still exist");
    assert_eq!(merged.name, "Acme Health");
    assert_eq!(merged.email.as_deref(), Some("hello@acme.example"));
    assert_eq!(merged.phone.as_deref(), Some("+2345550100"));
    assert_eq!(merged.website.as_deref(), Some("https://acme.example"));
    assert_eq!(merged.address_line1.as_deref(), Some("Dock 4"));
    assert_eq!(merged.city.as_deref(), Some("Lagos"));
    assert_eq!(merged.description.as_deref(), Some("Imported description"));
    assert!(merged.deleted_at.is_none());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_rollback_restores_changed_organization_merge_fields() {
    let (mut core, path) = open_test_core();
    let existing = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            None,
            None,
            Some("Dock 4".to_string()),
            None,
            Some("Lagos".to_string()),
            None,
            None,
            None,
            None,
        )
        .expect("existing organization should be created");

    let json_path = path.join("organizations-merge-rollback.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "name": "acme health",
    "email": "new@acme.example",
    "phone": "+2345550100",
    "website": "https://acme.example",
    "address_line1": "Incoming address",
    "description": "Imported description"
  }
]"#,
    )
    .expect("organization auto-merge JSON fixture should write");

    let result = core
        .import_organizations_json_with_options(
            json_path.to_str().expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("organization auto-merge import should succeed");
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("organization merge import should return a rollback plan");

    let merged = core
        .get_organization(&existing.id)
        .expect("organization should still exist");
    assert_eq!(merged.email.as_deref(), Some("hello@acme.example"));
    assert_eq!(merged.phone.as_deref(), Some("+2345550100"));
    assert_eq!(merged.website.as_deref(), Some("https://acme.example"));
    assert_eq!(merged.address_line1.as_deref(), Some("Dock 4"));
    assert_eq!(merged.description.as_deref(), Some("Imported description"));

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("organization merge rollback should complete");
    assert_eq!(rollback.rolled_back, 1);
    assert_eq!(rollback.skipped, 0);
    assert!(rollback.errors.is_empty());

    let restored = core
        .get_organization(&existing.id)
        .expect("organization should still exist after rollback");
    assert_eq!(restored.email.as_deref(), Some("hello@acme.example"));
    assert_eq!(restored.phone, None);
    assert_eq!(restored.website, None);
    assert_eq!(restored.address_line1.as_deref(), Some("Dock 4"));
    assert_eq!(restored.description, None);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_organizations_with_auto_merge_skips_ambiguous_matches_with_row_error() {
    let (mut core, path) = open_test_core();
    core.create_organization(
        "Acme Health".to_string(),
        Some("hello@acme.example".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("email-matched organization should be created");
    core.create_organization(
        "Other Org".to_string(),
        None,
        Some("+2345550100".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("phone-matched organization should be created");

    let json_path = path.join("organizations-ambiguous-auto-merge.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "name": "Imported Org",
    "email": "hello@acme.example",
    "phone": "+2345550100"
  }
]"#,
    )
    .expect("ambiguous organization JSON fixture should write");

    let result = core
        .import_organizations_json_with_options(
            json_path.to_str().expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("ambiguous organization auto-merge import should finish with row error");

    assert_eq!(result.created, 0);
    assert_eq!(result.merged, 0);
    assert_eq!(result.skipped, 1);
    assert_eq!(count(&core, "SELECT COUNT(*) FROM organizations"), 2);
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].starts_with("Row 2:"));
    assert!(result.errors[0]
        .contains("duplicate auto-merge skipped because multiple organizations match:"));
    assert!(result.errors[0].contains("(Imported Org)"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn preview_contacts_json_reports_headers_row_numbers_and_does_not_write() {
    let (core, path) = open_test_core();
    let json_path = path.join("contacts-preview.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "first_name": "Ada",
    "last_name": "Lovelace",
    "email": "ada@example.com",
    "ignored": "not shown"
  },
  {
    "first_name": "   ",
    "email": "blank@example.com"
  },
  {
    "first_name": "Grace",
    "phone": "+15550100"
  }
]
"#,
    )
    .expect("contact preview JSON fixture should write");

    let contact_count_before = count(&core, "SELECT COUNT(*) FROM contacts");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let preview = core
        .preview_contacts_json_import(json_path.to_str().expect("path should be valid UTF-8"))
        .expect("contact JSON preview should parse");

    assert_eq!(preview.total_rows, 3);
    assert_eq!(
        preview.headers,
        vec!["email", "first_name", "ignored", "last_name", "phone"]
    );
    assert_eq!(preview.rows.len(), 3);
    assert_eq!(preview.rows[0].row_number, 2);
    assert_eq!(
        preview.rows[0].values.get("first_name").map(String::as_str),
        Some("Ada")
    );
    assert_eq!(
        preview.rows[0].values.get("email").map(String::as_str),
        Some("ada@example.com")
    );
    assert_eq!(
        preview.rows[0].values.get("ignored").map(String::as_str),
        Some("not shown")
    );
    assert_eq!(preview.rows[1].row_number, 3);
    assert_eq!(
        preview.rows[1].values.get("first_name").map(String::as_str),
        Some("   ")
    );
    assert_eq!(preview.rows[2].row_number, 4);
    assert_eq!(
        preview.rows[2].values.get("phone").map(String::as_str),
        Some("+15550100")
    );

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM contacts"),
        contact_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn preview_json_rejects_unsupported_shape_before_import() {
    let (mut core, path) = open_test_core();
    let json_path = path.join("contacts-invalid-preview.json");
    std::fs::write(&json_path, r#"{ "first_name": "Ada" }"#)
        .expect("invalid preview JSON fixture should write");

    let preview_err = core
        .preview_contacts_json_import(json_path.to_str().expect("path should be valid UTF-8"))
        .expect_err("object JSON import preview should be rejected");
    match preview_err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("JSON import expects a top-level array of objects"));
        }
        other => panic!("expected InvalidInput for unsupported JSON shape, got {other:?}"),
    }

    let import_err = core
        .import_contacts_json(json_path.to_str().expect("path should be valid UTF-8"))
        .expect_err("object JSON import should also be rejected");
    match import_err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("JSON import expects a top-level array of objects"));
        }
        other => panic!("expected InvalidInput for unsupported JSON shape, got {other:?}"),
    }

    assert_eq!(count(&core, "SELECT COUNT(*) FROM contacts"), 0);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_deals_json_creates_valid_rows_and_skips_blank_titles() {
    let (mut core, path) = open_test_core();
    let json_path = path.join("deals.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "title": "Acme Renewal",
    "value": "12500.50",
    "currency": "EUR",
    "stage": "Proposal",
    "expected_close": "2026-09-30",
    "notes": "Renewal path"
  },
  {
    "title": "  ",
    "value": "2500",
    "currency": "USD"
  }
]
"#,
    )
    .expect("deal JSON fixture should write");

    let result = core
        .import_deals_json(json_path.to_str().expect("path should be valid UTF-8"))
        .expect("deal JSON import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());

    let deals = core
        .list_deals()
        .expect("deals should list after JSON import");
    assert_eq!(deals.len(), 1);
    assert_eq!(deals[0].title, "Acme Renewal");
    assert_eq!(deals[0].value, 12500.50);
    assert_eq!(deals[0].currency, "EUR");
    assert_eq!(deals[0].stage, "Proposal");
    assert_eq!(deals[0].expected_close.as_deref(), Some("2026-09-30"));
    assert_eq!(deals[0].notes, "Renewal path");
    assert_eq!(deals[0].contact_id, None);
    assert_eq!(deals[0].organization_id, None);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_deals_csv_with_auto_merge_fills_safe_fields_without_overwriting_and_creates_new_rows() {
    let (mut core, path) = open_test_core();
    let existing = core
        .create_deal(
            "Acme Renewal".to_string(),
            Some(0.0),
            Some("EUR".to_string()),
            Some("Proposal".to_string()),
            Some(50),
            None,
            None,
            None,
            None,
        )
        .expect("existing deal should be created");

    let csv_path = path.join("deals-auto-merge.csv");
    std::fs::write(
        &csv_path,
        "title,value,currency,stage,expected_close,notes\n\
         acme renewal,12500.50,USD,Negotiation,2026-10-15,Imported note\n\
         Lagos Expansion,9000,GBP,Lead,2026-11-01,New deal\n",
    )
    .expect("deal auto-merge CSV fixture should write");

    let result = core
        .import_deals_csv_with_options(
            csv_path.to_str().expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("deal auto-merge import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.merged, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());
    assert_eq!(count(&core, "SELECT COUNT(*) FROM deals"), 2);

    let merged = core
        .get_deal(&existing.id)
        .expect("merged deal should still exist");
    assert_eq!(merged.title, "Acme Renewal");
    assert_eq!(merged.value, 12500.50);
    assert_eq!(merged.currency, "EUR");
    assert_eq!(merged.stage, "Proposal");
    assert_eq!(merged.expected_close.as_deref(), Some("2026-10-15"));
    assert_eq!(merged.notes, "Imported note");
    assert!(merged.contact_id.is_none());
    assert!(merged.organization_id.is_none());
    assert!(merged.deleted_at.is_none());

    let import_merge_audit = count(
        &core,
        "SELECT COUNT(*) FROM audit_log WHERE actor_type = 'import' AND action = 'import_row_merge' AND entity_type = 'deal'",
    );
    assert_eq!(import_merge_audit, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_rollback_restores_changed_deal_merge_fields() {
    let (mut core, path) = open_test_core();
    let existing = core
        .create_deal(
            "Acme Renewal".to_string(),
            Some(0.0),
            Some("EUR".to_string()),
            Some("Proposal".to_string()),
            Some(50),
            None,
            None,
            None,
            None,
        )
        .expect("existing deal should be created");

    let csv_path = path.join("deals-merge-rollback.csv");
    std::fs::write(
        &csv_path,
        "title,value,currency,stage,expected_close,notes\n\
         acme renewal,12500.50,USD,Negotiation,2026-10-15,Imported note\n",
    )
    .expect("deal auto-merge CSV fixture should write");

    let result = core
        .import_deals_csv_with_options(
            csv_path.to_str().expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("deal auto-merge import should succeed");
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("deal merge import should return a rollback plan");

    let merged = core
        .get_deal(&existing.id)
        .expect("merged deal should still exist");
    assert_eq!(merged.title, "Acme Renewal");
    assert_eq!(merged.value, 12500.50);
    assert_eq!(merged.currency, "EUR");
    assert_eq!(merged.stage, "Proposal");
    assert_eq!(merged.expected_close.as_deref(), Some("2026-10-15"));
    assert_eq!(merged.notes, "Imported note");

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("deal merge rollback should complete");
    assert_eq!(rollback.rolled_back, 1);
    assert_eq!(rollback.skipped, 0);
    assert!(rollback.errors.is_empty());

    let restored = core
        .get_deal(&existing.id)
        .expect("deal should still exist after rollback");
    assert_eq!(restored.title, "Acme Renewal");
    assert_eq!(restored.value, 0.0);
    assert_eq!(restored.currency, "EUR");
    assert_eq!(restored.stage, "Proposal");
    assert_eq!(restored.expected_close, None);
    assert_eq!(restored.notes, "");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_rollback_skips_merge_when_deal_changed_after_import() {
    let (mut core, path) = open_test_core();
    let existing = core
        .create_deal(
            "Acme Renewal".to_string(),
            Some(0.0),
            Some("EUR".to_string()),
            Some("Proposal".to_string()),
            Some(50),
            None,
            None,
            None,
            None,
        )
        .expect("existing deal should be created");

    let csv_path = path.join("deals-merge-conflict-rollback.csv");
    std::fs::write(
        &csv_path,
        "title,value,currency,stage,expected_close,notes\n\
         acme renewal,12500.50,USD,Negotiation,2026-10-15,Imported note\n",
    )
    .expect("deal auto-merge CSV fixture should write");

    let result = core
        .import_deals_csv_with_options(
            csv_path.to_str().expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("deal auto-merge import should succeed");
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("merge import should return a rollback plan");

    core.update_deal(
        &existing.id,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("User edited note".to_string()),
    )
    .expect("post-import user edit should succeed");

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("conflicted merge rollback should complete");

    assert_eq!(rollback.rolled_back, 0);
    assert_eq!(rollback.skipped, 1);
    assert_eq!(rollback.errors[0].code, "conflict");

    let deal = core
        .get_deal(&existing.id)
        .expect("deal should remain after skipped rollback");
    assert_eq!(deal.value, 12500.50);
    assert_eq!(deal.expected_close.as_deref(), Some("2026-10-15"));
    assert_eq!(deal.notes, "User edited note");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_deals_duplicate_auto_merge_disabled_preserves_create_behavior() {
    let (mut core, path) = open_test_core();
    core.create_deal(
        "Acme Renewal".to_string(),
        Some(5000.0),
        Some("USD".to_string()),
        Some("Proposal".to_string()),
        Some(50),
        None,
        None,
        None,
        None,
    )
    .expect("existing deal should be created");

    let csv_path = path.join("deals-no-auto-merge.csv");
    std::fs::write(
        &csv_path,
        "title,value,currency,stage,expected_close,notes\n\
         acme renewal,7500,EUR,Negotiation,2026-10-15,Potential duplicate\n",
    )
    .expect("deal CSV fixture should write");

    let result = core
        .import_deals_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("default deal import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.merged, 0);
    assert_eq!(result.skipped, 0);
    assert_eq!(count(&core, "SELECT COUNT(*) FROM deals"), 2);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_deals_json_with_auto_merge_fills_only_blank_or_default_safe_fields() {
    let (mut core, path) = open_test_core();
    let existing = core
        .create_deal(
            "Acme Renewal".to_string(),
            Some(5000.0),
            Some("USD".to_string()),
            Some("Proposal".to_string()),
            Some(50),
            Some("2026-09-30".to_string()),
            None,
            None,
            Some("Existing note".to_string()),
        )
        .expect("existing deal should be created");

    let json_path = path.join("deals-auto-merge.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "title": " acme renewal ",
    "value": "7500",
    "currency": "EUR",
    "stage": "Negotiation",
    "expected_close": "2026-10-15",
    "notes": "Imported note"
  }
]
"#,
    )
    .expect("deal auto-merge JSON fixture should write");

    let result = core
        .import_deals_json_with_options(
            json_path.to_str().expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("deal JSON auto-merge import should succeed");

    assert_eq!(result.created, 0);
    assert_eq!(result.merged, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());
    assert_eq!(count(&core, "SELECT COUNT(*) FROM deals"), 1);

    let merged = core
        .get_deal(&existing.id)
        .expect("merged deal should still exist");
    assert_eq!(merged.title, "Acme Renewal");
    assert_eq!(merged.value, 5000.0);
    assert_eq!(merged.currency, "USD");
    assert_eq!(merged.stage, "Proposal");
    assert_eq!(merged.expected_close.as_deref(), Some("2026-09-30"));
    assert_eq!(merged.notes, "Existing note");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_deals_mapped_csv_with_auto_merge_uses_title_duplicate_rule() {
    let (mut core, path) = open_test_core();
    let existing = core
        .create_deal(
            " Acme Renewal ".to_string(),
            Some(0.0),
            Some("USD".to_string()),
            Some("Lead".to_string()),
            Some(10),
            None,
            None,
            None,
            None,
        )
        .expect("existing deal should be created");

    let csv_path = path.join("deals-mapped-auto-merge.csv");
    std::fs::write(
        &csv_path,
        "Opportunity,Amount,ISO,Phase,Close Date,Memo\n\
         ACME RENEWAL,4250,EUR,Negotiation,2026-12-01,Mapped note\n",
    )
    .expect("mapped deal CSV fixture should write");

    let result = core
        .import_deals_csv_with_mapping_and_options(
            csv_path.to_str().expect("path should be valid UTF-8"),
            import_mapping(&[
                ("Opportunity", Some("title")),
                ("Amount", Some("value")),
                ("ISO", Some("currency")),
                ("Phase", Some("stage")),
                ("Close Date", Some("expected_close")),
                ("Memo", Some("notes")),
            ]),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("mapped deal auto-merge import should succeed");

    assert_eq!(result.created, 0);
    assert_eq!(result.merged, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());

    let merged = core
        .get_deal(&existing.id)
        .expect("merged deal should still exist");
    assert_eq!(merged.title, " Acme Renewal ");
    assert_eq!(merged.value, 4250.0);
    assert_eq!(merged.currency, "USD");
    assert_eq!(merged.stage, "Lead");
    assert_eq!(merged.expected_close.as_deref(), Some("2026-12-01"));
    assert_eq!(merged.notes, "Mapped note");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_deals_mapped_json_with_auto_merge_skips_ambiguous_title_matches() {
    let (mut core, path) = open_test_core();
    for stage in ["Lead", "Proposal"] {
        core.create_deal(
            "Acme Renewal".to_string(),
            Some(1000.0),
            Some("USD".to_string()),
            Some(stage.to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("duplicate deal should be created");
    }

    let json_path = path.join("deals-mapped-ambiguous-auto-merge.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "Opportunity": " acme renewal ",
    "Amount": "7500",
    "Memo": "Ambiguous duplicate"
  }
]
"#,
    )
    .expect("mapped deal JSON fixture should write");

    let result = core
        .import_deals_json_with_mapping_and_options(
            json_path.to_str().expect("path should be valid UTF-8"),
            import_mapping(&[
                ("Opportunity", Some("title")),
                ("Amount", Some("value")),
                ("Memo", Some("notes")),
            ]),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("ambiguous deal auto-merge import should finish with row error");

    assert_eq!(result.created, 0);
    assert_eq!(result.merged, 0);
    assert_eq!(result.skipped, 1);
    assert_eq!(count(&core, "SELECT COUNT(*) FROM deals"), 2);
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].starts_with("Row 2:"));
    assert!(result.errors[0].contains("duplicate auto-merge skipped because multiple deals match"));
    assert!(result.errors[0].contains("(acme renewal)"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_organizations_json_creates_valid_rows_and_skips_blank_names() {
    let (mut core, path) = open_test_core();
    let json_path = path.join("organizations.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "name": "Acme Health",
    "email": "hello@acme.example",
    "phone": "+123456",
    "website": "https://acme.example",
    "address_line1": "Dock 4",
    "address_line2": "Suite 9",
    "city": "Lagos",
    "region": "Lagos State",
    "country": "NG",
    "postal_code": "100001",
    "description": "Regional partner"
  },
  {
    "name": "",
    "email": "blank@example.com"
  }
]
"#,
    )
    .expect("organization JSON fixture should write");

    let result = core
        .import_organizations_json(json_path.to_str().expect("path should be valid UTF-8"))
        .expect("organization JSON import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());

    let organizations = core
        .list_organizations()
        .expect("organizations should list after JSON import");
    assert_eq!(organizations.len(), 1);
    assert_eq!(organizations[0].name, "Acme Health");
    assert_eq!(
        organizations[0].email.as_deref(),
        Some("hello@acme.example")
    );
    assert_eq!(organizations[0].phone.as_deref(), Some("+123456"));
    assert_eq!(
        organizations[0].website.as_deref(),
        Some("https://acme.example")
    );
    assert_eq!(organizations[0].address_line1.as_deref(), Some("Dock 4"));
    assert_eq!(organizations[0].address_line2.as_deref(), Some("Suite 9"));
    assert_eq!(organizations[0].city.as_deref(), Some("Lagos"));
    assert_eq!(organizations[0].region.as_deref(), Some("Lagos State"));
    assert_eq!(organizations[0].country.as_deref(), Some("NG"));
    assert_eq!(organizations[0].postal_code.as_deref(), Some("100001"));
    assert_eq!(
        organizations[0].description.as_deref(),
        Some("Regional partner")
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_contacts_json_reports_row_number_for_non_object_rows() {
    let (mut core, path) = open_test_core();
    let json_path = path.join("contacts-invalid-row.json");
    std::fs::write(
        &json_path,
        r#"[
  { "first_name": "Ada" },
  42
]
"#,
    )
    .expect("invalid contact JSON fixture should write");

    let err = core
        .import_contacts_json(json_path.to_str().expect("path should be valid UTF-8"))
        .expect_err("non-object JSON rows should be rejected");

    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("JSON row 3 must be an object"));
        }
        other => panic!("expected InvalidInput for non-object JSON row, got {other:?}"),
    }

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn mapped_json_import_creates_contacts_deals_and_organizations() {
    let (mut core, path) = open_test_core();

    let contacts_path = path.join("contacts-mapped.json");
    std::fs::write(
        &contacts_path,
        r#"[
  {
    "Given": "Ada",
    "Surname": "Lovelace",
    "Mail": "ada@example.com",
    "Ignored": "ignored"
  }
]
"#,
    )
    .expect("mapped contact JSON fixture should write");
    let contact_mapping = import_mapping(&[
        ("Given", Some("first_name")),
        ("Surname", Some("last_name")),
        ("Mail", Some("email")),
        ("Ignored", None),
    ]);

    let contact_result = core
        .import_contacts_json_with_mapping(
            contacts_path.to_str().expect("path should be valid UTF-8"),
            contact_mapping,
        )
        .expect("mapped contact JSON import should succeed");
    assert_eq!(contact_result.created, 1);
    assert_eq!(contact_result.skipped, 0);

    let deals_path = path.join("deals-mapped.json");
    std::fs::write(
        &deals_path,
        r#"[
  {
    "Opportunity": "Acme Renewal",
    "Amount": 12500.5,
    "Phase": "Proposal",
    "Close": "2026-09-30"
  }
]
"#,
    )
    .expect("mapped deal JSON fixture should write");
    let deal_mapping = import_mapping(&[
        ("Opportunity", Some("title")),
        ("Amount", Some("value")),
        ("Phase", Some("stage")),
        ("Close", Some("expected_close")),
    ]);

    let deal_result = core
        .import_deals_json_with_mapping(
            deals_path.to_str().expect("path should be valid UTF-8"),
            deal_mapping,
        )
        .expect("mapped deal JSON import should succeed");
    assert_eq!(deal_result.created, 1);
    assert_eq!(deal_result.skipped, 0);

    let organizations_path = path.join("organizations-mapped.json");
    std::fs::write(
        &organizations_path,
        r#"[
  {
    "Company": "Acme Health",
    "Inbox": "hello@acme.example",
    "Telephone": "+123456"
  }
]
"#,
    )
    .expect("mapped organization JSON fixture should write");
    let organization_mapping = import_mapping(&[
        ("Company", Some("name")),
        ("Inbox", Some("email")),
        ("Telephone", Some("phone")),
    ]);

    let organization_result = core
        .import_organizations_json_with_mapping(
            organizations_path
                .to_str()
                .expect("path should be valid UTF-8"),
            organization_mapping,
        )
        .expect("mapped organization JSON import should succeed");
    assert_eq!(organization_result.created, 1);
    assert_eq!(organization_result.skipped, 0);

    let contacts = core
        .list_contacts(None)
        .expect("contacts should list after mapped JSON import");
    assert_eq!(contacts.contacts.len(), 1);
    assert_eq!(contacts.contacts[0].first_name, "Ada");
    assert_eq!(contacts.contacts[0].last_name, "Lovelace");
    assert_eq!(contacts.contacts[0].email, "ada@example.com");

    let deals = core
        .list_deals()
        .expect("deals should list after mapped JSON import");
    assert_eq!(deals.len(), 1);
    assert_eq!(deals[0].title, "Acme Renewal");
    assert_eq!(deals[0].value, 12500.5);
    assert_eq!(deals[0].stage, "Proposal");
    assert_eq!(deals[0].expected_close.as_deref(), Some("2026-09-30"));

    let organizations = core
        .list_organizations()
        .expect("organizations should list after mapped JSON import");
    assert_eq!(organizations.len(), 1);
    assert_eq!(organizations[0].name, "Acme Health");
    assert_eq!(
        organizations[0].email.as_deref(),
        Some("hello@acme.example")
    );
    assert_eq!(organizations[0].phone.as_deref(), Some("+123456"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_contacts_csv_with_mapping_imports_nonstandard_headers() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("contacts-mapped.csv");
    std::fs::write(
        &csv_path,
        "Given,Family,Company,Mail,Tel,Street,Town,Nation,Memo,Skip\n\
         Mina,Okafor,Acme Health,mina@example.com,+234500,Dock 4,Lagos,NG,Primary buyer,ignored\n\
         ,Blank,Skipped,blank@example.com,,,,,,ignored\n",
    )
    .expect("mapped contact CSV fixture should write");
    let mapping = import_mapping(&[
        ("Given", Some("first_name")),
        ("Family", Some("last_name")),
        ("Company", Some("org_name")),
        ("Mail", Some("email")),
        ("Tel", Some("phone")),
        ("Street", Some("address")),
        ("Town", Some("city")),
        ("Nation", Some("country")),
        ("Memo", Some("notes")),
        ("Skip", None),
    ]);

    let result = core
        .import_contacts_csv_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped contact CSV import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());

    let imported: (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    ) = core
        .db
        .conn
        .query_row(
            "SELECT first_name, last_name, org_name, email, phone, address, city, country, notes FROM contacts WHERE email = ?1",
            params!["mina@example.com"],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            },
        )
        .expect("mapped contact should query");
    assert_eq!(
        imported,
        (
            "Mina".to_string(),
            "Okafor".to_string(),
            "Acme Health".to_string(),
            "mina@example.com".to_string(),
            "+234500".to_string(),
            "Dock 4".to_string(),
            "Lagos".to_string(),
            "NG".to_string(),
            "Primary buyer".to_string(),
        )
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_organizations_csv_with_mapping_imports_nonstandard_headers() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("organizations-mapped.csv");
    std::fs::write(
        &csv_path,
        "Company,Inbox,Telephone,Site,Line One,Line Two,Town,State,Nation,Postcode,About,Skip\n\
         Amani Labs,hello@amani.example,+254700,https://amani.example,Floor 2,Unit B,Nairobi,Nairobi County,KE,00100,Research partner,ignored\n\
         ,blank@example.com,,,,,,,,,,ignored\n",
    )
    .expect("mapped organization CSV fixture should write");
    let mapping = import_mapping(&[
        ("Company", Some("name")),
        ("Inbox", Some("email")),
        ("Telephone", Some("phone")),
        ("Site", Some("website")),
        ("Line One", Some("address_line1")),
        ("Line Two", Some("address_line2")),
        ("Town", Some("city")),
        ("State", Some("region")),
        ("Nation", Some("country")),
        ("Postcode", Some("postal_code")),
        ("About", Some("description")),
        ("Skip", None),
    ]);

    let result = core
        .import_organizations_csv_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped organization CSV import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());

    let organizations = core
        .list_organizations()
        .expect("organizations should list after mapped import");
    assert_eq!(organizations.len(), 1);
    assert_eq!(organizations[0].name, "Amani Labs");
    assert_eq!(
        organizations[0].email.as_deref(),
        Some("hello@amani.example")
    );
    assert_eq!(organizations[0].phone.as_deref(), Some("+254700"));
    assert_eq!(
        organizations[0].website.as_deref(),
        Some("https://amani.example")
    );
    assert_eq!(organizations[0].address_line1.as_deref(), Some("Floor 2"));
    assert_eq!(organizations[0].address_line2.as_deref(), Some("Unit B"));
    assert_eq!(organizations[0].city.as_deref(), Some("Nairobi"));
    assert_eq!(organizations[0].region.as_deref(), Some("Nairobi County"));
    assert_eq!(organizations[0].country.as_deref(), Some("KE"));
    assert_eq!(organizations[0].postal_code.as_deref(), Some("00100"));
    assert_eq!(
        organizations[0].description.as_deref(),
        Some("Research partner")
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_deals_csv_with_mapping_imports_nonstandard_headers() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("deals-mapped.csv");
    std::fs::write(
        &csv_path,
        "Deal Name,Amount,ISO,Phase,Close Date,Memo,Skip\n\
         Lagos Renewal,12500.50,EUR,Proposal,2026-09-30,Renewal path,ignored\n\
         ,2500,USD,Lead,2026-10-01,Blank title,ignored\n",
    )
    .expect("mapped deal CSV fixture should write");
    let mapping = import_mapping(&[
        ("Deal Name", Some("title")),
        ("Amount", Some("value")),
        ("ISO", Some("currency")),
        ("Phase", Some("stage")),
        ("Close Date", Some("expected_close")),
        ("Memo", Some("notes")),
        ("Skip", None),
    ]);

    let result = core
        .import_deals_csv_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped deal CSV import should succeed");

    assert_eq!(result.created, 1);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());

    let deals = core.list_deals().expect("deals should list after import");
    assert_eq!(deals.len(), 1);
    assert_eq!(deals[0].title, "Lagos Renewal");
    assert_eq!(deals[0].value, 12500.50);
    assert_eq!(deals[0].currency, "EUR");
    assert_eq!(deals[0].stage, "Proposal");
    assert_eq!(deals[0].expected_close.as_deref(), Some("2026-09-30"));
    assert_eq!(deals[0].notes, "Renewal path");
    assert_eq!(deals[0].contact_id, None);
    assert_eq!(deals[0].organization_id, None);

    let import_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE actor_type = 'import' AND action = 'import_row' AND entity_type = 'deal'",
            [],
            |row| row.get(0),
        )
        .expect("deal import audit count should query");
    assert_eq!(import_audit_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn preflight_deals_csv_import_flags_title_duplicates_without_writes() {
    let (mut core, path) = open_test_core();

    let deal = core
        .create_deal(
            "Acme Renewal".to_string(),
            Some(5000.0),
            Some("USD".to_string()),
            Some("Proposal".to_string()),
            Some(50),
            Some("2026-09-30".to_string()),
            None,
            None,
            Some("Existing renewal".to_string()),
        )
        .expect("deal fixture should be created");

    let deal_count_before = count(&core, "SELECT COUNT(*) FROM deals");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let csv_path = path.join("deals-preflight.csv");
    std::fs::write(
        &csv_path,
        "title,value,currency,stage,expected_close,notes\n\
         acme renewal,7500,USD,Negotiation,2026-10-15,Potential duplicate\n",
    )
    .expect("deal preflight CSV fixture should write");

    let report = core
        .preflight_deals_csv_import(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("deal preflight should succeed");

    assert_eq!(report.entity_type, "deals");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.duplicate_warning_count, 1);
    assert_eq!(report.warnings.len(), 1);

    let warning = &report.warnings[0];
    assert_eq!(warning.row_number, 2);
    assert_eq!(warning.match_type, "title");
    assert_eq!(warning.csv_value, "acme renewal");
    assert_eq!(warning.existing_entity_type, "deal");
    assert_eq!(warning.existing_entity_id, deal.id);
    assert_eq!(warning.existing_display_label, "Acme Renewal");

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM deals"),
        deal_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn mapped_organization_preflight_flags_name_email_and_phone_duplicates() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            Some("+123456".to_string()),
            None,
            None,
            None,
            Some("Lagos".to_string()),
            None,
            Some("NG".to_string()),
            None,
            Some("Regional partner".to_string()),
        )
        .expect("organization fixture should be created");

    let csv_path = path.join("organizations-mapped-preflight.csv");
    std::fs::write(
        &csv_path,
        "Company,Inbox,Telephone,Ignored\n\
         acme health,HELLO@acme.example,+123456,ignored\n",
    )
    .expect("mapped organization preflight CSV fixture should write");
    let mapping = import_mapping(&[
        ("Company", Some("name")),
        ("Inbox", Some("email")),
        ("Telephone", Some("phone")),
        ("Ignored", None),
    ]);

    let report = core
        .preflight_organizations_csv_import_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped organization preflight should succeed");

    assert_eq!(report.entity_type, "organizations");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.duplicate_warning_count, 3);
    for match_type in ["name", "email", "phone"] {
        let warning = report
            .warnings
            .iter()
            .find(|warning| warning.match_type == match_type)
            .unwrap_or_else(|| panic!("{match_type} duplicate warning should exist"));
        assert_eq!(warning.row_number, 2);
        assert_eq!(warning.existing_entity_type, "organization");
        assert_eq!(warning.existing_entity_id, organization.id);
        assert_eq!(warning.existing_display_label, "Acme Health");
    }

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn mapped_import_rejects_invalid_target_fields_and_duplicate_assignments() {
    let (core, path) = open_test_core();
    let csv_path = path.join("invalid-mapping.csv");
    std::fs::write(&csv_path, "A,B,C\none,two,three\n")
        .expect("invalid mapping CSV fixture should write");

    let invalid_target = import_mapping(&[("A", Some("nickname"))]);
    let err = core
        .preflight_contacts_csv_import_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            invalid_target,
        )
        .expect_err("unknown mapped target should be rejected");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("Unknown import target field 'nickname'"));
        }
        other => panic!("expected InvalidInput for unknown target, got {other:?}"),
    }

    let duplicate_target = import_mapping(&[("A", Some("first_name")), ("B", Some("first_name"))]);
    let err = core
        .preflight_contacts_csv_import_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            duplicate_target,
        )
        .expect_err("duplicate mapped target should be rejected");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("mapped more than once"));
            assert!(message.contains("first_name"));
        }
        other => panic!("expected InvalidInput for duplicate target, got {other:?}"),
    }

    let json_path = path.join("invalid-mapping.json");
    std::fs::write(&json_path, r#"[{ "A": "one", "B": "two" }]"#)
        .expect("invalid mapping JSON fixture should write");

    let missing_source = import_mapping(&[("Missing", Some("first_name"))]);
    let err = core
        .preflight_contacts_json_import_with_mapping(
            json_path.to_str().expect("path should be valid UTF-8"),
            missing_source,
        )
        .expect_err("missing JSON source field should be rejected");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("Mapped source field 'Missing' is not present in the JSON"));
        }
        other => panic!("expected InvalidInput for missing JSON source, got {other:?}"),
    }

    let duplicate_json_target =
        import_mapping(&[("A", Some("first_name")), ("B", Some("first_name"))]);
    let err = core
        .preflight_contacts_json_import_with_mapping(
            json_path.to_str().expect("path should be valid UTF-8"),
            duplicate_json_target,
        )
        .expect_err("duplicate mapped JSON target should be rejected");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("mapped more than once"));
            assert!(message.contains("first_name"));
        }
        other => panic!("expected InvalidInput for duplicate JSON target, got {other:?}"),
    }

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn mapped_contact_preflight_is_read_only() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            Some("+15550100".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact fixture should be created");

    let contact_count_before = count(&core, "SELECT COUNT(*) FROM contacts");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let csv_path = path.join("contacts-mapped-preflight.csv");
    std::fs::write(
        &csv_path,
        "Given,Mail,Telephone,Ignored\n\
         Imported,ADA@example.com,+15550100,ignored\n",
    )
    .expect("mapped contact preflight CSV fixture should write");
    let mapping = import_mapping(&[
        ("Given", Some("first_name")),
        ("Mail", Some("email")),
        ("Telephone", Some("phone")),
        ("Ignored", None),
    ]);

    let report = core
        .preflight_contacts_csv_import_with_mapping(
            csv_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped contact preflight should succeed");

    assert_eq!(report.entity_type, "contacts");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.duplicate_warning_count, 2);
    assert!(report
        .warnings
        .iter()
        .all(|warning| warning.existing_entity_id == contact.id));
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM contacts"),
        contact_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn mapped_contact_json_preflight_uses_mapped_values_and_is_read_only() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            Some("+15550100".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact fixture should be created");

    let contact_count_before = count(&core, "SELECT COUNT(*) FROM contacts");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let json_path = path.join("contacts-mapped-preflight.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "Given": "Imported",
    "Mail": "ADA@example.com",
    "Telephone": "  +15550100  ",
    "Ignored": "ignored"
  }
]
"#,
    )
    .expect("mapped contact JSON preflight fixture should write");
    let mapping = import_mapping(&[
        ("Given", Some("first_name")),
        ("Mail", Some("email")),
        ("Telephone", Some("phone")),
        ("Ignored", None),
    ]);

    let report = core
        .preflight_contacts_json_import_with_mapping(
            json_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped contact JSON preflight should succeed");

    assert_eq!(report.entity_type, "contacts");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.duplicate_warning_count, 2);
    assert!(report
        .warnings
        .iter()
        .all(|warning| warning.existing_entity_id == contact.id));
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM contacts"),
        contact_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn preflight_contacts_csv_import_flags_email_and_phone_duplicates_without_writes() {
    let (mut core, path) = open_test_core();

    let email_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("email fixture contact should be created");
    let phone_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Grace".to_string()),
            Some("Hopper".to_string()),
            None,
            None,
            Some("+15550100".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("phone fixture contact should be created");

    let contact_count_before = count(&core, "SELECT COUNT(*) FROM contacts");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let csv_path = path.join("contacts-preflight.csv");
    std::fs::write(
        &csv_path,
        "first_name,last_name,email,phone\n\
         Imported,Email,ADA@example.com,\n\
         Imported,Phone,,+15550100\n",
    )
    .expect("contact preflight CSV fixture should write");

    let report = core
        .preflight_contacts_csv_import(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("contact preflight should succeed");

    assert_eq!(report.entity_type, "contacts");
    assert_eq!(report.total_rows, 2);
    assert_eq!(report.duplicate_warning_count, 2);
    assert_eq!(report.warnings.len(), 2);

    let email_warning = report
        .warnings
        .iter()
        .find(|warning| warning.match_type == "email")
        .expect("email duplicate warning should exist");
    assert_eq!(email_warning.row_number, 2);
    assert_eq!(email_warning.csv_value, "ADA@example.com");
    assert_eq!(email_warning.existing_entity_type, "contact");
    assert_eq!(email_warning.existing_entity_id, email_contact.id);
    assert_eq!(email_warning.existing_display_label, "Ada Lovelace");

    let phone_warning = report
        .warnings
        .iter()
        .find(|warning| warning.match_type == "phone")
        .expect("phone duplicate warning should exist");
    assert_eq!(phone_warning.row_number, 3);
    assert_eq!(phone_warning.csv_value, "+15550100");
    assert_eq!(phone_warning.existing_entity_type, "contact");
    assert_eq!(phone_warning.existing_entity_id, phone_contact.id);
    assert_eq!(phone_warning.existing_display_label, "Grace Hopper");

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM contacts"),
        contact_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn preflight_contacts_json_import_flags_email_and_phone_duplicates_without_writes() {
    let (mut core, path) = open_test_core();

    let email_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("email fixture contact should be created");
    let phone_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Grace".to_string()),
            Some("Hopper".to_string()),
            None,
            None,
            Some("+15550100".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("phone fixture contact should be created");

    let contact_count_before = count(&core, "SELECT COUNT(*) FROM contacts");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let json_path = path.join("contacts-preflight.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "first_name": "Imported",
    "last_name": "Email",
    "email": "ADA@example.com"
  },
  {
    "first_name": "Imported",
    "last_name": "Phone",
    "phone": "  +15550100  "
  }
]
"#,
    )
    .expect("contact preflight JSON fixture should write");

    let report = core
        .preflight_contacts_json_import(json_path.to_str().expect("path should be valid UTF-8"))
        .expect("contact JSON preflight should succeed");

    assert_eq!(report.entity_type, "contacts");
    assert_eq!(report.total_rows, 2);
    assert_eq!(report.duplicate_warning_count, 2);
    assert_eq!(report.warnings.len(), 2);

    let email_warning = report
        .warnings
        .iter()
        .find(|warning| warning.match_type == "email")
        .expect("email duplicate warning should exist");
    assert_eq!(email_warning.row_number, 2);
    assert_eq!(email_warning.csv_value, "ADA@example.com");
    assert_eq!(email_warning.existing_entity_type, "contact");
    assert_eq!(email_warning.existing_entity_id, email_contact.id);
    assert_eq!(email_warning.existing_display_label, "Ada Lovelace");

    let phone_warning = report
        .warnings
        .iter()
        .find(|warning| warning.match_type == "phone")
        .expect("phone duplicate warning should exist");
    assert_eq!(phone_warning.row_number, 3);
    assert_eq!(phone_warning.csv_value, "+15550100");
    assert_eq!(phone_warning.existing_entity_type, "contact");
    assert_eq!(phone_warning.existing_entity_id, phone_contact.id);
    assert_eq!(phone_warning.existing_display_label, "Grace Hopper");

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM contacts"),
        contact_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn preflight_organizations_csv_import_flags_name_email_and_phone_duplicates_without_writes() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            Some("+123456".to_string()),
            None,
            None,
            None,
            Some("Lagos".to_string()),
            None,
            Some("NG".to_string()),
            None,
            Some("Regional partner".to_string()),
        )
        .expect("organization fixture should be created");

    let organization_count_before = count(&core, "SELECT COUNT(*) FROM organizations");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let csv_path = path.join("organizations-preflight.csv");
    std::fs::write(
        &csv_path,
        "name,email,phone,website,address_line1,address_line2,city,region,country,postal_code,description\n\
         acme health,HELLO@acme.example,+123456,,,,,,,,\n",
    )
    .expect("organization preflight CSV fixture should write");

    let report = core
        .preflight_organizations_csv_import(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("organization preflight should succeed");

    assert_eq!(report.entity_type, "organizations");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.duplicate_warning_count, 3);
    assert_eq!(report.warnings.len(), 3);

    for match_type in ["name", "email", "phone"] {
        let warning = report
            .warnings
            .iter()
            .find(|warning| warning.match_type == match_type)
            .unwrap_or_else(|| panic!("{match_type} duplicate warning should exist"));
        assert_eq!(warning.row_number, 2);
        assert_eq!(warning.existing_entity_type, "organization");
        assert_eq!(warning.existing_entity_id, organization.id);
        assert_eq!(warning.existing_display_label, "Acme Health");
    }

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM organizations"),
        organization_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn preflight_organizations_json_import_flags_name_email_and_phone_duplicates_without_writes() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            Some("+123456".to_string()),
            None,
            None,
            None,
            Some("Lagos".to_string()),
            None,
            Some("NG".to_string()),
            None,
            Some("Regional partner".to_string()),
        )
        .expect("organization fixture should be created");

    let organization_count_before = count(&core, "SELECT COUNT(*) FROM organizations");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let json_path = path.join("organizations-preflight.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "name": " acme health ",
    "email": "HELLO@acme.example",
    "phone": "  +123456  "
  }
]
"#,
    )
    .expect("organization preflight JSON fixture should write");

    let report = core
        .preflight_organizations_json_import(
            json_path.to_str().expect("path should be valid UTF-8"),
        )
        .expect("organization JSON preflight should succeed");

    assert_eq!(report.entity_type, "organizations");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.duplicate_warning_count, 3);
    assert_eq!(report.warnings.len(), 3);

    for match_type in ["name", "email", "phone"] {
        let warning = report
            .warnings
            .iter()
            .find(|warning| warning.match_type == match_type)
            .unwrap_or_else(|| panic!("{match_type} duplicate warning should exist"));
        assert_eq!(warning.row_number, 2);
        assert_eq!(warning.existing_entity_type, "organization");
        assert_eq!(warning.existing_entity_id, organization.id);
        assert_eq!(warning.existing_display_label, "Acme Health");
    }

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM organizations"),
        organization_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn preflight_deals_json_import_flags_title_duplicates_without_writes() {
    let (mut core, path) = open_test_core();

    let deal = core
        .create_deal(
            "Acme Renewal".to_string(),
            Some(5000.0),
            Some("USD".to_string()),
            Some("Proposal".to_string()),
            Some(50),
            Some("2026-09-30".to_string()),
            None,
            None,
            Some("Existing renewal".to_string()),
        )
        .expect("deal fixture should be created");

    let deal_count_before = count(&core, "SELECT COUNT(*) FROM deals");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let json_path = path.join("deals-preflight.json");
    std::fs::write(
        &json_path,
        r#"[
  {
    "title": " acme renewal ",
    "value": "7500",
    "currency": "USD",
    "stage": "Negotiation",
    "expected_close": "2026-10-15",
    "notes": "Potential duplicate"
  }
]
"#,
    )
    .expect("deal preflight JSON fixture should write");

    let report = core
        .preflight_deals_json_import(json_path.to_str().expect("path should be valid UTF-8"))
        .expect("deal JSON preflight should succeed");

    assert_eq!(report.entity_type, "deals");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.duplicate_warning_count, 1);
    assert_eq!(report.warnings.len(), 1);

    let warning = &report.warnings[0];
    assert_eq!(warning.row_number, 2);
    assert_eq!(warning.match_type, "title");
    assert_eq!(warning.csv_value, "acme renewal");
    assert_eq!(warning.existing_entity_type, "deal");
    assert_eq!(warning.existing_entity_id, deal.id);
    assert_eq!(warning.existing_display_label, "Acme Renewal");

    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM deals"),
        deal_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn export_contacts_json_writes_active_flat_rows() {
    let (mut core, path) = open_test_core();

    core.create_contact(
        Some("person".to_string()),
        Some("Ada".to_string()),
        Some("Lovelace".to_string()),
        Some("Analytical Engines".to_string()),
        Some("ada@example.com".to_string()),
        Some("+15550123".to_string()),
        Some("1 Example Street".to_string()),
        Some("London".to_string()),
        Some("UK".to_string()),
        None,
        Some("Prefers email".to_string()),
    )
    .expect("active contact should be created");
    let deleted = core
        .create_contact(
            Some("person".to_string()),
            Some("Deleted".to_string()),
            Some("Contact".to_string()),
            None,
            Some("deleted@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("deleted contact should be created");
    core.delete_contact(&deleted.id)
        .expect("deleted contact should be soft-deleted");

    let json_path = path.join("contacts-export.json");
    let count = core
        .export_contacts_json(json_path.to_str().expect("path should be valid UTF-8"))
        .expect("contact JSON export should succeed");

    assert_eq!(count, 1);
    let rows = read_json_export(&json_path);
    assert_eq!(rows.len(), 1);
    let row = rows[0]
        .as_object()
        .expect("contact row should be an object");
    assert_eq!(
        row.get("first_name").and_then(|value| value.as_str()),
        Some("Ada")
    );
    assert_eq!(
        row.get("last_name").and_then(|value| value.as_str()),
        Some("Lovelace")
    );
    assert_eq!(
        row.get("org_name").and_then(|value| value.as_str()),
        Some("Analytical Engines")
    );
    assert_eq!(
        row.get("email").and_then(|value| value.as_str()),
        Some("ada@example.com")
    );
    assert_eq!(
        row.get("notes").and_then(|value| value.as_str()),
        Some("Prefers email")
    );
    assert!(row.get("id").is_none());
    assert!(row.get("deleted_at").is_none());
    assert!(!rows
        .iter()
        .any(|row| row.get("first_name").and_then(|value| value.as_str()) == Some("Deleted")));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn export_deals_json_writes_active_flat_rows() {
    let (mut core, path) = open_test_core();

    core.create_deal(
        "Acme Renewal".to_string(),
        Some(12500.0),
        Some("EUR".to_string()),
        Some("Proposal".to_string()),
        Some(55),
        Some("2026-10-01".to_string()),
        None,
        None,
        Some("Includes onboarding".to_string()),
    )
    .expect("active deal should be created");
    let deleted = core
        .create_deal(
            "Deleted Renewal".to_string(),
            Some(2500.0),
            Some("USD".to_string()),
            Some("Lead".to_string()),
            Some(10),
            None,
            None,
            None,
            None,
        )
        .expect("deleted deal should be created");
    core.delete_deal(&deleted.id)
        .expect("deleted deal should be soft-deleted");

    let json_path = path.join("deals-export.json");
    let count = core
        .export_deals_json(json_path.to_str().expect("path should be valid UTF-8"))
        .expect("deal JSON export should succeed");

    assert_eq!(count, 1);
    let rows = read_json_export(&json_path);
    assert_eq!(rows.len(), 1);
    let row = rows[0].as_object().expect("deal row should be an object");
    assert_eq!(
        row.get("title").and_then(|value| value.as_str()),
        Some("Acme Renewal")
    );
    assert_eq!(
        row.get("value").and_then(|value| value.as_str()),
        Some("12500.00")
    );
    assert_eq!(
        row.get("currency").and_then(|value| value.as_str()),
        Some("EUR")
    );
    assert_eq!(
        row.get("stage").and_then(|value| value.as_str()),
        Some("Proposal")
    );
    assert_eq!(
        row.get("expected_close").and_then(|value| value.as_str()),
        Some("2026-10-01")
    );
    assert_eq!(
        row.get("notes").and_then(|value| value.as_str()),
        Some("Includes onboarding")
    );
    assert!(row.get("id").is_none());
    assert!(row.get("probability").is_none());
    assert!(row.get("deleted_at").is_none());
    assert!(!rows
        .iter()
        .any(|row| row.get("title").and_then(|value| value.as_str()) == Some("Deleted Renewal")));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn export_contacts_and_deals_include_custom_field_values() {
    let (mut core, path) = open_test_core();

    let contact_field = core
        .create_custom_field_def(
            "contact".to_string(),
            "VIP Tier".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("contact custom field should be created");
    let escaped_contact_field = core
        .create_custom_field_def(
            "contact".to_string(),
            "Plan #".to_string(),
            "text".to_string(),
            None,
            Some(1),
        )
        .expect("escaped-name contact custom field should be created");
    let deal_field = core
        .create_custom_field_def(
            "deal".to_string(),
            "Risk Band".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("deal custom field should be created");
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
    let deal = core
        .create_deal(
            "Acme Renewal".to_string(),
            Some(12500.0),
            Some("EUR".to_string()),
            Some("Proposal".to_string()),
            Some(55),
            None,
            None,
            None,
            None,
        )
        .expect("deal should be created");
    core.set_custom_field_value(
        contact_field.id.clone(),
        contact.id.clone(),
        "Gold".to_string(),
    )
    .expect("contact custom value should be set");
    core.set_custom_field_value(
        escaped_contact_field.id.clone(),
        contact.id.clone(),
        "Enterprise".to_string(),
    )
    .expect("escaped-name contact custom value should be set");
    core.set_custom_field_value(deal_field.id.clone(), deal.id.clone(), "Medium".to_string())
        .expect("deal custom value should be set");

    let contacts_json_path = path.join("contacts-custom-export.json");
    let contacts_csv_path = path.join("contacts-custom-export.csv");
    let deals_json_path = path.join("deals-custom-export.json");
    let deals_csv_path = path.join("deals-custom-export.csv");

    assert_eq!(
        core.export_contacts_json(
            contacts_json_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("contact JSON export should succeed"),
        1
    );
    assert_eq!(
        core.export_contacts_csv(
            contacts_csv_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("contact CSV export should succeed"),
        1
    );
    assert_eq!(
        core.export_deals_json(
            deals_json_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("deal JSON export should succeed"),
        1
    );
    assert_eq!(
        core.export_deals_csv(deals_csv_path.to_str().expect("path should be valid UTF-8"))
            .expect("deal CSV export should succeed"),
        1
    );

    let contact_json_rows = read_json_export(&contacts_json_path);
    assert_eq!(
        contact_json_rows[0]
            .get("custom:VIP Tier")
            .and_then(|value| value.as_str()),
        Some("Gold")
    );
    assert_eq!(
        contact_json_rows[0]
            .get("custom:Plan %23")
            .and_then(|value| value.as_str()),
        Some("Enterprise")
    );
    let contact_csv_rows = read_csv_export(&contacts_csv_path);
    assert_eq!(
        contact_csv_rows[0]
            .get("custom:VIP Tier")
            .map(String::as_str),
        Some("Gold")
    );
    assert_eq!(
        contact_csv_rows[0]
            .get("custom:Plan %23")
            .map(String::as_str),
        Some("Enterprise")
    );

    let deal_json_rows = read_json_export(&deals_json_path);
    assert_eq!(
        deal_json_rows[0]
            .get("custom:Risk Band")
            .and_then(|value| value.as_str()),
        Some("Medium")
    );
    let deal_csv_rows = read_csv_export(&deals_csv_path);
    assert_eq!(
        deal_csv_rows[0].get("custom:Risk Band").map(String::as_str),
        Some("Medium")
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn duplicate_custom_field_names_use_deterministic_targets() {
    let (mut core, path) = open_test_core();

    let primary_field = core
        .create_custom_field_def(
            "contact".to_string(),
            "Region".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("primary contact custom field should be created");
    let secondary_field = core
        .create_custom_field_def(
            "contact".to_string(),
            "Region".to_string(),
            "text".to_string(),
            None,
            Some(1),
        )
        .expect("secondary contact custom field should be created");
    let primary_target = format!("custom:Region#{}", primary_field.id);
    let secondary_target = format!("custom:Region#{}", secondary_field.id);
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            None,
            None,
            Some("ada@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");

    core.set_custom_field_value(
        primary_field.id.clone(),
        contact.id.clone(),
        "North".to_string(),
    )
    .expect("primary custom value should be set");
    core.set_custom_field_value(
        secondary_field.id.clone(),
        contact.id.clone(),
        "West".to_string(),
    )
    .expect("secondary custom value should be set");

    let contacts_json_path = path.join("contacts-duplicate-custom-export.json");
    core.export_contacts_json(
        contacts_json_path
            .to_str()
            .expect("path should be valid UTF-8"),
    )
    .expect("contact JSON export should succeed");
    let contact_json_rows = read_json_export(&contacts_json_path);
    assert!(contact_json_rows[0].get("custom:Region").is_none());
    assert_eq!(
        contact_json_rows[0]
            .get(&primary_target)
            .and_then(|value| value.as_str()),
        Some("North")
    );
    assert_eq!(
        contact_json_rows[0]
            .get(&secondary_target)
            .and_then(|value| value.as_str()),
        Some("West")
    );

    let contacts_csv_path = path.join("contacts-duplicate-custom-import.csv");
    std::fs::write(
        &contacts_csv_path,
        format!(
            "first_name,email,{primary_target},{secondary_target}\nGrace,grace@example.com,East,South\n"
        ),
    )
    .expect("contact custom CSV fixture should write");
    let import_result = core
        .import_contacts_csv(
            contacts_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("contact custom CSV import should succeed");
    assert_eq!(import_result.created, 1);
    let imported_contact = core
        .list_contacts(None)
        .expect("contacts should list")
        .contacts
        .into_iter()
        .find(|contact| contact.email == "grace@example.com")
        .expect("imported contact should exist");
    let values: std::collections::BTreeMap<_, _> = core
        .list_custom_field_values("contact", &imported_contact.id)
        .expect("contact custom values should list")
        .into_iter()
        .map(|value| (value.field_def_id, value.value))
        .collect();
    assert_eq!(
        values.get(&primary_field.id).map(String::as_str),
        Some("East")
    );
    assert_eq!(
        values.get(&secondary_field.id).map(String::as_str),
        Some("South")
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_contacts_csv_and_mapped_deal_json_set_custom_field_values() {
    let (mut core, path) = open_test_core();

    let contact_field = core
        .create_custom_field_def(
            "contact".to_string(),
            "VIP Tier".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("contact custom field should be created");
    let deal_field = core
        .create_custom_field_def(
            "deal".to_string(),
            "Risk Band".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("deal custom field should be created");

    let contacts_csv_path = path.join("contacts-custom-import.csv");
    std::fs::write(
        &contacts_csv_path,
        "first_name,email,custom:VIP Tier\nAda,ada@example.com,Gold\n",
    )
    .expect("contact custom CSV fixture should write");
    let contact_result = core
        .import_contacts_csv(
            contacts_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("contact custom CSV import should succeed");
    assert_eq!(contact_result.created, 1);
    let contacts = core
        .list_contacts(None)
        .expect("contacts should list")
        .contacts;
    let contact_values = core
        .list_custom_field_values("contact", &contacts[0].id)
        .expect("contact custom values should list");
    assert_eq!(contact_values.len(), 1);
    assert_eq!(contact_values[0].field_def_id, contact_field.id);
    assert_eq!(contact_values[0].value, "Gold");

    let deals_json_path = path.join("deals-custom-import.json");
    std::fs::write(
        &deals_json_path,
        r#"[{"Opportunity":"Acme Renewal","Risk":"Medium"}]"#,
    )
    .expect("deal custom JSON fixture should write");
    let deal_result = core
        .import_deals_json_with_mapping(
            deals_json_path
                .to_str()
                .expect("path should be valid UTF-8"),
            import_mapping(&[
                ("Opportunity", Some("title")),
                ("Risk", Some("custom:Risk Band")),
            ]),
        )
        .expect("mapped deal custom JSON import should succeed");
    assert_eq!(deal_result.created, 1);
    let deals = core.list_deals().expect("deals should list");
    let deal_values = core
        .list_custom_field_values("deal", &deals[0].id)
        .expect("deal custom values should list");
    assert_eq!(deal_values.len(), 1);
    assert_eq!(deal_values[0].field_def_id, deal_field.id);
    assert_eq!(deal_values[0].value, "Medium");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn duplicate_auto_merge_fills_missing_custom_values_without_overwriting() {
    let (mut core, path) = open_test_core();

    let contact_field = core
        .create_custom_field_def(
            "contact".to_string(),
            "VIP Tier".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("contact custom field should be created");
    let deal_field = core
        .create_custom_field_def(
            "deal".to_string(),
            "Risk Band".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("deal custom field should be created");
    let missing_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            None,
            None,
            Some("ada@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("missing custom contact should be created");
    let existing_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Grace".to_string()),
            None,
            None,
            Some("grace@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("existing custom contact should be created");
    core.set_custom_field_value(
        contact_field.id.clone(),
        existing_contact.id.clone(),
        "Platinum".to_string(),
    )
    .expect("existing contact custom value should be set");

    let missing_deal = core
        .create_deal(
            "Acme Renewal".to_string(),
            Some(0.0),
            Some("USD".to_string()),
            Some("Lead".to_string()),
            Some(0),
            None,
            None,
            None,
            None,
        )
        .expect("missing custom deal should be created");
    let existing_deal = core
        .create_deal(
            "Enterprise Expansion".to_string(),
            Some(0.0),
            Some("USD".to_string()),
            Some("Lead".to_string()),
            Some(0),
            None,
            None,
            None,
            None,
        )
        .expect("existing custom deal should be created");
    core.set_custom_field_value(
        deal_field.id.clone(),
        existing_deal.id.clone(),
        "Low".to_string(),
    )
    .expect("existing deal custom value should be set");

    let contacts_csv_path = path.join("contacts-custom-auto-merge.csv");
    std::fs::write(
        &contacts_csv_path,
        "first_name,email,custom:VIP Tier\nAda,ada@example.com,Gold\nGrace,grace@example.com,Silver\n",
    )
    .expect("contact custom auto-merge CSV fixture should write");
    let contact_result = core
        .import_contacts_csv_with_options(
            contacts_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("contact custom auto-merge should succeed");
    assert_eq!(contact_result.merged, 2);
    assert_eq!(
        core.list_custom_field_values("contact", &missing_contact.id)
            .expect("missing contact custom value should list")[0]
            .value,
        "Gold"
    );
    assert_eq!(
        core.list_custom_field_values("contact", &existing_contact.id)
            .expect("existing contact custom value should list")[0]
            .value,
        "Platinum"
    );

    let deals_csv_path = path.join("deals-custom-auto-merge.csv");
    std::fs::write(
        &deals_csv_path,
        "title,custom:Risk Band\nAcme Renewal,Medium\nEnterprise Expansion,High\n",
    )
    .expect("deal custom auto-merge CSV fixture should write");
    let deal_result = core
        .import_deals_csv_with_options(
            deals_csv_path.to_str().expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("deal custom auto-merge should succeed");
    assert_eq!(deal_result.merged, 2);
    assert_eq!(
        core.list_custom_field_values("deal", &missing_deal.id)
            .expect("missing deal custom value should list")[0]
            .value,
        "Medium"
    );
    assert_eq!(
        core.list_custom_field_values("deal", &existing_deal.id)
            .expect("existing deal custom value should list")[0]
            .value,
        "Low"
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_rollback_restores_custom_field_value_changes() {
    let (mut core, path) = open_test_core();

    let contact_field = core
        .create_custom_field_def(
            "contact".to_string(),
            "VIP Tier".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("contact custom field should be created");
    let deal_field = core
        .create_custom_field_def(
            "deal".to_string(),
            "Risk Band".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("deal custom field should be created");

    let contacts_csv_path = path.join("contacts-custom-rollback.csv");
    std::fs::write(
        &contacts_csv_path,
        "first_name,email,custom:VIP Tier\nAda,ada@example.com,Gold\n",
    )
    .expect("contact rollback CSV fixture should write");
    let contact_result = core
        .import_contacts_csv(
            contacts_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("contact custom import should succeed");
    let contact_plan = contact_result
        .rollback_plan
        .clone()
        .expect("created contact custom import should return rollback plan");
    let imported_contact = core
        .list_contacts(None)
        .expect("contacts should list")
        .contacts
        .into_iter()
        .find(|contact| contact.email == "ada@example.com")
        .expect("imported contact should exist");
    let imported_contact_values = core
        .list_custom_field_values("contact", &imported_contact.id)
        .expect("contact custom value should list");
    assert_eq!(imported_contact_values[0].field_def_id, contact_field.id);
    let imported_contact_value_id = imported_contact_values[0].value_id.clone();
    let created_contact_delete_audit_before =
        count_custom_field_audit_action(&core, &imported_contact_value_id, "delete_value");
    let created_contact_delete_sync_before =
        count_custom_field_delete_sync(&core, &imported_contact_value_id, "Gold");
    let contact_rollback = core
        .rollback_completed_import(&contact_plan)
        .expect("created contact custom rollback should succeed");
    assert_eq!(contact_rollback.rolled_back, 1);
    assert!(contact_rollback.errors.is_empty());
    assert!(core
        .list_custom_field_values("contact", &imported_contact.id)
        .expect("rolled back contact custom values should list")
        .is_empty());
    assert_eq!(
        count_custom_field_audit_action(&core, &imported_contact_value_id, "delete_value"),
        created_contact_delete_audit_before + 1
    );
    assert_eq!(
        count_custom_field_delete_sync(&core, &imported_contact_value_id, "Gold"),
        created_contact_delete_sync_before + 1
    );

    let blank_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Grace".to_string()),
            None,
            None,
            Some("grace@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("blank custom contact should be created");
    let blank_contact_value = core
        .set_custom_field_value(
            contact_field.id.clone(),
            blank_contact.id.clone(),
            "".to_string(),
        )
        .expect("blank contact custom value should be set");
    let blank_contacts_csv_path = path.join("contacts-custom-blank-merge-rollback.csv");
    std::fs::write(
        &blank_contacts_csv_path,
        "first_name,email,custom:VIP Tier\nGrace,grace@example.com,Silver\n",
    )
    .expect("blank contact rollback CSV fixture should write");
    let blank_contact_result = core
        .import_contacts_csv_with_options(
            blank_contacts_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("blank contact custom merge import should succeed");
    let blank_contact_plan = blank_contact_result
        .rollback_plan
        .clone()
        .expect("blank contact custom merge import should return rollback plan");
    assert_eq!(
        core.list_custom_field_values("contact", &blank_contact.id)
            .expect("filled contact custom value should list")[0]
            .value,
        "Silver"
    );
    let blank_contact_set_audit_before =
        count_custom_field_audit_action(&core, &blank_contact_value.id, "set_value");
    let blank_contact_set_sync_before =
        count_custom_field_set_sync(&core, &blank_contact_value.id, "");
    let blank_contact_rollback = core
        .rollback_completed_import(&blank_contact_plan)
        .expect("blank contact custom merge rollback should succeed");
    assert_eq!(blank_contact_rollback.rolled_back, 1);
    assert!(blank_contact_rollback.errors.is_empty());
    assert_eq!(
        core.list_custom_field_values("contact", &blank_contact.id)
            .expect("restored blank contact custom value should list")[0]
            .value,
        ""
    );
    assert_eq!(
        count_custom_field_audit_action(&core, &blank_contact_value.id, "set_value"),
        blank_contact_set_audit_before + 1
    );
    assert_eq!(
        count_custom_field_set_sync(&core, &blank_contact_value.id, ""),
        blank_contact_set_sync_before + 1
    );

    let existing_deal = core
        .create_deal(
            "Acme Renewal".to_string(),
            Some(0.0),
            Some("USD".to_string()),
            Some("Lead".to_string()),
            Some(0),
            None,
            None,
            None,
            None,
        )
        .expect("existing deal should be created");
    let deals_csv_path = path.join("deals-custom-merge-rollback.csv");
    std::fs::write(
        &deals_csv_path,
        "title,custom:Risk Band\nAcme Renewal,Medium\n",
    )
    .expect("deal rollback CSV fixture should write");
    let deal_result = core
        .import_deals_csv_with_options(
            deals_csv_path.to_str().expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("deal custom merge import should succeed");
    let deal_plan = deal_result
        .rollback_plan
        .clone()
        .expect("merged deal custom import should return rollback plan");
    let existing_deal_values = core
        .list_custom_field_values("deal", &existing_deal.id)
        .expect("deal custom value should list");
    assert_eq!(existing_deal_values[0].field_def_id, deal_field.id);
    let existing_deal_value_id = existing_deal_values[0].value_id.clone();
    let deal_delete_audit_before =
        count_custom_field_audit_action(&core, &existing_deal_value_id, "delete_value");
    let deal_delete_sync_before =
        count_custom_field_delete_sync(&core, &existing_deal_value_id, "Medium");
    let deal_rollback = core
        .rollback_completed_import(&deal_plan)
        .expect("merged deal custom rollback should succeed");
    assert_eq!(deal_rollback.rolled_back, 1);
    assert!(deal_rollback.errors.is_empty());
    assert!(core
        .list_custom_field_values("deal", &existing_deal.id)
        .expect("rolled back deal custom values should list")
        .is_empty());
    assert_eq!(
        count_custom_field_audit_action(&core, &existing_deal_value_id, "delete_value"),
        deal_delete_audit_before + 1
    );
    assert_eq!(
        count_custom_field_delete_sync(&core, &existing_deal_value_id, "Medium"),
        deal_delete_sync_before + 1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn export_activities_csv_and_json_include_flat_fields_and_custom_values() {
    let (mut core, path) = open_test_core();

    let activity_field = core
        .create_custom_field_def(
            "activity".to_string(),
            "Outcome".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("activity custom field should be created");
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            Some("amina@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
    let deal = core
        .create_deal(
            "Amina Renewal".to_string(),
            Some(2500.0),
            Some("USD".to_string()),
            Some("Lead".to_string()),
            Some(20),
            None,
            Some(contact.id.clone()),
            None,
            None,
        )
        .expect("deal should be created");
    let activity = core
        .create_activity(
            "call".to_string(),
            "Discovery call".to_string(),
            Some("Discuss renewal scope".to_string()),
            Some("2026-07-01T10:00:00Z".to_string()),
            Some(contact.id.clone()),
            Some(deal.id.clone()),
        )
        .expect("activity should be created");
    let activity = core
        .mark_activity_complete(&activity.id)
        .expect("activity should be marked complete");
    core.set_custom_field_value(
        activity_field.id.clone(),
        activity.id.clone(),
        "Positive".to_string(),
    )
    .expect("activity custom value should be set");

    let activities_json_path = path.join("activities-custom-export.json");
    let activities_csv_path = path.join("activities-custom-export.csv");
    assert_eq!(
        core.export_activities_json(
            activities_json_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("activity JSON export should succeed"),
        1
    );
    assert_eq!(
        core.export_activities_csv(
            activities_csv_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("activity CSV export should succeed"),
        1
    );

    let json_rows = read_json_export(&activities_json_path);
    assert_eq!(
        json_rows[0]
            .get("activity_type")
            .and_then(|value| value.as_str()),
        Some("call")
    );
    assert_eq!(
        json_rows[0].get("title").and_then(|value| value.as_str()),
        Some("Discovery call")
    );
    assert_eq!(
        json_rows[0]
            .get("description")
            .and_then(|value| value.as_str()),
        Some("Discuss renewal scope")
    );
    assert_eq!(
        json_rows[0]
            .get("completed")
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        json_rows[0]
            .get("contact_id")
            .and_then(|value| value.as_str()),
        Some(contact.id.as_str())
    );
    assert_eq!(
        json_rows[0].get("deal_id").and_then(|value| value.as_str()),
        Some(deal.id.as_str())
    );
    assert_eq!(
        json_rows[0]
            .get("custom:Outcome")
            .and_then(|value| value.as_str()),
        Some("Positive")
    );

    let csv_rows = read_csv_export(&activities_csv_path);
    assert_eq!(
        csv_rows[0].get("activity_type").map(String::as_str),
        Some("call")
    );
    assert_eq!(
        csv_rows[0].get("title").map(String::as_str),
        Some("Discovery call")
    );
    assert_eq!(
        csv_rows[0].get("completed").map(String::as_str),
        Some("true")
    );
    assert_eq!(
        csv_rows[0].get("contact_id").map(String::as_str),
        Some(contact.id.as_str())
    );
    assert_eq!(
        csv_rows[0].get("deal_id").map(String::as_str),
        Some(deal.id.as_str())
    );
    assert_eq!(
        csv_rows[0].get("custom:Outcome").map(String::as_str),
        Some("Positive")
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_activities_csv_and_json_create_rows_and_links() {
    let (mut core, path) = open_test_core();
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
    let deal = core
        .create_deal(
            "Ada Expansion".to_string(),
            Some(5000.0),
            Some("EUR".to_string()),
            Some("Proposal".to_string()),
            Some(60),
            None,
            Some(contact.id.clone()),
            None,
            None,
        )
        .expect("deal should be created");

    let activities_csv_path = path.join("activities-import.csv");
    std::fs::write(
        &activities_csv_path,
        format!(
            "activity_type,title,description,due_date,completed,contact_id,deal_id\n\
             task,Follow up,Send recap,2026-07-02,true,{},{}\n",
            contact.id, deal.id
        ),
    )
    .expect("activity CSV fixture should write");
    let csv_result = core
        .import_activities_csv(
            activities_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("activity CSV import should succeed");
    assert_eq!(csv_result.created, 1);
    assert_eq!(csv_result.merged, 0);
    assert_eq!(csv_result.skipped, 0);
    assert!(csv_result.errors.is_empty());
    assert!(csv_result.rollback_plan.is_some());

    let imported_csv_activity = core
        .list_activities()
        .expect("activities should list")
        .into_iter()
        .find(|activity| activity.title == "Follow up")
        .expect("CSV activity should exist");
    assert!(imported_csv_activity.completed);
    assert_eq!(
        imported_csv_activity.contact_id.as_deref(),
        Some(contact.id.as_str())
    );
    assert_eq!(
        imported_csv_activity.deal_id.as_deref(),
        Some(deal.id.as_str())
    );
    let links = core
        .list_activity_links(&imported_csv_activity.id)
        .expect("activity links should list");
    assert!(links
        .iter()
        .any(|link| link.entity_type.as_str() == "contact" && link.entity_id == contact.id));
    assert!(links
        .iter()
        .any(|link| link.entity_type.as_str() == "deal" && link.entity_id == deal.id));

    let activities_json_path = path.join("activities-import.json");
    std::fs::write(
        &activities_json_path,
        r#"[
  {
    "activity_type": "meeting",
    "title": "Renewal review",
    "description": "Review proposal",
    "due_date": "2026-07-03",
    "completed": false
  }
]"#,
    )
    .expect("activity JSON fixture should write");
    let json_result = core
        .import_activities_json(
            activities_json_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("activity JSON import should succeed");
    assert_eq!(json_result.created, 1);
    assert_eq!(json_result.skipped, 0);
    let imported_json_activity = core
        .list_activities()
        .expect("activities should list")
        .into_iter()
        .find(|activity| activity.title == "Renewal review")
        .expect("JSON activity should exist");
    assert!(!imported_json_activity.completed);
    assert_eq!(imported_json_activity.contact_id, None);
    assert_eq!(imported_json_activity.deal_id, None);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn mapped_activity_json_import_sets_custom_fields_without_duplicate_warnings() {
    let (mut core, path) = open_test_core();

    let activity_field = core
        .create_custom_field_def(
            "activity".to_string(),
            "Outcome".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("activity custom field should be created");
    let activities_json_path = path.join("activities-mapped-custom.json");
    std::fs::write(
        &activities_json_path,
        r#"[{"Kind":"email","Subject":"Send update","Done":"yes","Outcome":"Sent"}]"#,
    )
    .expect("activity mapped JSON fixture should write");
    let mapping = import_mapping(&[
        ("Kind", Some("activity_type")),
        ("Subject", Some("title")),
        ("Done", Some("completed")),
        ("Outcome", Some("custom:Outcome")),
    ]);

    let preflight = core
        .preflight_activities_json_import_with_mapping(
            activities_json_path
                .to_str()
                .expect("path should be valid UTF-8"),
            mapping.clone(),
        )
        .expect("activity mapped JSON preflight should succeed");
    assert_eq!(preflight.entity_type, "activities");
    assert_eq!(preflight.total_rows, 1);
    assert_eq!(preflight.duplicate_warning_count, 0);
    assert!(preflight.warnings.is_empty());

    let result = core
        .import_activities_json_with_mapping(
            activities_json_path
                .to_str()
                .expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped activity JSON import should succeed");
    assert_eq!(result.created, 1);
    assert_eq!(result.skipped, 0);
    let activity = core
        .list_activities()
        .expect("activities should list")
        .into_iter()
        .find(|activity| activity.title == "Send update")
        .expect("mapped activity should exist");
    assert_eq!(activity.activity_type, "email");
    assert!(activity.completed);
    let values = core
        .list_custom_field_values("activity", &activity.id)
        .expect("activity custom values should list");
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].field_def_id, activity_field.id);
    assert_eq!(values[0].value, "Sent");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn import_rollback_soft_deletes_created_activity_rows_and_custom_values() {
    let (mut core, path) = open_test_core();

    let activity_field = core
        .create_custom_field_def(
            "activity".to_string(),
            "Outcome".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("activity custom field should be created");
    let activities_csv_path = path.join("activities-custom-rollback.csv");
    std::fs::write(
        &activities_csv_path,
        "activity_type,title,custom:Outcome\ncall,Intro call,Positive\n",
    )
    .expect("activity rollback CSV fixture should write");
    let result = core
        .import_activities_csv(
            activities_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("activity custom import should succeed");
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("created activity custom import should return rollback plan");
    let imported_activity = core
        .list_activities()
        .expect("activities should list")
        .into_iter()
        .find(|activity| activity.title == "Intro call")
        .expect("imported activity should exist");
    let imported_activity_values = core
        .list_custom_field_values("activity", &imported_activity.id)
        .expect("activity custom value should list");
    assert_eq!(imported_activity_values[0].field_def_id, activity_field.id);
    let imported_activity_value_id = imported_activity_values[0].value_id.clone();
    let delete_audit_before =
        count_custom_field_audit_action(&core, &imported_activity_value_id, "delete_value");
    let delete_sync_before =
        count_custom_field_delete_sync(&core, &imported_activity_value_id, "Positive");

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("created activity custom rollback should succeed");
    assert_eq!(rollback.rolled_back, 1);
    assert_eq!(rollback.skipped, 0);
    assert!(rollback.errors.is_empty());
    assert!(core
        .list_activities()
        .expect("rolled back activities should list")
        .is_empty());
    assert!(core
        .list_custom_field_values("activity", &imported_activity.id)
        .expect("rolled back activity custom values should list")
        .is_empty());
    assert_eq!(
        count_custom_field_audit_action(&core, &imported_activity_value_id, "delete_value"),
        delete_audit_before + 1
    );
    assert_eq!(
        count_custom_field_delete_sync(&core, &imported_activity_value_id, "Positive"),
        delete_sync_before + 1
    );

    let conflict_csv_path = path.join("activities-conflict-rollback.csv");
    std::fs::write(
        &conflict_csv_path,
        "activity_type,title\nmeeting,Conflict check\n",
    )
    .expect("activity conflict CSV fixture should write");
    let conflict_result = core
        .import_activities_csv(
            conflict_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("activity conflict import should succeed");
    let conflict_plan = conflict_result
        .rollback_plan
        .clone()
        .expect("created activity import should return rollback plan");
    let conflict_activity_id = match &conflict_plan.actions[0] {
        super::import_rollback::ImportRollbackAction::Activity { entity_id, .. } => {
            entity_id.clone()
        }
        other => panic!("expected activity rollback action, got {other:?}"),
    };
    core.update_activity(
        &conflict_activity_id,
        None,
        Some("Edited conflict check".to_string()),
        None,
        None,
        None,
        None,
        None,
    )
    .expect("activity should be edited after import");
    let conflict_rollback = core
        .rollback_completed_import(&conflict_plan)
        .expect("conflict activity rollback should complete");
    assert_eq!(conflict_rollback.rolled_back, 0);
    assert_eq!(conflict_rollback.skipped, 1);
    assert_eq!(conflict_rollback.errors[0].code, "conflict");
    assert!(core.get_activity(&conflict_activity_id).is_ok());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn export_organizations_csv_writes_optional_fields() {
    let (mut core, path) = open_test_core();

    core.create_organization(
        "Acme Health".to_string(),
        Some("hello@acme.example".to_string()),
        Some("+123456".to_string()),
        Some("https://acme.example".to_string()),
        Some("Dock 4".to_string()),
        Some("Suite 9".to_string()),
        Some("Lagos".to_string()),
        Some("Lagos State".to_string()),
        Some("NG".to_string()),
        Some("100001".to_string()),
        Some("Regional partner".to_string()),
    )
    .expect("organization should be created before export");

    let csv_path = path.join("organizations-export.csv");
    let count = core
        .export_organizations_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("organization CSV export should succeed");

    assert_eq!(count, 1);
    let csv = std::fs::read_to_string(&csv_path).expect("organization CSV should read");
    assert!(csv.contains("name,email,phone,website,address_line1,address_line2,city,region,country,postal_code,description"));
    assert!(csv.contains("Acme Health"));
    assert!(csv.contains("hello@acme.example"));
    assert!(csv.contains("+123456"));
    assert!(csv.contains("https://acme.example"));
    assert!(csv.contains("Dock 4"));
    assert!(csv.contains("Suite 9"));
    assert!(csv.contains("Lagos State"));
    assert!(csv.contains("100001"));
    assert!(csv.contains("Regional partner"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn export_organizations_json_writes_active_flat_rows() {
    let (mut core, path) = open_test_core();

    core.create_organization(
        "Acme Health".to_string(),
        Some("hello@acme.example".to_string()),
        Some("+123456".to_string()),
        Some("https://acme.example".to_string()),
        Some("Dock 4".to_string()),
        Some("Suite 9".to_string()),
        Some("Lagos".to_string()),
        Some("Lagos State".to_string()),
        Some("NG".to_string()),
        Some("100001".to_string()),
        Some("Regional partner".to_string()),
    )
    .expect("active organization should be created");
    let deleted = core
        .create_organization(
            "Deleted Org".to_string(),
            Some("deleted@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("deleted organization should be created");
    core.delete_organization(&deleted.id)
        .expect("deleted organization should be soft-deleted");

    let json_path = path.join("organizations-export.json");
    let count = core
        .export_organizations_json(json_path.to_str().expect("path should be valid UTF-8"))
        .expect("organization JSON export should succeed");

    assert_eq!(count, 1);
    let rows = read_json_export(&json_path);
    assert_eq!(rows.len(), 1);
    let row = rows[0]
        .as_object()
        .expect("organization row should be an object");
    assert_eq!(
        row.get("name").and_then(|value| value.as_str()),
        Some("Acme Health")
    );
    assert_eq!(
        row.get("email").and_then(|value| value.as_str()),
        Some("hello@acme.example")
    );
    assert_eq!(
        row.get("phone").and_then(|value| value.as_str()),
        Some("+123456")
    );
    assert_eq!(
        row.get("website").and_then(|value| value.as_str()),
        Some("https://acme.example")
    );
    assert_eq!(
        row.get("description").and_then(|value| value.as_str()),
        Some("Regional partner")
    );
    assert!(row.get("id").is_none());
    assert!(row.get("source").is_none());
    assert!(row.get("deleted_at").is_none());
    assert!(!rows
        .iter()
        .any(|row| row.get("name").and_then(|value| value.as_str()) == Some("Deleted Org")));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn organization_custom_field_values_require_active_organization() {
    let (mut core, path) = open_test_core();

    let field = core
        .create_custom_field_def(
            "organization".to_string(),
            "Segment".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("organization custom field should be created");
    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should be created");

    let value = core
        .set_custom_field_value(
            field.id.clone(),
            organization.id.clone(),
            "Enterprise".to_string(),
        )
        .expect("organization custom value should be set");
    assert_eq!(value.value, "Enterprise");
    assert_eq!(
        count_custom_field_audit_action(&core, &value.id, "set_value"),
        1
    );
    assert_eq!(
        count_custom_field_set_sync(&core, &value.id, "Enterprise"),
        1
    );

    core.delete_organization(&organization.id)
        .expect("organization should be soft-deleted");
    let deleted_error = core
        .set_custom_field_value(
            field.id.clone(),
            organization.id.clone(),
            "Government".to_string(),
        )
        .expect_err("deleted organization custom value should be rejected");
    assert!(matches!(deleted_error, CrmError::NotFound(_)));

    let missing_error = core
        .set_custom_field_value(
            field.id.clone(),
            "missing-organization".to_string(),
            "Government".to_string(),
        )
        .expect_err("missing organization custom value should be rejected");
    assert!(matches!(missing_error, CrmError::NotFound(_)));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn organizations_export_and_import_custom_field_values() {
    let (mut core, path) = open_test_core();

    let segment_field = core
        .create_custom_field_def(
            "organization".to_string(),
            "Segment".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("segment custom field should be created");
    let escaped_field = core
        .create_custom_field_def(
            "organization".to_string(),
            "Plan #".to_string(),
            "text".to_string(),
            None,
            Some(1),
        )
        .expect("escaped organization custom field should be created");
    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should be created");
    core.set_custom_field_value(
        segment_field.id.clone(),
        organization.id.clone(),
        "Enterprise".to_string(),
    )
    .expect("segment value should be set");
    core.set_custom_field_value(
        escaped_field.id.clone(),
        organization.id.clone(),
        "Plus".to_string(),
    )
    .expect("escaped-name value should be set");

    let json_path = path.join("organizations-custom-export.json");
    let csv_path = path.join("organizations-custom-export.csv");
    assert_eq!(
        core.export_organizations_json(json_path.to_str().expect("path should be valid UTF-8"))
            .expect("organization JSON export should succeed"),
        1
    );
    assert_eq!(
        core.export_organizations_csv(csv_path.to_str().expect("path should be valid UTF-8"))
            .expect("organization CSV export should succeed"),
        1
    );
    let json_rows = read_json_export(&json_path);
    assert_eq!(
        json_rows[0]
            .get("custom:Segment")
            .and_then(|value| value.as_str()),
        Some("Enterprise")
    );
    assert_eq!(
        json_rows[0]
            .get("custom:Plan %23")
            .and_then(|value| value.as_str()),
        Some("Plus")
    );
    let csv_rows = read_csv_export(&csv_path);
    assert_eq!(
        csv_rows[0].get("custom:Segment").map(String::as_str),
        Some("Enterprise")
    );
    assert_eq!(
        csv_rows[0].get("custom:Plan %23").map(String::as_str),
        Some("Plus")
    );

    let import_csv_path = path.join("organizations-custom-import.csv");
    std::fs::write(
        &import_csv_path,
        "name,email,custom:Segment\nGlobex,hello@globex.example,Mid-market\n",
    )
    .expect("organization custom CSV fixture should write");
    let preflight = core
        .preflight_organizations_csv_import(
            import_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("organization custom preflight should succeed");
    assert_eq!(preflight.entity_type, "organizations");
    assert_eq!(preflight.total_rows, 1);

    let import_result = core
        .import_organizations_csv(
            import_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect("organization custom CSV import should succeed");
    assert_eq!(import_result.created, 1);
    let rollback_plan = import_result
        .rollback_plan
        .clone()
        .expect("created organization custom import should return rollback plan");
    let imported = core
        .list_organizations()
        .expect("organizations should list")
        .into_iter()
        .find(|organization| organization.name == "Globex")
        .expect("imported organization should exist");
    let imported_values = core
        .list_custom_field_values("organization", &imported.id)
        .expect("organization custom values should list");
    assert_eq!(imported_values.len(), 1);
    assert_eq!(imported_values[0].field_def_id, segment_field.id);
    assert_eq!(imported_values[0].value, "Mid-market");
    let imported_value_id = imported_values[0].value_id.clone();
    let delete_audit_before =
        count_custom_field_audit_action(&core, &imported_value_id, "delete_value");
    let delete_sync_before =
        count_custom_field_delete_sync(&core, &imported_value_id, "Mid-market");
    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("created organization custom rollback should succeed");
    assert_eq!(rollback.rolled_back, 1);
    assert!(rollback.errors.is_empty());
    assert!(core
        .list_organizations()
        .expect("rolled back organizations should list")
        .into_iter()
        .all(|organization| organization.name != "Globex"));
    assert!(core
        .list_custom_field_values("organization", &imported.id)
        .expect("rolled back organization custom values should list")
        .is_empty());
    assert_eq!(
        count_custom_field_audit_action(&core, &imported_value_id, "delete_value"),
        delete_audit_before + 1
    );
    assert_eq!(
        count_custom_field_delete_sync(&core, &imported_value_id, "Mid-market"),
        delete_sync_before + 1
    );

    let mapped_json_path = path.join("organizations-custom-mapped.json");
    std::fs::write(
        &mapped_json_path,
        r#"[{"Company":"Initech","Segment":"Enterprise"}]"#,
    )
    .expect("mapped organization JSON fixture should write");
    let mapping = import_mapping(&[
        ("Company", Some("name")),
        ("Segment", Some("custom:Segment")),
    ]);
    let mapped_preflight = core
        .preflight_organizations_json_import_with_mapping(
            mapped_json_path
                .to_str()
                .expect("path should be valid UTF-8"),
            mapping.clone(),
        )
        .expect("mapped organization custom preflight should succeed");
    assert_eq!(mapped_preflight.total_rows, 1);
    let mapped_result = core
        .import_organizations_json_with_mapping(
            mapped_json_path
                .to_str()
                .expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped organization custom import should succeed");
    assert_eq!(mapped_result.created, 1);
    let mapped_org = core
        .list_organizations()
        .expect("organizations should list")
        .into_iter()
        .find(|organization| organization.name == "Initech")
        .expect("mapped organization should exist");
    let mapped_values = core
        .list_custom_field_values("organization", &mapped_org.id)
        .expect("mapped organization custom values should list");
    assert_eq!(mapped_values.len(), 1);
    assert_eq!(mapped_values[0].field_def_id, segment_field.id);
    assert_eq!(mapped_values[0].value, "Enterprise");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn organization_duplicate_auto_merge_fills_missing_custom_values_without_overwriting() {
    let (mut core, path) = open_test_core();

    let segment_field = core
        .create_custom_field_def(
            "organization".to_string(),
            "Segment".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("segment custom field should be created");
    let missing_custom_org = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization without custom value should be created");
    let existing_custom_org = core
        .create_organization(
            "Globex".to_string(),
            Some("hello@globex.example".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization with custom value should be created");
    core.set_custom_field_value(
        segment_field.id.clone(),
        existing_custom_org.id.clone(),
        "Public Sector".to_string(),
    )
    .expect("existing organization custom value should be set");

    let csv_path = path.join("organizations-custom-auto-merge.csv");
    std::fs::write(
        &csv_path,
        "name,email,custom:Segment\nAcme Health,hello@acme.example,Enterprise\nGlobex,hello@globex.example,Mid-market\n",
    )
    .expect("organization custom auto-merge fixture should write");
    let result = core
        .import_organizations_csv_with_options(
            csv_path.to_str().expect("path should be valid UTF-8"),
            ImportOptions {
                merge_duplicates: true,
            },
        )
        .expect("organization custom auto-merge should succeed");
    assert_eq!(result.merged, 2);
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("filled organization custom merge should return rollback plan");

    let missing_values = core
        .list_custom_field_values("organization", &missing_custom_org.id)
        .expect("filled organization custom value should list");
    assert_eq!(missing_values.len(), 1);
    assert_eq!(missing_values[0].field_def_id, segment_field.id);
    assert_eq!(missing_values[0].value, "Enterprise");
    let filled_value_id = missing_values[0].value_id.clone();
    assert_eq!(
        core.list_custom_field_values("organization", &existing_custom_org.id)
            .expect("existing organization custom value should list")[0]
            .value,
        "Public Sector"
    );

    let delete_audit_before =
        count_custom_field_audit_action(&core, &filled_value_id, "delete_value");
    let delete_sync_before = count_custom_field_delete_sync(&core, &filled_value_id, "Enterprise");
    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("organization custom merge rollback should succeed");
    assert_eq!(rollback.rolled_back, 1);
    assert!(rollback.errors.is_empty());
    assert!(core
        .list_custom_field_values("organization", &missing_custom_org.id)
        .expect("rolled back organization custom values should list")
        .is_empty());
    assert_eq!(
        core.list_custom_field_values("organization", &existing_custom_org.id)
            .expect("existing organization custom value should still list")[0]
            .value,
        "Public Sector"
    );
    assert_eq!(
        count_custom_field_audit_action(&core, &filled_value_id, "delete_value"),
        delete_audit_before + 1
    );
    assert_eq!(
        count_custom_field_delete_sync(&core, &filled_value_id, "Enterprise"),
        delete_sync_before + 1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_organization_can_clear_optional_fields() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            Some("hello@acme.example".to_string()),
            Some("+123456".to_string()),
            Some("https://acme.example".to_string()),
            Some("Dock 4".to_string()),
            Some("Suite 9".to_string()),
            Some("Lagos".to_string()),
            Some("Lagos State".to_string()),
            Some("NG".to_string()),
            Some("100001".to_string()),
            Some("Regional partner".to_string()),
        )
        .expect("organization should be created");

    let updated = core
        .update_organization(
            &organization.id,
            None,
            Some(None),
            Some(Some("".to_string())),
            Some(Some("   ".to_string())),
            Some(None),
            Some(Some("".to_string())),
            Some(Some(" ".to_string())),
            Some(None),
            Some(Some("".to_string())),
            Some(Some(" ".to_string())),
            Some(Some("".to_string())),
        )
        .expect("organization optional fields should clear");

    assert_eq!(updated.name, "Acme Health");
    assert_eq!(updated.email, None);
    assert_eq!(updated.phone, None);
    assert_eq!(updated.website, None);
    assert_eq!(updated.address_line1, None);
    assert_eq!(updated.address_line2, None);
    assert_eq!(updated.city, None);
    assert_eq!(updated.region, None);
    assert_eq!(updated.country, None);
    assert_eq!(updated.postal_code, None);
    assert_eq!(updated.description, None);

    let audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'update' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("audit count should query");
    assert_eq!(audit_count, 1);

    let sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = '__update__'",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("sync count should query");
    assert_eq!(sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn link_contact_to_organization_writes_contact_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should be created");
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");

    let linked = core
        .link_contact_to_organization(&contact.id, Some(organization.id.clone()))
        .expect("contact should link to organization");
    assert_eq!(
        linked.organization_id.as_deref(),
        Some(organization.id.as_str())
    );
    assert_eq!(linked.org_id, None);
    assert_eq!(linked.org_name, "Acme Health");

    let audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'link_organization' AND entity_type = 'contact' AND entity_id = ?1",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("audit count should query");
    assert_eq!(audit_count, 1);

    let sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'contact' AND entity_id = ?1 AND field_name = 'organization_id'",
            params![contact.id],
            |row| row.get(0),
        )
        .expect("sync count should query");
    assert_eq!(sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn deal_contact_link_unlink_writes_audit_sync_and_updates_legacy_mirror() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            None,
            None,
            Some("Expansion project".to_string()),
        )
        .expect("deal should be created");

    let deal_contact = core
        .add_deal_contact(
            &deal.id,
            &contact.id,
            Some(" Decision maker ".to_string()),
            true,
        )
        .expect("deal contact should link");
    assert_eq!(deal_contact.deal_id, deal.id);
    assert_eq!(deal_contact.contact_id, contact.id);
    assert_eq!(deal_contact.role.as_deref(), Some("Decision maker"));
    assert!(deal_contact.is_primary);
    assert_eq!(
        core.get_deal(&deal.id)
            .expect("deal should load")
            .contact_id
            .as_deref(),
        Some(contact.id.as_str())
    );

    let active_links = core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list");
    assert_eq!(active_links.len(), 1);

    let link_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'link_contact' AND entity_type = 'deal_contact' AND entity_id = ?1",
            params![deal_contact.id],
            |row| row.get(0),
        )
        .expect("link audit count should query");
    assert_eq!(link_audit_count, 1);

    let link_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND entity_id = ?1 AND field_name = '__create__'",
            params![deal_contact.id],
            |row| row.get(0),
        )
        .expect("link sync count should query");
    assert_eq!(link_sync_count, 1);

    let deal_contact_mirror_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal' AND entity_id = ?1 AND field_name = 'contact_id'",
            params![deal.id],
            |row| row.get(0),
        )
        .expect("deal mirror sync count should query");
    assert_eq!(deal_contact_mirror_sync_count, 1);

    let removed = core
        .remove_deal_contact(&deal.id, &contact.id)
        .expect("deal contact should unlink");
    assert_eq!(removed.id, deal_contact.id);
    assert!(removed.deleted_at.is_some());
    assert_eq!(
        core.get_deal(&deal.id)
            .expect("deal should load after unlink")
            .contact_id,
        None
    );
    assert!(core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list after unlink")
        .is_empty());

    let unlink_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'unlink_contact' AND entity_type = 'deal_contact' AND entity_id = ?1",
            params![deal_contact.id],
            |row| row.get(0),
        )
        .expect("unlink audit count should query");
    assert_eq!(unlink_audit_count, 1);

    let unlink_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND entity_id = ?1 AND field_name = '__delete__'",
            params![deal_contact.id],
            |row| row.get(0),
        )
        .expect("unlink sync count should query");
    assert_eq!(unlink_sync_count, 1);

    let deal_contact_mirror_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal' AND entity_id = ?1 AND field_name = 'contact_id'",
            params![deal.id],
            |row| row.get(0),
        )
        .expect("deal mirror sync count should query after unlink");
    assert_eq!(deal_contact_mirror_sync_count, 2);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn create_deal_with_contact_id_creates_primary_deal_contact_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");

    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            Some(contact.id.clone()),
            None,
            Some("Expansion project".to_string()),
        )
        .expect("deal should be created");

    assert_eq!(deal.contact_id.as_deref(), Some(contact.id.as_str()));
    let links = core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list");
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].contact_id, contact.id);
    assert!(links[0].is_primary);

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'link_contact' AND entity_type = 'deal_contact'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND field_name = '__create__'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_deal_changing_contact_id_updates_primary_deal_contact_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let first_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("first contact should be created");
    let second_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Luis".to_string()),
            Some("Rivera".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("second contact should be created");
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            Some(first_contact.id.clone()),
            None,
            None,
        )
        .expect("deal should be created");

    let updated = core
        .update_deal(
            &deal.id,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Some(second_contact.id.clone())),
            None,
            None,
        )
        .expect("deal contact should update");

    assert_eq!(
        updated.contact_id.as_deref(),
        Some(second_contact.id.as_str())
    );
    let links = core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list");
    assert_eq!(
        links
            .iter()
            .filter(|link| link.is_primary && link.contact_id == second_contact.id)
            .count(),
        1
    );
    assert!(links
        .iter()
        .any(|link| !link.is_primary && link.contact_id == first_contact.id));
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal' AND field_name = 'contact_id'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND field_name = '__create__'"
        ),
        2
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND field_name = 'is_primary'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_deal_clearing_contact_id_removes_primary_deal_contact_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            Some(contact.id.clone()),
            None,
            None,
        )
        .expect("deal should be created");

    let updated = core
        .update_deal(
            &deal.id,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(None),
            None,
            None,
        )
        .expect("deal contact should clear");

    assert_eq!(updated.contact_id, None);
    assert!(core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list")
        .is_empty());
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'unlink_contact' AND entity_type = 'deal_contact'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal_contact' AND field_name = '__delete__'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal' AND field_name = 'contact_id'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn create_activity_with_contact_and_deal_creates_activity_links_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("deal should be created");

    let activity = core
        .create_activity(
            "task".to_string(),
            "Follow up".to_string(),
            None,
            None,
            Some(contact.id.clone()),
            Some(deal.id.clone()),
        )
        .expect("activity should be created");

    assert_eq!(activity.contact_id.as_deref(), Some(contact.id.as_str()));
    assert_eq!(activity.deal_id.as_deref(), Some(deal.id.as_str()));
    let links = core
        .list_activity_links(&activity.id)
        .expect("activity links should list");
    assert_eq!(links.len(), 2);
    assert!(links.iter().any(|link| {
        link.entity_type == crate::storage::activities::ActivityLinkEntityType::Contact
            && link.entity_id == contact.id
    }));
    assert!(links.iter().any(|link| {
        link.entity_type == crate::storage::activities::ActivityLinkEntityType::Deal
            && link.entity_id == deal.id
    }));

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action IN ('link_contact', 'link_deal') AND entity_type = 'activity_link'"
        ),
        2
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity_link' AND field_name = '__create__'"
        ),
        2
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn duplicate_contact_and_deal_activity_links_do_not_rewrite_activity_or_audit_sync() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("deal should be created");
    let activity = core
        .create_activity(
            "task".to_string(),
            "Follow up".to_string(),
            None,
            None,
            Some(contact.id.clone()),
            Some(deal.id.clone()),
        )
        .expect("activity should be created");

    let links_before = core
        .list_activity_links(&activity.id)
        .expect("activity links should list before duplicate adds");
    let contact_link_id = links_before
        .iter()
        .find(|link| {
            link.entity_type == crate::storage::activities::ActivityLinkEntityType::Contact
                && link.entity_id == contact.id
        })
        .expect("contact link should exist")
        .id
        .clone();
    let deal_link_id = links_before
        .iter()
        .find(|link| {
            link.entity_type == crate::storage::activities::ActivityLinkEntityType::Deal
                && link.entity_id == deal.id
        })
        .expect("deal link should exist")
        .id
        .clone();

    let sentinel_updated_at = "2026-06-24T09:15:00Z";
    core.db
        .conn
        .execute(
            "UPDATE activities SET updated_at = ?1 WHERE id = ?2",
            params![sentinel_updated_at, activity.id.as_str()],
        )
        .expect("activity updated_at should be pinned");
    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");

    let duplicate_contact_link = core
        .add_activity_link(&activity.id, "contact", &contact.id)
        .expect("duplicate contact activity link should be a no-op");
    let duplicate_deal_link = core
        .add_activity_link(&activity.id, "deal", &deal.id)
        .expect("duplicate deal activity link should be a no-op");

    assert_eq!(duplicate_contact_link.id, contact_link_id);
    assert_eq!(duplicate_deal_link.id, deal_link_id);
    let activity_after_duplicates = core
        .get_activity(&activity.id)
        .expect("activity should load after duplicate adds");
    assert_eq!(
        activity_after_duplicates.updated_at.as_str(),
        sentinel_updated_at
    );
    assert_eq!(
        activity_after_duplicates.contact_id.as_deref(),
        Some(contact.id.as_str())
    );
    assert_eq!(
        activity_after_duplicates.deal_id.as_deref(),
        Some(deal.id.as_str())
    );
    let active_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM activity_links WHERE activity_id = ?1 AND deleted_at IS NULL",
            params![activity.id],
            |row| row.get(0),
        )
        .expect("active activity link count should query");
    assert_eq!(active_link_count, 2);
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_activity_changing_and_clearing_contact_deal_keeps_links_and_mirrors_aligned() {
    let (mut core, path) = open_test_core();

    let first_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("first contact should be created");
    let second_contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Luis".to_string()),
            Some("Rivera".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("second contact should be created");
    let first_deal = core
        .create_deal(
            "First deal".to_string(),
            Some(1000.0),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("first deal should be created");
    let second_deal = core
        .create_deal(
            "Second deal".to_string(),
            Some(2000.0),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("second deal should be created");
    let activity = core
        .create_activity(
            "task".to_string(),
            "Follow up".to_string(),
            None,
            None,
            Some(first_contact.id.clone()),
            Some(first_deal.id.clone()),
        )
        .expect("activity should be created");

    let updated = core
        .update_activity(
            &activity.id,
            None,
            None,
            None,
            None,
            None,
            Some(Some(second_contact.id.clone())),
            Some(Some(second_deal.id.clone())),
        )
        .expect("activity relationships should update");
    assert_eq!(
        updated.contact_id.as_deref(),
        Some(second_contact.id.as_str())
    );
    assert_eq!(updated.deal_id.as_deref(), Some(second_deal.id.as_str()));

    let active_links = core
        .list_activity_links(&activity.id)
        .expect("activity links should list after update");
    assert_eq!(active_links.len(), 2);
    assert!(active_links
        .iter()
        .any(|link| link.entity_id == second_contact.id));
    assert!(active_links
        .iter()
        .any(|link| link.entity_id == second_deal.id));

    let cleared = core
        .update_activity(
            &activity.id,
            None,
            None,
            None,
            None,
            None,
            Some(None),
            Some(None),
        )
        .expect("activity relationships should clear");
    assert_eq!(cleared.contact_id, None);
    assert_eq!(cleared.deal_id, None);
    assert!(core
        .list_activity_links(&activity.id)
        .expect("activity links should be empty after clear")
        .is_empty());
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM activity_links WHERE deleted_at IS NOT NULL"
        ),
        4
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity_link' AND field_name = '__delete__'"
        ),
        4
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity' AND field_name IN ('contact_id', 'deal_id')"
        ),
        4
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn organization_activity_link_validates_reference_writes_audit_sync_and_skips_legacy_columns() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Regional Clinic".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should be created");
    let activity = core
        .create_activity(
            "meeting".to_string(),
            "Planning call".to_string(),
            None,
            None,
            None,
            None,
        )
        .expect("activity should be created");

    let err = core
        .add_activity_link(&activity.id, "organization", "missing-org")
        .expect_err("missing organization should be rejected");
    match err {
        crate::utils::errors::CrmError::NotFound(_) => {}
        other => panic!("expected NotFound, got {other:?}"),
    }

    let link = core
        .add_activity_link(&activity.id, "organization", &organization.id)
        .expect("organization activity link should be created");
    assert_eq!(link.entity_id, organization.id);
    assert_eq!(
        link.entity_type,
        crate::storage::activities::ActivityLinkEntityType::Organization
    );

    let activity_after_link = core
        .get_activity(&activity.id)
        .expect("activity should load after organization link");
    assert_eq!(activity_after_link.contact_id, None);
    assert_eq!(activity_after_link.deal_id, None);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'link_organization' AND entity_type = 'activity_link'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity_link' AND field_name = '__create__'"
        ),
        1
    );

    let removed = core
        .remove_activity_link(&activity.id, "organization", &organization.id)
        .expect("organization activity link should remove");
    assert!(removed.deleted_at.is_some());
    let activity_after_remove = core
        .get_activity(&activity.id)
        .expect("activity should load after organization unlink");
    assert_eq!(activity_after_remove.contact_id, None);
    assert_eq!(activity_after_remove.deal_id, None);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity' AND field_name IN ('contact_id', 'deal_id')"
        ),
        0
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'activity_link' AND field_name = '__delete__'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_deal_distinguishes_omitted_and_explicit_clears_for_legacy_fields() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            Some("2026-07-15".to_string()),
            Some(contact.id.clone()),
            None,
            None,
        )
        .expect("deal should be created");

    let unchanged = core
        .update_deal(
            &deal.id,
            Some("Renamed expansion".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("deal should update without clearing omitted fields");
    assert_eq!(unchanged.expected_close.as_deref(), Some("2026-07-15"));
    assert_eq!(unchanged.contact_id.as_deref(), Some(contact.id.as_str()));
    assert_eq!(
        core.list_deal_contacts(&deal.id)
            .expect("deal contacts should list")
            .len(),
        1
    );

    let cleared_expected_close = core
        .update_deal(
            &deal.id,
            None,
            None,
            None,
            None,
            None,
            Some(None),
            None,
            None,
            None,
        )
        .expect("expected close should clear");
    assert_eq!(cleared_expected_close.expected_close, None);
    assert_eq!(
        cleared_expected_close.contact_id.as_deref(),
        Some(contact.id.as_str())
    );

    let cleared_contact = core
        .update_deal(
            &deal.id,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(Some("   ".to_string())),
            None,
            None,
        )
        .expect("blank contact id should clear");
    assert_eq!(cleared_contact.contact_id, None);
    assert!(core
        .list_deal_contacts(&deal.id)
        .expect("deal contacts should list after clear")
        .is_empty());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn deal_organization_link_unlink_writes_audit_and_sync() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should be created");
    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            None,
            None,
            Some("Expansion project".to_string()),
        )
        .expect("deal should be created");

    let linked = core
        .link_deal_to_organization(&deal.id, Some(organization.id.clone()))
        .expect("deal should link to organization");
    assert_eq!(
        linked.organization_id.as_deref(),
        Some(organization.id.as_str())
    );

    let unlinked = core
        .link_deal_to_organization(&deal.id, None)
        .expect("deal should unlink from organization");
    assert_eq!(unlinked.organization_id, None);

    let link_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'link_organization' AND entity_type = 'deal' AND entity_id = ?1",
            params![deal.id],
            |row| row.get(0),
        )
        .expect("link organization audit count should query");
    assert_eq!(link_audit_count, 1);

    let unlink_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'unlink_organization' AND entity_type = 'deal' AND entity_id = ?1",
            params![deal.id],
            |row| row.get(0),
        )
        .expect("unlink organization audit count should query");
    assert_eq!(unlink_audit_count, 1);

    let sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'deal' AND entity_id = ?1 AND field_name = 'organization_id'",
            params![deal.id],
            |row| row.get(0),
        )
        .expect("deal organization sync count should query");
    assert_eq!(sync_count, 2);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn note_lifecycle_writes_note_audit_sync_and_soft_deletes() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Diallo".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");

    let note = core
        .create_note(
            "contact".to_string(),
            contact.id.clone(),
            "Initial note".to_string(),
        )
        .expect("note should be created");
    assert_eq!(note.content, "Initial note");

    let created_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'create' AND entity_type = 'note' AND entity_id = ?1",
            params![note.id],
            |row| row.get(0),
        )
        .expect("note create audit count should query");
    assert_eq!(created_audit_count, 1);

    let created_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'note' AND entity_id = ?1 AND field_name = '__create__'",
            params![note.id],
            |row| row.get(0),
        )
        .expect("note create sync count should query");
    assert_eq!(created_sync_count, 1);

    let updated = core
        .update_note(&note.id, "Updated note".to_string())
        .expect("note should update");
    assert_eq!(updated.content, "Updated note");

    let body_content_pair: (String, String) = core
        .db
        .conn
        .query_row(
            "SELECT content, body FROM notes WHERE id = ?1",
            params![note.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("note content/body should query");
    assert_eq!(
        body_content_pair,
        ("Updated note".to_string(), "Updated note".to_string())
    );

    core.delete_note(&note.id).expect("note should soft delete");

    assert!(core.get_note(&note.id).is_err());
    assert!(core
        .list_notes_for_entity("contact".to_string(), contact.id.clone())
        .expect("contact notes should list")
        .is_empty());

    let deleted_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE id = ?1 AND deleted_at IS NOT NULL",
            params![note.id],
            |row| row.get(0),
        )
        .expect("note deleted count should query");
    assert_eq!(deleted_count, 1);

    let delete_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'delete' AND entity_type = 'note' AND entity_id = ?1",
            params![note.id],
            |row| row.get(0),
        )
        .expect("note delete audit count should query");
    assert_eq!(delete_audit_count, 1);

    let delete_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'note' AND entity_id = ?1 AND field_name = '__delete__'",
            params![note.id],
            |row| row.get(0),
        )
        .expect("note delete sync count should query");
    assert_eq!(delete_sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn notes_read_legacy_content_and_target_body_without_loss() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Compat".to_string()),
            Some("Tester".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");

    let now = now_iso8601();
    core.db
        .conn
        .execute(
            r#"
            INSERT INTO notes
                (id, content, body, entity_type, entity_id, created_at, updated_at, device_id)
            VALUES
                ('legacy-content-note', 'Legacy content', NULL, 'contact', ?1, ?2, ?2, 'device-a')
            "#,
            params![contact.id, now],
        )
        .expect("legacy content note should insert");
    core.db
        .conn
        .execute(
            r#"
            INSERT INTO notes
                (id, content, body, entity_type, entity_id, created_at, updated_at, device_id)
            VALUES
                ('target-body-note', '', 'Target body', 'contact', ?1, ?2, ?2, 'device-a')
            "#,
            params![contact.id, now],
        )
        .expect("target body note should insert");

    let legacy = core
        .get_note("legacy-content-note")
        .expect("legacy content note should read");
    assert_eq!(legacy.content, "Legacy content");

    let target = core
        .get_note("target-body-note")
        .expect("target body note should read");
    assert_eq!(target.content, "Target body");

    let updated = core
        .update_note("target-body-note", "Unified body".to_string())
        .expect("target body note should update");
    assert_eq!(updated.content, "Unified body");

    let body_content_pair: (String, String) = core
        .db
        .conn
        .query_row(
            "SELECT content, body FROM notes WHERE id = 'target-body-note'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("updated content/body should query");
    assert_eq!(
        body_content_pair,
        ("Unified body".to_string(), "Unified body".to_string())
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn tag_lifecycle_writes_tag_audit_sync_and_soft_deletes() {
    let (mut core, path) = open_test_core();

    let tag = core
        .create_tag("VIP".to_string(), Some("#ef4444".to_string()))
        .expect("tag should be created");
    assert_eq!(tag.name, "VIP");
    assert_eq!(tag.color, "#ef4444");

    let create_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'create' AND entity_type = 'tag' AND entity_id = ?1",
            params![tag.id],
            |row| row.get(0),
        )
        .expect("tag create audit count should query");
    assert_eq!(create_audit_count, 1);

    let create_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'tag' AND entity_id = ?1 AND field_name = '__create__'",
            params![tag.id],
            |row| row.get(0),
        )
        .expect("tag create sync count should query");
    assert_eq!(create_sync_count, 1);

    let updated = core
        .update_tag(
            &tag.id,
            Some("Priority".to_string()),
            Some(TagColorUpdate::Set("#0f766e".to_string())),
        )
        .expect("tag should update");
    assert_eq!(updated.name, "Priority");
    assert_eq!(updated.color, "#0f766e");

    core.delete_tag(&tag.id).expect("tag should soft delete");

    assert!(core.get_tag(&tag.id).is_err());
    assert!(core
        .list_tags()
        .expect("tags should list")
        .iter()
        .all(|listed| listed.id != tag.id));

    let deleted_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tags WHERE id = ?1 AND deleted_at IS NOT NULL",
            params![tag.id],
            |row| row.get(0),
        )
        .expect("tag deleted count should query");
    assert_eq!(deleted_count, 1);

    let delete_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'tag' AND entity_id = ?1 AND field_name = '__delete__'",
            params![tag.id],
            |row| row.get(0),
        )
        .expect("tag delete sync count should query");
    assert_eq!(delete_sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn update_tag_distinguishes_omitted_color_from_explicit_reset() {
    let (mut core, path) = open_test_core();

    let tag = core
        .create_tag("Warm".to_string(), Some("#ef4444".to_string()))
        .expect("tag should be created");

    let renamed = core
        .update_tag(&tag.id, Some("Hot".to_string()), None)
        .expect("tag name should update without changing color");
    assert_eq!(renamed.name, "Hot");
    assert_eq!(renamed.color, "#ef4444");

    let reset_from_flag = core
        .update_tag(&tag.id, None, Some(TagColorUpdate::Reset))
        .expect("explicit reset should reset color");
    assert_eq!(
        reset_from_flag.color,
        crate::storage::tags::DEFAULT_TAG_COLOR
    );

    let recolored = core
        .update_tag(
            &tag.id,
            None,
            Some(TagColorUpdate::Set(" #0f766e ".to_string())),
        )
        .expect("explicit color should trim and update");
    assert_eq!(recolored.color, "#0f766e");

    let reset_from_blank = core
        .update_tag(&tag.id, None, Some(TagColorUpdate::Set("   ".to_string())))
        .expect("blank color should reset");
    assert_eq!(
        reset_from_blank.color,
        crate::storage::tags::DEFAULT_TAG_COLOR
    );

    let existing = core
        .create_tag("Existing".to_string(), None)
        .expect("second tag should be created");
    let err = core
        .update_tag(&tag.id, Some(existing.name), None)
        .expect_err("duplicate tag name should be rejected");
    assert!(err.to_string().contains("already exists"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn tag_apply_remove_mirrors_legacy_and_target_links_with_audit_sync() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Acme Health".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should be created");
    let tag = core
        .create_tag("Partner".to_string(), None)
        .expect("tag should be created");

    core.apply_tag_to_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("tag should apply to organization");

    let tags = core
        .list_tags_for_entity("organization".to_string(), organization.id.clone())
        .expect("organization tags should list");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].id, tag.id);

    let legacy_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM entity_tags WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("legacy entity_tags count should query");
    assert_eq!(legacy_link_count, 1);

    let target_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2 AND deleted_at IS NULL",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("target tag_links count should query");
    assert_eq!(target_link_count, 1);

    let apply_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'apply_tag' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("apply tag audit count should query");
    assert_eq!(apply_audit_count, 1);

    let apply_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = 'tags' AND new_value = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("apply tag sync count should query");
    assert_eq!(apply_sync_count, 1);

    core.remove_tag_from_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("tag should remove from organization");

    assert!(core
        .list_tags_for_entity("organization".to_string(), organization.id.clone())
        .expect("organization tags should list after remove")
        .is_empty());

    let legacy_removed_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM entity_tags WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("legacy entity_tags removed count should query");
    assert_eq!(legacy_removed_count, 0);

    let target_active_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2 AND deleted_at IS NULL",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("target active tag_links count should query");
    assert_eq!(target_active_count, 0);

    let target_soft_deleted_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2 AND deleted_at IS NOT NULL",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("target soft-deleted tag_links count should query");
    assert_eq!(target_soft_deleted_count, 1);

    let remove_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'remove_tag' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("remove tag audit count should query");
    assert_eq!(remove_audit_count, 1);

    let remove_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = 'tags' AND old_value = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("remove tag sync count should query");
    assert_eq!(remove_sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn duplicate_tag_apply_and_remove_do_not_record_extra_audit_or_sync_entries() {
    let (mut core, path) = open_test_core();

    let organization = core
        .create_organization(
            "Noiseless Links".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization should be created");
    let tag = core
        .create_tag("Quiet".to_string(), None)
        .expect("tag should be created");

    core.apply_tag_to_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("first apply should create link");
    core.apply_tag_to_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("duplicate apply should be idempotent");

    let legacy_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM entity_tags WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("legacy entity_tags count should query");
    assert_eq!(legacy_link_count, 1);

    let target_active_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2 AND deleted_at IS NULL",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("target active tag_links count should query");
    assert_eq!(target_active_count, 1);

    let apply_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'apply_tag' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("apply tag audit count should query");
    assert_eq!(apply_audit_count, 1);

    let apply_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = 'tags' AND new_value = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("apply tag sync count should query");
    assert_eq!(apply_sync_count, 1);

    core.remove_tag_from_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("first remove should remove link");
    core.remove_tag_from_entity(
        "organization".to_string(),
        organization.id.clone(),
        tag.id.clone(),
    )
    .expect("duplicate remove should be idempotent");

    let target_soft_deleted_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'organization' AND entity_id = ?1 AND tag_id = ?2 AND deleted_at IS NOT NULL",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("target soft-deleted tag_links count should query");
    assert_eq!(target_soft_deleted_count, 1);

    let remove_audit_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'remove_tag' AND entity_type = 'organization' AND entity_id = ?1",
            params![organization.id],
            |row| row.get(0),
        )
        .expect("remove tag audit count should query");
    assert_eq!(remove_audit_count, 1);

    let remove_sync_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sync_changelog WHERE entity_type = 'organization' AND entity_id = ?1 AND field_name = 'tags' AND old_value = ?2",
            params![organization.id, tag.id],
            |row| row.get(0),
        )
        .expect("remove tag sync count should query");
    assert_eq!(remove_sync_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn notes_and_tags_reject_unknown_entity_references() {
    let (mut core, path) = open_test_core();

    assert!(core
        .create_note(
            "invoice".to_string(),
            "entity-1".to_string(),
            "Unsupported".to_string(),
        )
        .is_err());

    assert!(core
        .create_note(
            "contact".to_string(),
            "missing-contact".to_string(),
            "Missing".to_string(),
        )
        .is_err());

    let tag = core
        .create_tag("Follow-up".to_string(), None)
        .expect("tag should be created");
    assert!(core
        .apply_tag_to_entity(
            "deal".to_string(),
            "missing-deal".to_string(),
            tag.id.clone(),
        )
        .is_err());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn tag_definition_import_export_and_rollback_are_local_and_idempotent() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("tag-definitions.csv");
    std::fs::write(&csv_path, "name,color\nVIP,#ef4444\nFollow-up,\n")
        .expect("tag definition CSV fixture should write");

    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");
    let preflight = core
        .preflight_tag_definitions_csv_import(
            csv_path.to_str().expect("path should be valid UTF-8"),
        )
        .expect("tag definition preflight should succeed");
    assert_eq!(preflight.entity_type, "tag_definitions");
    assert_eq!(preflight.total_rows, 2);
    assert_eq!(preflight.duplicate_warning_count, 0);
    assert!(preflight.warnings.is_empty());
    assert_eq!(count(&core, "SELECT COUNT(*) FROM tags"), 0);
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    let result = core
        .import_tag_definitions_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("tag definition import should succeed");
    assert_eq!(result.created, 2);
    assert_eq!(result.merged, 0);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("created tag definitions should return a rollback plan");

    let tags = core.list_tags().expect("tags should list after import");
    assert_eq!(tags.len(), 2);
    let vip = tags
        .iter()
        .find(|tag| tag.name == "VIP")
        .expect("VIP tag should import");
    assert_eq!(vip.color, "#ef4444");
    let follow_up = tags
        .iter()
        .find(|tag| tag.name == "Follow-up")
        .expect("Follow-up tag should import");
    assert_eq!(follow_up.color, crate::storage::tags::DEFAULT_TAG_COLOR);

    let csv_export_path = path.join("tag-definitions-export.csv");
    assert_eq!(
        core.export_tag_definitions_csv(
            csv_export_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("tag definition CSV export should succeed"),
        2
    );
    let csv_rows = read_csv_export(&csv_export_path);
    assert_eq!(csv_rows.len(), 2);
    assert!(csv_rows.iter().any(|row| {
        row.get("name").map(String::as_str) == Some("VIP")
            && row.get("color").map(String::as_str) == Some("#ef4444")
    }));

    let json_export_path = path.join("tag-definitions-export.json");
    assert_eq!(
        core.export_tag_definitions_json(
            json_export_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("tag definition JSON export should succeed"),
        2
    );
    let json_rows = read_json_export(&json_export_path);
    assert!(json_rows
        .iter()
        .any(|row| row.get("name").and_then(|value| value.as_str()) == Some("VIP")));

    let idempotent_result = core
        .import_tag_definitions_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("existing tag definition import should succeed");
    assert_eq!(idempotent_result.created, 0);
    assert_eq!(idempotent_result.skipped, 2);
    assert!(idempotent_result.errors.is_empty());

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("tag definition rollback should complete");
    assert_eq!(rollback.rolled_back, 2);
    assert_eq!(rollback.skipped, 0);
    assert!(rollback.errors.is_empty());
    assert!(core.list_tags().expect("tags should list").is_empty());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn tag_definition_rollback_skips_when_tag_changed_after_import() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("tag-definition-conflict.csv");
    std::fs::write(&csv_path, "name,color\nVIP,#ef4444\n")
        .expect("tag definition CSV fixture should write");

    let result = core
        .import_tag_definitions_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("tag definition import should succeed");
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("created tag should have rollback plan");
    let tag = core.list_tags().expect("tags should list")[0].clone();
    core.update_tag(
        &tag.id,
        None,
        Some(TagColorUpdate::Set("#0f766e".to_string())),
    )
    .expect("post-import tag edit should succeed");

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("tag definition conflict rollback should complete");
    assert_eq!(rollback.rolled_back, 0);
    assert_eq!(rollback.skipped, 1);
    assert_eq!(rollback.errors[0].code, "conflict");
    assert_eq!(
        core.get_tag(&tag.id)
            .expect("changed tag should remain after skipped rollback")
            .color,
        "#0f766e"
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn custom_field_definition_import_export_and_rollback_are_local_and_idempotent() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("custom-field-definitions.csv");
    std::fs::write(
        &csv_path,
        "entity_type,field_name,field_type,field_options,sort_order\ncontact,Status,select,\"[\"\"New\"\",\"\"Won\"\"]\",10\ndeal,Priority,text,,\n",
    )
    .expect("custom field definition CSV fixture should write");

    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");
    let preflight = core
        .preflight_custom_field_definitions_csv_import(
            csv_path.to_str().expect("path should be valid UTF-8"),
        )
        .expect("custom field definition preflight should succeed");
    assert_eq!(preflight.entity_type, "custom_field_definitions");
    assert_eq!(preflight.total_rows, 2);
    assert_eq!(preflight.duplicate_warning_count, 0);
    assert!(preflight.warnings.is_empty());
    assert_eq!(count(&core, "SELECT COUNT(*) FROM custom_field_defs"), 0);
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    let result = core
        .import_custom_field_definitions_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("custom field definition import should succeed");
    assert_eq!(result.created, 2);
    assert_eq!(result.merged, 0);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("created custom field definitions should return rollback plan");

    let definitions = core
        .list_custom_field_defs(None)
        .expect("custom field definitions should list after import");
    assert_eq!(definitions.len(), 2);
    let status = definitions
        .iter()
        .find(|definition| definition.field_name == "Status")
        .expect("Status definition should import");
    assert_eq!(status.entity_type, "contact");
    assert_eq!(status.field_type, "select");
    assert_eq!(status.field_options.as_deref(), Some("[\"New\",\"Won\"]"));
    assert_eq!(status.sort_order, 10);

    let csv_export_path = path.join("custom-field-definitions-export.csv");
    assert_eq!(
        core.export_custom_field_definitions_csv(
            csv_export_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("custom field definition CSV export should succeed"),
        2
    );
    let csv_rows = read_csv_export(&csv_export_path);
    assert!(csv_rows.iter().any(|row| {
        row.get("entity_type").map(String::as_str) == Some("contact")
            && row.get("field_name").map(String::as_str) == Some("Status")
            && row.get("field_type").map(String::as_str) == Some("select")
            && row.get("field_options").map(String::as_str) == Some("[\"New\",\"Won\"]")
            && row.get("sort_order").map(String::as_str) == Some("10")
    }));

    let json_export_path = path.join("custom-field-definitions-export.json");
    assert_eq!(
        core.export_custom_field_definitions_json(
            json_export_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("custom field definition JSON export should succeed"),
        2
    );
    let json_rows = read_json_export(&json_export_path);
    assert!(json_rows.iter().any(|row| {
        row.get("field_name").and_then(|value| value.as_str()) == Some("Status")
            && row.get("sort_order").and_then(|value| value.as_str()) == Some("10")
    }));

    let idempotent_result = core
        .import_custom_field_definitions_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("existing custom field definition import should succeed");
    assert_eq!(idempotent_result.created, 0);
    assert_eq!(idempotent_result.skipped, 2);
    assert!(idempotent_result.errors.is_empty());

    let conflict_csv_path = path.join("custom-field-definition-conflict.csv");
    std::fs::write(
        &conflict_csv_path,
        "entity_type,field_name,field_type,field_options,sort_order\ncontact,Status,text,,10\n",
    )
    .expect("conflicting custom field definition CSV fixture should write");
    let conflict = core
        .preflight_custom_field_definitions_csv_import(
            conflict_csv_path
                .to_str()
                .expect("path should be valid UTF-8"),
        )
        .expect_err("preflight should reject conflicting definition shape");
    assert!(conflict
        .to_string()
        .contains("already exists with different shape"));

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("custom field definition rollback should complete");
    assert_eq!(rollback.rolled_back, 2);
    assert_eq!(rollback.skipped, 0);
    assert!(rollback.errors.is_empty());
    assert!(core
        .list_custom_field_defs(None)
        .expect("custom field definitions should list")
        .is_empty());

    let json_path = path.join("custom-field-definitions-mapped.json");
    std::fs::write(
        &json_path,
        r#"[{"Owner":"organization","Label":"Segment","Kind":"text","Order":"7"}]"#,
    )
    .expect("mapped custom field definition JSON fixture should write");
    let mapping = import_mapping(&[
        ("Owner", Some("entity_type")),
        ("Label", Some("field_name")),
        ("Kind", Some("field_type")),
        ("Order", Some("sort_order")),
    ]);
    let json_preflight = core
        .preflight_custom_field_definitions_json_import_with_mapping(
            json_path.to_str().expect("path should be valid UTF-8"),
            mapping.clone(),
        )
        .expect("mapped JSON preflight should succeed");
    assert_eq!(json_preflight.total_rows, 1);
    let json_result = core
        .import_custom_field_definitions_json_with_mapping(
            json_path.to_str().expect("path should be valid UTF-8"),
            mapping,
        )
        .expect("mapped JSON custom field definition import should succeed");
    assert_eq!(json_result.created, 1);
    assert!(core
        .list_custom_field_defs(Some("organization".to_string()))
        .expect("organization definitions should list")
        .iter()
        .any(|definition| definition.field_name == "Segment" && definition.sort_order == 7));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn custom_field_definition_rollback_skips_modified_or_value_bearing_definitions() {
    let (mut core, path) = open_test_core();
    let csv_path = path.join("custom-field-definition-rollback-skip.csv");
    std::fs::write(
        &csv_path,
        "entity_type,field_name,field_type,field_options,sort_order\ncontact,Tier,text,,1\ncontact,Region,text,,2\n",
    )
    .expect("custom field definition CSV fixture should write");
    let result = core
        .import_custom_field_definitions_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("custom field definition import should succeed");
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("created custom field definitions should have rollback plan");

    let definitions = core
        .list_custom_field_defs(Some("contact".to_string()))
        .expect("contact definitions should list");
    let tier = definitions
        .iter()
        .find(|definition| definition.field_name == "Tier")
        .expect("Tier definition should exist")
        .clone();
    let region = definitions
        .iter()
        .find(|definition| definition.field_name == "Region")
        .expect("Region definition should exist")
        .clone();

    core.update_custom_field_def(&tier.id, None, None, None, Some(5))
        .expect("post-import definition edit should succeed");
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Value".to_string()),
            Some("Bearing".to_string()),
            None,
            Some("value-bearing@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should create");
    core.set_custom_field_value(region.id.clone(), contact.id, "North".to_string())
        .expect("custom field value should create");

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("custom field definition rollback skip should complete");
    assert_eq!(rollback.rolled_back, 0);
    assert_eq!(rollback.skipped, 2);
    assert_eq!(rollback.errors.len(), 2);
    assert!(rollback.errors.iter().all(|error| error.code == "conflict"));
    let remaining_definitions = core
        .list_custom_field_defs(Some("contact".to_string()))
        .expect("contact definitions should list after skipped rollback");
    assert_eq!(
        remaining_definitions
            .iter()
            .find(|definition| definition.id == tier.id)
            .expect("modified definition should remain")
            .sort_order,
        5
    );
    assert!(remaining_definitions
        .iter()
        .any(|definition| definition.id == region.id));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn tag_link_import_export_preflight_and_rollback_use_local_ids() {
    let (mut core, path) = open_test_core();
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Lovelace".to_string()),
            None,
            Some("ada@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact fixture should be created");
    let tag = core
        .create_tag("VIP".to_string(), Some("#ef4444".to_string()))
        .expect("tag fixture should be created");
    let csv_path = path.join("tag-links.csv");
    std::fs::write(
        &csv_path,
        format!(
            "entity_type,entity_id,tag_id\ncontact,{},{}\n",
            contact.id, tag.id
        ),
    )
    .expect("tag link CSV fixture should write");

    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");
    let preflight = core
        .preflight_tag_links_csv_import(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("tag link preflight should succeed");
    assert_eq!(preflight.entity_type, "tag_links");
    assert_eq!(preflight.total_rows, 1);
    assert_eq!(preflight.duplicate_warning_count, 0);
    assert!(preflight.warnings.is_empty());
    assert!(core
        .list_tags_for_entity("contact".to_string(), contact.id.clone())
        .expect("contact tags should list")
        .is_empty());
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    let result = core
        .import_tag_links_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("tag link import should succeed");
    assert_eq!(result.created, 1);
    assert_eq!(result.merged, 0);
    assert_eq!(result.skipped, 0);
    assert!(result.errors.is_empty());
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("created tag link should have rollback plan");
    assert_eq!(
        core.list_tags_for_entity("contact".to_string(), contact.id.clone())
            .expect("contact tags should list")
            .len(),
        1
    );

    let csv_export_path = path.join("tag-links-export.csv");
    assert_eq!(
        core.export_tag_links_csv(
            csv_export_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("tag link CSV export should succeed"),
        1
    );
    let csv_rows = read_csv_export(&csv_export_path);
    assert_eq!(
        csv_rows[0].get("entity_type").map(String::as_str),
        Some("contact")
    );
    assert_eq!(
        csv_rows[0].get("entity_id").map(String::as_str),
        Some(contact.id.as_str())
    );
    assert_eq!(
        csv_rows[0].get("tag_id").map(String::as_str),
        Some(tag.id.as_str())
    );

    let json_export_path = path.join("tag-links-export.json");
    assert_eq!(
        core.export_tag_links_json(
            json_export_path
                .to_str()
                .expect("path should be valid UTF-8")
        )
        .expect("tag link JSON export should succeed"),
        1
    );
    let json_rows = read_json_export(&json_export_path);
    assert_eq!(
        json_rows[0].get("tag_id").and_then(|value| value.as_str()),
        Some(tag.id.as_str())
    );

    let idempotent_result = core
        .import_tag_links_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("existing tag link import should succeed");
    assert_eq!(idempotent_result.created, 0);
    assert_eq!(idempotent_result.skipped, 1);
    assert!(idempotent_result.errors.is_empty());

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("tag link rollback should complete");
    assert_eq!(rollback.rolled_back, 1);
    assert_eq!(rollback.skipped, 0);
    assert!(rollback.errors.is_empty());
    assert!(core
        .list_tags_for_entity("contact".to_string(), contact.id.clone())
        .expect("contact tags should list after rollback")
        .is_empty());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn tag_link_rollback_skips_after_link_is_removed_and_reapplied() {
    let (mut core, path) = open_test_core();
    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Grace".to_string()),
            Some("Hopper".to_string()),
            None,
            Some("grace@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact fixture should be created");
    let tag = core
        .create_tag("Reapplied".to_string(), Some("#0f766e".to_string()))
        .expect("tag fixture should be created");
    let csv_path = path.join("tag-link-reapply.csv");
    std::fs::write(
        &csv_path,
        format!(
            "entity_type,entity_id,tag_id\ncontact,{},{}\n",
            contact.id, tag.id
        ),
    )
    .expect("tag link CSV fixture should write");

    let result = core
        .import_tag_links_csv(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect("tag link import should succeed");
    let rollback_plan = result
        .rollback_plan
        .clone()
        .expect("created tag link should have rollback plan");
    assert_eq!(
        core.list_tags_for_entity("contact".to_string(), contact.id.clone())
            .expect("imported contact tag should list")
            .len(),
        1
    );

    core.remove_tag_from_entity("contact".to_string(), contact.id.clone(), tag.id.clone())
        .expect("post-import user remove should succeed");
    assert!(core
        .list_tags_for_entity("contact".to_string(), contact.id.clone())
        .expect("removed contact tags should list")
        .is_empty());

    core.apply_tag_to_entity("contact".to_string(), contact.id.clone(), tag.id.clone())
        .expect("post-import user reapply should succeed");
    assert_eq!(
        core.list_tags_for_entity("contact".to_string(), contact.id.clone())
            .expect("reapplied contact tag should list")
            .len(),
        1
    );

    let rollback = core
        .rollback_completed_import(&rollback_plan)
        .expect("stale tag link rollback should complete");
    assert_eq!(rollback.rolled_back, 0);
    assert_eq!(rollback.skipped, 1);
    assert_eq!(rollback.errors.len(), 1);
    assert_eq!(rollback.errors[0].code, "not_found");
    assert_eq!(
        core.list_tags_for_entity("contact".to_string(), contact.id.clone())
            .expect("newer reapplied contact tag should remain")
            .len(),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'contact' AND deleted_at IS NULL",
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn tag_link_preflight_rejects_unknown_local_references_without_writes() {
    let (mut core, path) = open_test_core();
    let tag = core
        .create_tag("VIP".to_string(), None)
        .expect("tag fixture should be created");
    let csv_path = path.join("tag-links-invalid.csv");
    std::fs::write(
        &csv_path,
        format!(
            "entity_type,entity_id,tag_id\ncontact,missing-contact,{}\n",
            tag.id
        ),
    )
    .expect("invalid tag link CSV fixture should write");

    let audit_count_before = count(&core, "SELECT COUNT(*) FROM audit_log");
    let sync_count_before = count(&core, "SELECT COUNT(*) FROM sync_changelog");
    let err = core
        .preflight_tag_links_csv_import(csv_path.to_str().expect("path should be valid UTF-8"))
        .expect_err("missing parent should fail preflight");
    match err {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("Row 2:"), "{message}");
            assert!(message.contains("missing-contact"), "{message}");
        }
        other => panic!("expected InvalidInput for missing parent, got {other:?}"),
    }
    assert_eq!(count(&core, "SELECT COUNT(*) FROM entity_tags"), 0);
    assert_eq!(count(&core, "SELECT COUNT(*) FROM tag_links"), 0);
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM audit_log"),
        audit_count_before
    );
    assert_eq!(
        count(&core, "SELECT COUNT(*) FROM sync_changelog"),
        sync_count_before
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_clients_default_to_disabled() {
    let (mut core, path) = open_test_core();

    let client = core
        .create_external_client_placeholder(" Claude Desktop ", " mcp ")
        .expect("external client placeholder should be created");

    assert_eq!(client.name, "Claude Desktop");
    assert_eq!(client.client_type, "mcp");
    assert_eq!(client.permission_mode, "disabled");
    assert!(!client.enabled);

    let disabled_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM external_clients WHERE id = ?1 AND enabled = 0 AND permission_mode = 'disabled'",
            params![client.id],
            |row| row.get(0),
        )
        .expect("external client count should query");
    assert_eq!(disabled_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_activation_enables_read_only_and_records_audit_sync() {
    let (mut core, path) = open_test_core();

    let client = core
        .create_external_client_placeholder("Claude Desktop", "mcp")
        .expect("external client placeholder should be created");
    core.db
        .conn
        .execute(
            "UPDATE external_clients SET updated_at = '2026-06-24T08:00:00Z' WHERE id = ?1",
            params![&client.id],
        )
        .expect("external client timestamp should be pinned");

    let updated = core
        .update_external_client_activation(&client.id, true, "read_only")
        .expect("external client should enable read-only");

    assert!(updated.enabled);
    assert_eq!(updated.permission_mode, "read_only");
    assert_ne!(updated.updated_at, "2026-06-24T08:00:00Z");
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM external_clients WHERE enabled = 1 AND permission_mode = 'read_only'"
        ),
        1
    );
    assert_eq!(
        count_external_client_activation_audit_rows(&core, &client.id),
        1
    );
    assert_eq!(count_external_client_sync_rows(&core, &client.id), 1);

    let read = core
        .evaluate_external_client_tool_read_permission(&client.id, "contacts.search")
        .expect("read-only activation should be evaluable");
    assert_eq!(
        read.reason,
        ToolPermissionDecisionReason::MissingToolPermission
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_activation_noop_preserves_timestamp_without_audit_or_sync() {
    let (mut core, path) = open_test_core();

    let client = core
        .create_external_client_placeholder("Claude Desktop", "mcp")
        .expect("external client placeholder should be created");
    let first = core
        .update_external_client_activation(&client.id, true, "read_only")
        .expect("external client should enable read-only");
    assert_eq!(
        count_external_client_activation_audit_rows(&core, &client.id),
        1
    );
    assert_eq!(count_external_client_sync_rows(&core, &client.id), 1);

    let second = core
        .update_external_client_activation(&client.id, true, "read_only")
        .expect("same activation state should be idempotent");

    assert_eq!(second.id, first.id);
    assert!(second.enabled);
    assert_eq!(second.permission_mode, "read_only");
    assert_eq!(second.updated_at, first.updated_at);
    assert_eq!(
        count_external_client_activation_audit_rows(&core, &client.id),
        1
    );
    assert_eq!(count_external_client_sync_rows(&core, &client.id), 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_activation_enables_draft_only() {
    let (mut core, path) = open_test_core();

    let client = core
        .create_external_client_placeholder("Local Draft Client", "mcp")
        .expect("external client placeholder should be created");

    let updated = core
        .update_external_client_activation(&client.id, true, "draft_only")
        .expect("external client should enable draft-only");

    assert!(updated.enabled);
    assert_eq!(updated.permission_mode, "draft_only");
    let draft = core
        .evaluate_external_client_draft_permission(&client.id, "create_activity_draft")
        .expect("draft-only activation should be evaluable");
    assert_eq!(
        draft.reason,
        ToolPermissionDecisionReason::MissingToolPermission
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_activation_disables_client_and_keeps_evaluation_conservative() {
    let (mut core, path) = open_test_core();

    let client = create_test_external_client_with_mode(&mut core, "read_only");
    core.upsert_external_client_tool_permission(&client.id, "contacts.search", true, false, true)
        .expect("permission row should upsert before disabling");

    let updated = core
        .update_external_client_activation(&client.id, false, "disabled")
        .expect("external client should disable");

    assert!(!updated.enabled);
    assert_eq!(updated.permission_mode, "disabled");
    let read = core
        .evaluate_external_client_tool_read_permission(&client.id, "contacts.search")
        .expect("disabled client should still evaluate");
    assert_eq!(read.reason, ToolPermissionDecisionReason::ClientDisabled);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_activation_rejects_future_permission_modes() {
    for mode in ["write_with_confirmation", "write_allowed"] {
        let (mut core, path) = open_test_core();
        let client = core
            .create_external_client_placeholder("Future Client", "mcp")
            .expect("external client placeholder should be created");

        let err = core
            .update_external_client_activation(&client.id, true, mode)
            .expect_err("future activation mode should be rejected");
        match err {
            CrmError::InvalidInput(message) => {
                assert!(message.contains("reserved for future support"), "{message}");
                assert!(message.contains(mode), "{message}");
            }
            other => panic!("expected InvalidInput for future mode, got {other:?}"),
        }
        assert_eq!(
            count_external_client_activation_audit_rows(&core, &client.id),
            0
        );
        assert_eq!(count_external_client_sync_rows(&core, &client.id), 0);

        let stored = core
            .list_external_clients()
            .expect("external clients should list after rejected activation");
        assert_eq!(stored[0].permission_mode, "disabled");
        assert!(!stored[0].enabled);

        drop(core);
        let _ = std::fs::remove_dir_all(path);
    }
}

#[test]
fn external_client_activation_rejects_inconsistent_enabled_mode_pairs() {
    let (mut core, path) = open_test_core();
    let client = core
        .create_external_client_placeholder("Inconsistent Client", "mcp")
        .expect("external client placeholder should be created");

    let enabled_disabled = core
        .update_external_client_activation(&client.id, true, "disabled")
        .expect_err("enabled disabled-mode client should be rejected");
    match enabled_disabled {
        CrmError::InvalidInput(message) => {
            assert!(
                message.contains("Enabled external clients must use"),
                "{message}"
            );
        }
        other => panic!("expected InvalidInput for enabled disabled mode, got {other:?}"),
    }

    let disabled_read_only = core
        .update_external_client_activation(&client.id, false, "read_only")
        .expect_err("disabled read-only client should be rejected");
    match disabled_read_only {
        CrmError::InvalidInput(message) => {
            assert!(
                message.contains("Disabled external clients must use"),
                "{message}"
            );
        }
        other => panic!("expected InvalidInput for disabled read-only mode, got {other:?}"),
    }

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_activation_rejects_unknown_and_deleted_clients() {
    let (mut core, path) = open_test_core();

    let missing = core
        .update_external_client_activation("missing-client", true, "read_only")
        .expect_err("unknown client activation should fail");
    assert!(matches!(missing, CrmError::NotFound(_)));

    let client = core
        .create_external_client_placeholder("Deleted Client", "mcp")
        .expect("external client placeholder should be created");
    core.db
        .conn
        .execute(
            "UPDATE external_clients SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now_iso8601(), &client.id],
        )
        .expect("external client should be marked deleted");

    let deleted = core
        .update_external_client_activation(&client.id, true, "read_only")
        .expect_err("deleted client activation should fail");
    assert!(matches!(deleted, CrmError::NotFound(_)));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn list_external_clients_returns_disabled_placeholders_in_created_order() {
    let (mut core, path) = open_test_core();

    let first = core
        .create_external_client_placeholder("Claude Desktop", "mcp")
        .expect("first external client placeholder should be created");
    let second = core
        .create_external_client_placeholder("Local Script", "script")
        .expect("second external client placeholder should be created");

    let clients = core
        .list_external_clients()
        .expect("external clients should list");

    assert_eq!(clients.len(), 2);
    assert_eq!(clients[0].id, first.id);
    assert_eq!(clients[1].id, second.id);
    assert!(clients.iter().all(|client| !client.enabled));
    assert!(clients
        .iter()
        .all(|client| client.permission_mode == "disabled"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn external_client_placeholder_rejects_blank_required_fields() {
    let (mut core, path) = open_test_core();

    let blank_name = core
        .create_external_client_placeholder("   ", "mcp")
        .expect_err("blank external client name should be rejected");
    match blank_name {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("name"));
        }
        other => panic!("expected InvalidInput for blank name, got {other:?}"),
    }

    let blank_type = core
        .create_external_client_placeholder("Claude Desktop", "\t")
        .expect_err("blank external client type should be rejected");
    match blank_type {
        CrmError::InvalidInput(message) => {
            assert!(message.contains("client_type"));
        }
        other => panic!("expected InvalidInput for blank client type, got {other:?}"),
    }

    let clients = core
        .list_external_clients()
        .expect("external clients should list after rejected creates");
    assert!(clients.is_empty());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v7_creates_deal_relationship_schema() {
    let path = std::env::temp_dir().join(format!("900crm-v7-schema-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE contacts (
                id              TEXT PRIMARY KEY NOT NULL,
                organization_id TEXT,
                deleted_at      TEXT
            );
            CREATE TABLE organizations (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE deals (
                id          TEXT PRIMARY KEY NOT NULL,
                contact_id  TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL DEFAULT ''
            );
            PRAGMA user_version = 6;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and run v7");

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM pragma_table_info('deals') WHERE name = 'organization_id'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'deal_contacts'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_deal_contacts_active_primary'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v7_backfills_legacy_deal_contact_and_organization_link() {
    let path = std::env::temp_dir().join(format!("900crm-v7-backfill-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE contacts (
                id              TEXT PRIMARY KEY NOT NULL,
                organization_id TEXT,
                deleted_at      TEXT
            );
            CREATE TABLE organizations (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE deals (
                id          TEXT PRIMARY KEY NOT NULL,
                contact_id  TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO organizations (id, deleted_at)
            VALUES ('org-1', NULL);
            INSERT INTO contacts (id, organization_id, deleted_at)
            VALUES ('contact-1', 'org-1', NULL);
            INSERT INTO deals (id, contact_id, created_at, updated_at, deleted_at, device_id)
            VALUES ('deal-1', 'contact-1', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a');
            PRAGMA user_version = 6;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and backfill v7");

    let backfilled_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM deal_contacts WHERE deal_id = 'deal-1' AND contact_id = 'contact-1' AND is_primary = 1 AND deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
        .expect("deal contact backfill count should query");
    assert_eq!(backfilled_link_count, 1);

    let organization_id: Option<String> = core
        .db
        .conn
        .query_row(
            "SELECT organization_id FROM deals WHERE id = 'deal-1'",
            [],
            |row| row.get(0),
        )
        .expect("deal organization backfill should query");
    assert_eq!(organization_id.as_deref(), Some("org-1"));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v7_skips_deal_organization_backfill_for_missing_or_deleted_organizations() {
    let path = std::env::temp_dir().join(format!("900crm-v7-invalid-org-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE contacts (
                id              TEXT PRIMARY KEY NOT NULL,
                organization_id TEXT,
                deleted_at      TEXT
            );
            CREATE TABLE organizations (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE deals (
                id          TEXT PRIMARY KEY NOT NULL,
                contact_id  TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO organizations (id, deleted_at)
            VALUES ('deleted-org', '2026-06-24T08:30:00Z');
            INSERT INTO contacts (id, organization_id, deleted_at)
            VALUES
                ('missing-org-contact', 'missing-org', NULL),
                ('deleted-org-contact', 'deleted-org', NULL);
            INSERT INTO deals (id, contact_id, created_at, updated_at, deleted_at, device_id)
            VALUES
                ('missing-org-deal', 'missing-org-contact', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a'),
                ('deleted-org-deal', 'deleted-org-contact', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a');
            PRAGMA user_version = 6;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and run v7");

    let dangling_deal_org_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM deals WHERE organization_id IS NOT NULL AND organization_id <> ''",
            [],
            |row| row.get(0),
        )
        .expect("deal organization count should query");
    assert_eq!(dangling_deal_org_count, 0);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM deal_contacts WHERE is_primary = 1 AND deleted_at IS NULL"
        ),
        2
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v7_skips_deal_organization_backfill_when_organizations_table_is_missing() {
    let path = std::env::temp_dir().join(format!("900crm-v7-no-org-table-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE contacts (
                id              TEXT PRIMARY KEY NOT NULL,
                organization_id TEXT,
                deleted_at      TEXT
            );
            CREATE TABLE deals (
                id          TEXT PRIMARY KEY NOT NULL,
                contact_id  TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                deleted_at  TEXT,
                device_id   TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO contacts (id, organization_id, deleted_at)
            VALUES ('contact-1', 'missing-org', NULL);
            INSERT INTO deals (id, contact_id, created_at, updated_at, deleted_at, device_id)
            VALUES ('deal-1', 'contact-1', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a');
            PRAGMA user_version = 6;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and skip org backfill safely");

    let organization_id: Option<String> = core
        .db
        .conn
        .query_row(
            "SELECT organization_id FROM deals WHERE id = 'deal-1'",
            [],
            |row| row.get(0),
        )
        .expect("deal organization should query");
    assert_eq!(organization_id, None);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM deal_contacts WHERE deal_id = 'deal-1' AND contact_id = 'contact-1' AND is_primary = 1 AND deleted_at IS NULL"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v8_creates_activity_relationship_schema_and_backfills_valid_legacy_links() {
    let path = std::env::temp_dir().join(format!("900crm-v8-backfill-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE contacts (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE deals (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE activities (
                id            TEXT PRIMARY KEY NOT NULL,
                activity_type TEXT NOT NULL DEFAULT 'task',
                title         TEXT NOT NULL DEFAULT '',
                description   TEXT NOT NULL DEFAULT '',
                due_date      TEXT,
                completed     INTEGER NOT NULL DEFAULT 0,
                contact_id    TEXT,
                deal_id       TEXT,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                deleted_at    TEXT,
                device_id     TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO contacts (id, deleted_at) VALUES ('contact-1', NULL);
            INSERT INTO deals (id, deleted_at) VALUES ('deal-1', NULL);
            INSERT INTO activities
                (id, activity_type, title, description, contact_id, deal_id, created_at, updated_at, deleted_at, device_id)
            VALUES
                ('activity-1', 'task', 'Follow up', '', 'contact-1', 'deal-1', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a');
            PRAGMA user_version = 7;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and run v8");

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'activity_links'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_activity_links_active_unique'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM activity_links WHERE activity_id = 'activity-1' AND entity_type = 'contact' AND entity_id = 'contact-1' AND deleted_at IS NULL"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM activity_links WHERE activity_id = 'activity-1' AND entity_type = 'deal' AND entity_id = 'deal-1' AND deleted_at IS NULL"
        ),
        1
    );
    assert_eq!(
        core.db
            .schema_version()
            .expect("schema version should read"),
        crate::storage::Database::current_schema_version()
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v8_skips_missing_deleted_and_deleted_activity_legacy_links() {
    let path = std::env::temp_dir().join(format!("900crm-v8-invalid-backfill-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE contacts (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE deals (
                id         TEXT PRIMARY KEY NOT NULL,
                deleted_at TEXT
            );
            CREATE TABLE activities (
                id            TEXT PRIMARY KEY NOT NULL,
                activity_type TEXT NOT NULL DEFAULT 'task',
                title         TEXT NOT NULL DEFAULT '',
                description   TEXT NOT NULL DEFAULT '',
                due_date      TEXT,
                completed     INTEGER NOT NULL DEFAULT 0,
                contact_id    TEXT,
                deal_id       TEXT,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                deleted_at    TEXT,
                device_id     TEXT NOT NULL DEFAULT ''
            );
            INSERT INTO contacts (id, deleted_at)
            VALUES
                ('active-contact', NULL),
                ('deleted-contact', '2026-06-24T09:00:00Z');
            INSERT INTO deals (id, deleted_at)
            VALUES
                ('active-deal', NULL),
                ('deleted-deal', '2026-06-24T09:00:00Z');
            INSERT INTO activities
                (id, activity_type, title, description, contact_id, deal_id, created_at, updated_at, deleted_at, device_id)
            VALUES
                ('missing-contact-activity', 'task', 'Missing contact', '', 'missing-contact', NULL, '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a'),
                ('deleted-contact-activity', 'task', 'Deleted contact', '', 'deleted-contact', NULL, '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a'),
                ('missing-deal-activity', 'task', 'Missing deal', '', NULL, 'missing-deal', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a'),
                ('deleted-deal-activity', 'task', 'Deleted deal', '', NULL, 'deleted-deal', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', NULL, 'device-a'),
                ('deleted-activity', 'task', 'Deleted activity', '', 'active-contact', 'active-deal', '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', '2026-06-24T09:30:00Z', 'device-a');
            PRAGMA user_version = 7;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and run v8");

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM activity_links WHERE deleted_at IS NULL"
        ),
        0
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v9_deduplicates_external_client_permissions_and_upsert_keeps_unique_pair() {
    let path = std::env::temp_dir().join(format!("900crm-v9-permission-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE external_clients (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                client_type     TEXT NOT NULL,
                permission_mode TEXT NOT NULL DEFAULT 'disabled',
                enabled         INTEGER NOT NULL DEFAULT 0,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL,
                deleted_at      TEXT,
                device_id       TEXT NOT NULL
            );
            CREATE TABLE external_client_permissions (
                id                    TEXT PRIMARY KEY,
                client_id             TEXT NOT NULL,
                tool_name             TEXT NOT NULL,
                can_read              INTEGER NOT NULL DEFAULT 0,
                can_write             INTEGER NOT NULL DEFAULT 0,
                requires_confirmation INTEGER NOT NULL DEFAULT 1,
                created_at            TEXT NOT NULL,
                updated_at            TEXT NOT NULL
            );
            CREATE TABLE sync_changelog (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                entity_type TEXT NOT NULL,
                entity_id   TEXT NOT NULL,
                field_name  TEXT NOT NULL,
                old_value   TEXT,
                new_value   TEXT,
                timestamp   TEXT NOT NULL,
                device_id   TEXT NOT NULL,
                operation   TEXT NOT NULL DEFAULT 'update',
                synced_at   TEXT
            );
            CREATE TABLE audit_log (
                id          TEXT PRIMARY KEY,
                actor_type  TEXT NOT NULL,
                actor_id    TEXT,
                action      TEXT NOT NULL,
                entity_type TEXT,
                entity_id   TEXT,
                before_json TEXT,
                after_json  TEXT,
                created_at  TEXT NOT NULL,
                device_id   TEXT NOT NULL
            );
            CREATE INDEX idx_external_client_permissions_client
                ON external_client_permissions (client_id);
            INSERT INTO external_clients
                (id, name, client_type, permission_mode, enabled, created_at, updated_at, device_id)
            VALUES
                ('client-1', 'Legacy Client', 'mcp', 'draft_only', 1,
                 '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z', 'device-a');
            INSERT INTO external_client_permissions
                (id, client_id, tool_name, can_read, can_write,
                 requires_confirmation, created_at, updated_at)
            VALUES
                ('permission-old', 'client-1', 'activity.create', 1, 0, 1,
                 '2026-06-24T08:00:00Z', '2026-06-24T08:00:00Z'),
                ('permission-new', 'client-1', 'activity.create', 0, 0, 1,
                 '2026-06-24T09:00:00Z', '2026-06-24T09:00:00Z');
            PRAGMA user_version = 8;
            "#,
        )
        .expect("v8 permission schema should be created");
    }

    let mut core = CrmCore::open(&path).expect("core should open and run v9");

    assert_eq!(
        core.db
            .schema_version()
            .expect("schema version should read"),
        crate::storage::Database::current_schema_version()
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_external_client_permissions_client_tool'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM external_client_permissions WHERE client_id = 'client-1' AND tool_name = 'activity.create'"
        ),
        1
    );

    let migrated: (String, bool, bool) = core
        .db
        .conn
        .query_row(
            "SELECT id, can_read, can_write FROM external_client_permissions WHERE client_id = 'client-1' AND tool_name = 'activity.create'",
            [],
            |row| {
                let can_read: i64 = row.get(1)?;
                let can_write: i64 = row.get(2)?;
                Ok((row.get(0)?, can_read != 0, can_write != 0))
            },
        )
        .expect("deduplicated permission should query");
    assert_eq!(migrated.0, "permission-new");
    assert!(!migrated.1);
    assert!(!migrated.2);

    let updated = core
        .upsert_external_client_tool_permission("client-1", "activity.create", true, true, true)
        .expect("upsert should update the single effective permission row");
    assert_eq!(updated.id, "permission-new");
    assert!(updated.can_read);
    assert!(updated.can_write);
    assert!(updated.requires_confirmation);
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM external_client_permissions WHERE client_id = 'client-1' AND tool_name = 'activity.create'"
        ),
        1
    );

    let draft = core
        .evaluate_external_client_draft_permission("client-1", "activity.create")
        .expect("deduplicated updated permission should evaluate");
    assert!(draft.allowed);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v3_bridges_legacy_organization_contacts_without_deleting_contacts() {
    let path = std::env::temp_dir().join(format!("900crm-bridge-test-{}", new_uuid()));
    std::fs::create_dir_all(&path).expect("test dir should be created");
    let db_path = path.join("900crm.db");
    {
        let conn = rusqlite::Connection::open(&db_path).expect("legacy db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE contacts (
                id              TEXT PRIMARY KEY NOT NULL,
                contact_type    TEXT NOT NULL DEFAULT 'person',
                first_name      TEXT NOT NULL DEFAULT '',
                last_name       TEXT NOT NULL DEFAULT '',
                org_name        TEXT NOT NULL DEFAULT '',
                email           TEXT NOT NULL DEFAULT '',
                phone           TEXT NOT NULL DEFAULT '',
                address         TEXT NOT NULL DEFAULT '',
                city            TEXT NOT NULL DEFAULT '',
                country         TEXT NOT NULL DEFAULT '',
                org_id          TEXT,
                notes           TEXT NOT NULL DEFAULT '',
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL,
                deleted_at      TEXT,
                device_id       TEXT NOT NULL DEFAULT '',
                organization_id TEXT
            );
            CREATE TABLE organizations (
                id            TEXT PRIMARY KEY,
                name          TEXT NOT NULL,
                email         TEXT,
                phone         TEXT,
                website       TEXT,
                address_line1 TEXT,
                address_line2 TEXT,
                city          TEXT,
                region        TEXT,
                country       TEXT,
                postal_code   TEXT,
                source        TEXT,
                description   TEXT,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                deleted_at    TEXT,
                device_id     TEXT NOT NULL
            );
            CREATE TABLE settings (
                key        TEXT PRIMARY KEY NOT NULL,
                value      TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            INSERT INTO contacts
                (id, contact_type, first_name, last_name, org_name, email, phone,
                 address, city, country, org_id, notes, created_at, updated_at, device_id)
            VALUES
                ('legacy-org', 'organization', '', '', 'Legacy Org', 'org@example.com', '',
                 '1 Main', 'Paris', 'FR', NULL, 'Legacy note', '2026-01-01', '2026-01-02', 'device-a'),
                ('person-1', 'person', 'Amina', 'Diallo', 'Legacy Org', '', '',
                 '', '', '', 'legacy-org', '', '2026-01-03', '2026-01-04', 'device-a');
            PRAGMA user_version = 2;
            "#,
        )
        .expect("legacy schema should be created");
    }

    let core = CrmCore::open(&path).expect("core should open and run v3 bridge");

    let bridged_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM organizations WHERE id = 'legacy-org' AND name = 'Legacy Org' AND source = 'legacy_contact'",
            [],
            |row| row.get(0),
        )
        .expect("bridged organization count should query");
    assert_eq!(bridged_count, 1);

    let linked_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE id = 'person-1' AND org_id = 'legacy-org' AND organization_id = 'legacy-org'",
            [],
            |row| row.get(0),
        )
        .expect("linked contact count should query");
    assert_eq!(linked_count, 1);

    let legacy_contact_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE id = 'legacy-org' AND contact_type = 'organization'",
            [],
            |row| row.get(0),
        )
        .expect("legacy contact count should query");
    assert_eq!(legacy_contact_count, 1);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn normalization_migration_preflight_reports_clean_migrated_database() {
    let (core, path) = open_test_core();

    let report = core
        .normalization_migration_preflight()
        .expect("preflight report should be generated");

    assert_eq!(report.legacy_organization_contacts, 0);
    assert_eq!(report.contacts_with_org_id_missing_organization_id, 0);
    assert_eq!(report.contacts_with_invalid_legacy_org_id_links, 0);
    assert_eq!(
        report.contacts_with_invalid_normalized_organization_id_links,
        0
    );
    assert!(report.backup_restore_baseline_available);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn normalization_migration_preflight_counts_legacy_and_invalid_links() {
    let (core, path) = open_test_core();

    let now = now_iso8601();
    core.db
        .conn
        .execute_batch(&format!(
            r#"
            INSERT INTO contacts
                (id, contact_type, first_name, last_name, org_name, email, phone,
                 address, city, country, org_id, organization_id, notes,
                 created_at, updated_at, device_id)
            VALUES
                ('legacy-org', 'organization', '', '', 'Legacy Org', '', '',
                 '', '', '', NULL, NULL, '', '{now}', '{now}', 'device-a'),
                ('person-with-missing-normalized', 'person', 'Amina', 'Diallo', 'Legacy Org', '', '',
                 '', '', '', 'legacy-org', NULL, '', '{now}', '{now}', 'device-a'),
                ('person-as-org', 'person', 'Wrong', 'Type', '', '', '',
                 '', '', '', NULL, NULL, '', '{now}', '{now}', 'device-a'),
                ('person-with-invalid-org-id', 'person', 'Invalid', 'Legacy Link', '', '', '',
                 '', '', '', 'person-as-org', NULL, '', '{now}', '{now}', 'device-a'),
                ('person-with-invalid-organization-id', 'person', 'Invalid', 'Normalized Link', '', '', '',
                 '', '', '', NULL, 'missing-organization', '', '{now}', '{now}', 'device-a');
            "#
        ))
        .expect("test contacts should be inserted");

    let report = core
        .normalization_migration_preflight()
        .expect("preflight report should be generated");

    assert_eq!(report.legacy_organization_contacts, 1);
    assert_eq!(report.contacts_with_org_id_missing_organization_id, 2);
    assert_eq!(report.contacts_with_invalid_legacy_org_id_links, 1);
    assert_eq!(
        report.contacts_with_invalid_normalized_organization_id_links,
        1
    );
    assert!(report.backup_restore_baseline_available);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn migration_v2_creates_required_readiness_tables() {
    let (core, path) = open_test_core();

    for table in [
        "organizations",
        "audit_log",
        "sync_changelog",
        "external_clients",
        "external_client_permissions",
        "proposed_actions",
        "entity_links",
        "saved_views",
    ] {
        let sql = format!(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '{}'",
            table
        );
        assert_eq!(count(&core, &sql), 1, "missing table {table}");
    }

    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM pragma_table_info('contacts') WHERE name = 'organization_id'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM pragma_table_info('contacts') WHERE name = 'lifecycle'"
        ),
        1
    );
    assert_eq!(
        count(
            &core,
            "SELECT COUNT(*) FROM pragma_table_info('sync_changelog') WHERE name = 'operation'"
        ),
        1
    );

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn contact_lifecycle_defaults_to_customer_and_can_convert_a_lead() {
    let (mut core, path) = open_test_core();

    let created = core
        .create_contact(
            Some("person".to_string()),
            Some("Amara".to_string()),
            Some("Okafor".to_string()),
            None,
            Some("amara@example.test".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
    assert_eq!(created.lifecycle, "customer");

    let lead = core
        .set_contact_lifecycle(&created.id, "lead")
        .expect("contact should become a lead");
    assert_eq!(lead.lifecycle, "lead");

    let customers = core
        .list_contacts(Some(crate::storage::contacts::ContactListParams {
            filter_lifecycle: Some("customer".to_string()),
            ..Default::default()
        }))
        .expect("customer list should load");
    assert!(customers
        .contacts
        .iter()
        .all(|contact| contact.id != created.id));

    let leads = core
        .list_contacts(Some(crate::storage::contacts::ContactListParams {
            filter_lifecycle: Some("lead".to_string()),
            ..Default::default()
        }))
        .expect("lead list should load");
    assert_eq!(leads.total, 1);
    assert_eq!(leads.contacts[0].id, created.id);

    let customer = core
        .set_contact_lifecycle(&created.id, "customer")
        .expect("lead should convert to customer");
    assert_eq!(customer.lifecycle, "customer");

    let invalid = core.set_contact_lifecycle(&created.id, "prospect");
    assert!(matches!(invalid, Err(CrmError::InvalidInput(_))));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn record_owner_can_be_set_and_used_to_filter_contacts() {
    let (mut core, _path) = open_test_core();

    let owned = core
        .create_contact_with_lifecycle(
            Some("person".to_string()),
            Some("Maya".to_string()),
            Some("Owner".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("Samira".to_string()),
        )
        .expect("owned contact");
    assert_eq!(owned.owner.as_deref(), Some("Samira"));

    core.create_contact(
        Some("person".to_string()),
        Some("No".to_string()),
        Some("Owner".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("unowned contact");

    let listed = core
        .list_contacts(Some(crate::storage::contacts::ContactListParams {
            filter_owner: Some("samira".to_string()),
            ..Default::default()
        }))
        .expect("owner filter");
    assert_eq!(listed.total, 1);
    assert_eq!(listed.contacts[0].id, owned.id);

    let deal = core
        .create_deal(
            "Owned rollout".to_string(),
            Some(1000.0),
            Some("USD".to_string()),
            Some("Lead".to_string()),
            Some(10),
            None,
            None,
            None,
            None,
        )
        .expect("deal");
    let deal = core
        .set_deal_owner(&deal.id, Some("Samira"))
        .expect("deal owner");
    assert_eq!(deal.owner.as_deref(), Some("Samira"));

    let organization = core
        .create_organization(
            "Owned Account".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("organization");
    let organization = core
        .set_organization_owner(&organization.id, Some("Samira"))
        .expect("organization owner");
    assert_eq!(organization.owner.as_deref(), Some("Samira"));
}

#[test]
fn create_contact_with_lifecycle_writes_lead_in_one_step() {
    let (mut core, path) = open_test_core();

    let lead = core
        .create_contact_with_lifecycle(
            Some("person".to_string()),
            Some("Amara".to_string()),
            Some("Lead".to_string()),
            None,
            Some("amara.lead@example.test".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            Some("lead".to_string()),
            None,
        )
        .expect("lead should be created in one transaction");
    assert_eq!(lead.lifecycle, "lead");

    let listed = core
        .list_contacts(Some(crate::storage::contacts::ContactListParams {
            filter_lifecycle: Some("lead".to_string()),
            ..Default::default()
        }))
        .expect("lead list should load");
    assert_eq!(listed.total, 1);
    assert_eq!(listed.contacts[0].id, lead.id);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn entity_links_store_http_urls_and_local_paths_without_copying_files() {
    let (mut core, path) = open_test_core();

    let contact = core
        .create_contact(
            Some("person".to_string()),
            Some("Nia".to_string()),
            Some("Mensah".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");

    let website = core
        .create_entity_link(
            "contact".to_string(),
            contact.id.clone(),
            Some("Quote sheet".to_string()),
            "url".to_string(),
            "https://northstar.example/quote".to_string(),
        )
        .expect("url link should be created");
    assert_eq!(website.kind, "url");
    assert_eq!(website.target, "https://northstar.example/quote");

    let file = core
        .create_entity_link(
            "contact".to_string(),
            contact.id.clone(),
            None,
            "path".to_string(),
            "/tmp/900crm-quote.pdf".to_string(),
        )
        .expect("path link should be created");
    assert_eq!(file.kind, "path");
    assert_eq!(file.title, "900crm-quote.pdf");

    let listed = core
        .list_entity_links("contact".to_string(), contact.id.clone())
        .expect("links should list");
    assert_eq!(listed.len(), 2);

    assert!(core
        .create_entity_link(
            "contact".to_string(),
            contact.id.clone(),
            None,
            "url".to_string(),
            "javascript:alert(1)".to_string(),
        )
        .is_err());
    assert!(core
        .create_entity_link(
            "contact".to_string(),
            contact.id,
            None,
            "path".to_string(),
            "https://evil.example/file".to_string(),
        )
        .is_err());

    core.delete_entity_link(&website.id)
        .expect("url link should delete");
    let remaining = core
        .list_entity_links("contact".to_string(), listed[0].entity_id.clone())
        .expect("remaining links should list");
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, file.id);

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn saved_views_store_named_contact_filters() {
    let (mut core, path) = open_test_core();

    let created = core.create_saved_view(
        "contact".to_string(),
        " New leads ".to_string(),
        r#"{"lifecycle":"lead","search":"","unknown":"nope"}"#.to_string(),
    );
    assert!(created.is_err());

    let view = core
        .create_saved_view(
            "contact".to_string(),
            " New leads ".to_string(),
            r#"{"lifecycle":"lead","search":"  "}"#.to_string(),
        )
        .expect("saved view should be created");
    assert_eq!(view.name, "New leads");
    assert_eq!(view.filters_json, r#"{"lifecycle":"lead"}"#);

    let duplicate = core.create_saved_view(
        "contact".to_string(),
        "new leads".to_string(),
        r#"{"lifecycle":"customer"}"#.to_string(),
    );
    assert!(matches!(duplicate, Err(CrmError::InvalidInput(_))));

    let listed = core
        .list_saved_views("contact".to_string())
        .expect("saved views should list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, view.id);

    core.delete_saved_view(&view.id)
        .expect("saved view should delete");
    let remaining = core
        .list_saved_views("contact".to_string())
        .expect("empty saved views should list");
    assert!(remaining.is_empty());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn saved_views_store_named_organization_filters() {
    let (mut core, path) = open_test_core();

    let view = core
        .create_saved_view(
            "organization".to_string(),
            "Kenya accounts".to_string(),
            r#"{"country":"Kenya","search":"clinic"}"#.to_string(),
        )
        .expect("organization saved view should be created");
    assert_eq!(view.entity_type, "organization");
    assert!(view.filters_json.contains("Kenya"));
    assert!(view.filters_json.contains("clinic"));

    let listed = core
        .list_saved_views("organization".to_string())
        .expect("organization views should list");
    assert_eq!(listed.len(), 1);
    assert!(core
        .list_saved_views("contact".to_string())
        .expect("contact views stay separate")
        .is_empty());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn saved_views_store_named_deal_filters() {
    let (mut core, path) = open_test_core();

    let view = core
        .create_saved_view(
            "deal".to_string(),
            "Clinic rollouts".to_string(),
            r#"{"search":"clinic","custom_field_query":"solar","attention":"needsFollowUp"}"#
                .to_string(),
        )
        .expect("deal saved view should be created");
    assert_eq!(view.entity_type, "deal");
    assert!(view.filters_json.contains("clinic"));
    assert!(view.filters_json.contains("solar"));
    assert!(view.filters_json.contains("needsFollowUp"));

    let listed = core
        .list_saved_views("deal".to_string())
        .expect("deal views should list");
    assert_eq!(listed.len(), 1);
    assert!(core
        .list_saved_views("contact".to_string())
        .expect("contact views stay separate")
        .is_empty());
    assert!(core
        .list_saved_views("organization".to_string())
        .expect("organization views stay separate")
        .is_empty());

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn saved_views_store_named_activity_filters() {
    let (mut core, path) = open_test_core();

    let view = core
        .create_saved_view(
            "activity".to_string(),
            "Overdue calls".to_string(),
            r#"{"type":"call","status":"overdue","bucket":"today","custom_field_query":"clinic"}"#
                .to_string(),
        )
        .expect("activity saved view should be created");
    assert_eq!(view.entity_type, "activity");
    assert!(view.filters_json.contains("call"));
    assert!(view.filters_json.contains("overdue"));
    assert!(view.filters_json.contains("today"));
    assert!(view.filters_json.contains("clinic"));

    let listed = core
        .list_saved_views("activity".to_string())
        .expect("activity views should list");
    assert_eq!(listed.len(), 1);
    assert!(core
        .list_saved_views("contact".to_string())
        .expect("contact views stay separate")
        .is_empty());
    assert!(core
        .list_saved_views("organization".to_string())
        .expect("organization views stay separate")
        .is_empty());
    assert!(core
        .list_saved_views("deal".to_string())
        .expect("deal views stay separate")
        .is_empty());

    let invalid = core.create_saved_view(
        "activity".to_string(),
        "Bad bucket".to_string(),
        r#"{"bucket":"yesterday"}"#.to_string(),
    );
    assert!(matches!(invalid, Err(CrmError::InvalidInput(_))));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn saved_views_store_named_report_focus() {
    let (mut core, path) = open_test_core();

    let view = core
        .create_saved_view(
            "report".to_string(),
            "Stale deals".to_string(),
            r#"{"focus":"stale"}"#.to_string(),
        )
        .expect("report saved view should be created");
    assert_eq!(view.entity_type, "report");
    assert!(view.filters_json.contains("stale"));

    let listed = core
        .list_saved_views("report".to_string())
        .expect("report views should list");
    assert_eq!(listed.len(), 1);
    assert!(core
        .list_saved_views("deal".to_string())
        .expect("deal views stay separate")
        .is_empty());
    assert!(core
        .list_saved_views("activity".to_string())
        .expect("activity views stay separate")
        .is_empty());

    let invalid = core.create_saved_view(
        "report".to_string(),
        "Bad focus".to_string(),
        r#"{"focus":"forecast"}"#.to_string(),
    );
    assert!(matches!(invalid, Err(CrmError::InvalidInput(_))));

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn merge_contacts_reassigns_related_records_and_soft_deletes_source() {
    let (mut core, path) = open_test_core();

    let target = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Target".to_string()),
            None,
            Some("ada.target@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("target contact should be created");
    let source = core
        .create_contact(
            Some("person".to_string()),
            Some("Ada".to_string()),
            Some("Source".to_string()),
            None,
            Some("ada.source@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("source contact should be created");

    let deal = core
        .create_deal(
            "Clinic expansion".to_string(),
            Some(2500.0),
            None,
            Some("Proposal".to_string()),
            None,
            None,
            Some(source.id.clone()),
            None,
            None,
        )
        .expect("deal should be created");
    let activity = core
        .create_activity(
            "task".to_string(),
            "Follow up".to_string(),
            None,
            None,
            Some(source.id.clone()),
            None,
        )
        .expect("activity should be created");
    let note = core
        .create_note(
            "contact".to_string(),
            source.id.clone(),
            "Source onboarding note".to_string(),
        )
        .expect("note should be created");
    let tag = core
        .create_tag("Clinic Priority".to_string(), None)
        .expect("tag should be created");
    core.apply_tag_to_entity("contact".to_string(), source.id.clone(), tag.id.clone())
        .expect("tag should apply to source");
    let field = core
        .create_custom_field_def(
            "contact".to_string(),
            "VIP Tier".to_string(),
            "text".to_string(),
            None,
            Some(0),
        )
        .expect("contact custom field should be created");
    core.set_custom_field_value(field.id.clone(), source.id.clone(), "Gold".to_string())
        .expect("source custom field value should be set");

    let merged = core
        .merge_contacts(&target.id, &source.id)
        .expect("contacts should merge");
    assert_eq!(merged.id, target.id);

    assert!(matches!(
        core.get_contact(&source.id),
        Err(CrmError::NotFound(_))
    ));
    let listed = core
        .list_contacts(None)
        .expect("active contacts should list");
    assert_eq!(listed.total, 1);
    assert_eq!(listed.contacts.len(), 1);
    assert_eq!(listed.contacts[0].id, target.id);

    let reassigned_deal = core.get_deal(&deal.id).expect("deal should load");
    assert_eq!(
        reassigned_deal.contact_id.as_deref(),
        Some(target.id.as_str())
    );

    let reassigned_activity = core
        .get_activity(&activity.id)
        .expect("activity should load");
    assert_eq!(
        reassigned_activity.contact_id.as_deref(),
        Some(target.id.as_str())
    );

    let reassigned_note = core.get_note(&note.id).expect("note should load");
    assert_eq!(reassigned_note.entity_id, target.id);

    let target_tags = core
        .list_tags_for_entity("contact".to_string(), target.id.clone())
        .expect("target tags should list");
    assert_eq!(target_tags.len(), 1);
    assert_eq!(target_tags[0].id, tag.id);
    let tag_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'contact' AND entity_id = ?1 AND tag_id = ?2 AND deleted_at IS NULL",
            params![target.id, tag.id],
            |row| row.get(0),
        )
        .expect("target tag link count should query");
    assert_eq!(tag_link_count, 1);
    let source_tag_link_count: i64 = core
        .db
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tag_links WHERE entity_type = 'contact' AND entity_id = ?1 AND deleted_at IS NULL",
            params![source.id],
            |row| row.get(0),
        )
        .expect("source tag link count should query");
    assert_eq!(source_tag_link_count, 0);

    let custom_values = core
        .list_custom_field_values_for_type("contact")
        .expect("custom field values should list");
    assert_eq!(custom_values.len(), 1);
    assert_eq!(custom_values[0].entity_id, target.id);
    assert_eq!(custom_values[0].field_def_id, field.id);
    assert_eq!(custom_values[0].value, "Gold");

    drop(core);
    let _ = std::fs::remove_dir_all(path);
}
