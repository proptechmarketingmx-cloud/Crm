//! 900CRM desktop shell.
//!
//! The desktop crate owns Tauri setup and IPC command registration only.
//! Business logic, SQLite storage, audit, and MCP-readiness foundations live in
//! the Tauri-independent `crm-core` crate.

use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crm_core::CrmCore;
use tauri::Manager;

pub mod commands;
pub mod state;

pub use state::AppState;

pub fn run() {
    let default_log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(default_log_level))
        .init();

    log::info!(
        "900CRM desktop v{} starting ({})",
        env!("CARGO_PKG_VERSION"),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");
            let core = CrmCore::open(&app_data_dir).expect("Failed to initialize 900CRM core");

            log::info!("App data directory: {}", app_data_dir.display());
            log::info!("Device id: {}", core.device_id());
            log::info!(
                "CrmCore initialized with {} default pipeline stages",
                core.default_stage_count()
            );

            app.manage(AppState {
                core: Mutex::new(Some(core)),
                data_dir: app_data_dir,
                needs_reopen: AtomicBool::new(false),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::contact_commands::create_contact,
            commands::contact_commands::get_contact,
            commands::contact_commands::list_contacts,
            commands::contact_commands::update_contact,
            commands::contact_commands::delete_contact,
            commands::contact_commands::restore_contact,
            commands::contact_commands::search_contacts,
            commands::contact_commands::list_contact_duplicate_candidates,
            commands::contact_commands::merge_contacts,
            commands::contact_commands::set_contact_lifecycle,
            commands::organization_commands::create_organization,
            commands::organization_commands::get_organization,
            commands::organization_commands::list_organizations,
            commands::organization_commands::update_organization,
            commands::organization_commands::delete_organization,
            commands::organization_commands::link_contact_to_organization,
            commands::link_commands::create_entity_link,
            commands::link_commands::list_entity_links,
            commands::link_commands::update_entity_link,
            commands::link_commands::delete_entity_link,
            commands::saved_view_commands::create_saved_view,
            commands::saved_view_commands::list_saved_views,
            commands::saved_view_commands::update_saved_view,
            commands::saved_view_commands::delete_saved_view,
            commands::note_commands::create_note,
            commands::note_commands::get_note,
            commands::note_commands::list_notes_for_entity,
            commands::note_commands::update_note,
            commands::note_commands::delete_note,
            commands::tag_commands::create_tag,
            commands::tag_commands::get_tag,
            commands::tag_commands::list_tags,
            commands::tag_commands::update_tag,
            commands::tag_commands::delete_tag,
            commands::tag_commands::apply_tag_to_entity,
            commands::tag_commands::remove_tag_from_entity,
            commands::tag_commands::list_tags_for_entity,
            commands::deal_commands::create_deal,
            commands::deal_commands::get_deal,
            commands::deal_commands::list_deals,
            commands::deal_commands::list_deals_by_stage,
            commands::deal_commands::update_deal,
            commands::deal_commands::move_deal_stage,
            commands::deal_commands::delete_deal,
            commands::deal_commands::link_deal_to_organization,
            commands::deal_commands::add_deal_contact,
            commands::deal_commands::remove_deal_contact,
            commands::deal_commands::list_deal_contacts,
            commands::deal_commands::get_pipeline_summary,
            commands::activity_commands::create_activity,
            commands::activity_commands::get_activity,
            commands::activity_commands::list_activities,
            commands::activity_commands::list_activities_for_contact,
            commands::activity_commands::list_activities_for_deal,
            commands::activity_commands::list_activities_for_deals,
            commands::activity_commands::list_upcoming_activities,
            commands::activity_commands::mark_activity_complete,
            commands::activity_commands::mark_activity_incomplete,
            commands::activity_commands::update_activity,
            commands::activity_commands::delete_activity,
            commands::activity_commands::list_activity_links,
            commands::activity_commands::list_activity_links_for_activities,
            commands::activity_commands::add_activity_link,
            commands::activity_commands::remove_activity_link,
            commands::dashboard_commands::get_dashboard_stats,
            commands::custom_field_commands::list_custom_field_defs,
            commands::custom_field_commands::create_custom_field_def,
            commands::custom_field_commands::update_custom_field_def,
            commands::custom_field_commands::delete_custom_field_def,
            commands::custom_field_commands::set_custom_field_value,
            commands::custom_field_commands::list_custom_field_values,
            commands::custom_field_commands::list_custom_field_values_for_type,
            commands::report_commands::get_pipeline_conversion_report,
            commands::report_commands::get_activity_funnel_report,
            commands::search_commands::global_search,
            commands::audit_pending_commands::list_recent_audit_log,
            commands::audit_pending_commands::list_pending_proposed_actions,
            commands::audit_pending_commands::approve_proposed_action,
            commands::audit_pending_commands::reject_proposed_action,
            commands::external_client_commands::list_external_clients,
            commands::external_client_commands::create_external_client_placeholder,
            commands::external_client_commands::update_external_client_activation,
            commands::external_client_commands::list_external_client_permissions,
            commands::external_client_commands::upsert_external_client_tool_permission,
            commands::external_client_commands::evaluate_external_client_tool_read_permission,
            commands::external_client_commands::evaluate_external_client_draft_permission,
            commands::backup_commands::create_local_backup,
            commands::backup_commands::validate_local_backup,
            commands::backup_commands::restore_local_backup_to_app_data,
            commands::import_export::read_import_text,
            commands::import_export::validate_open_path,
            commands::import_export::write_export_text,
            commands::import_export::rollback_completed_import,
            commands::import_export::import_contacts_csv,
            commands::import_export::import_contacts_csv_with_mapping,
            commands::import_export::import_contacts_json,
            commands::import_export::import_contacts_json_with_mapping,
            commands::import_export::preview_contacts_json_import,
            commands::import_export::preflight_contacts_csv_import,
            commands::import_export::preflight_contacts_csv_import_with_mapping,
            commands::import_export::preflight_contacts_json_import,
            commands::import_export::preflight_contacts_json_import_with_mapping,
            commands::import_export::export_contacts_csv,
            commands::import_export::export_contacts_json,
            commands::import_export::export_audit_log_csv,
            commands::import_export::export_audit_log_json,
            commands::import_export::export_proposed_actions_csv,
            commands::import_export::export_proposed_actions_json,
            commands::import_export::export_external_clients_csv,
            commands::import_export::export_external_clients_json,
            commands::import_export::export_external_client_permissions_csv,
            commands::import_export::export_external_client_permissions_json,
            commands::import_export::import_deals_csv,
            commands::import_export::import_deals_csv_with_mapping,
            commands::import_export::import_deals_json,
            commands::import_export::import_deals_json_with_mapping,
            commands::import_export::preview_deals_json_import,
            commands::import_export::preflight_deals_csv_import,
            commands::import_export::preflight_deals_csv_import_with_mapping,
            commands::import_export::preflight_deals_json_import,
            commands::import_export::preflight_deals_json_import_with_mapping,
            commands::import_export::export_deals_csv,
            commands::import_export::export_deals_json,
            commands::import_export::import_activities_csv,
            commands::import_export::import_activities_csv_with_mapping,
            commands::import_export::import_activities_json,
            commands::import_export::import_activities_json_with_mapping,
            commands::import_export::preview_activities_json_import,
            commands::import_export::preflight_activities_csv_import,
            commands::import_export::preflight_activities_csv_import_with_mapping,
            commands::import_export::preflight_activities_json_import,
            commands::import_export::preflight_activities_json_import_with_mapping,
            commands::import_export::export_activities_csv,
            commands::import_export::export_activities_json,
            commands::import_export::import_notes_csv,
            commands::import_export::import_notes_csv_with_mapping,
            commands::import_export::import_notes_json,
            commands::import_export::import_notes_json_with_mapping,
            commands::import_export::preview_notes_json_import,
            commands::import_export::preflight_notes_csv_import,
            commands::import_export::preflight_notes_csv_import_with_mapping,
            commands::import_export::preflight_notes_json_import,
            commands::import_export::preflight_notes_json_import_with_mapping,
            commands::import_export::export_notes_csv,
            commands::import_export::export_notes_json,
            commands::import_export::import_tag_definitions_csv,
            commands::import_export::import_tag_definitions_csv_with_mapping,
            commands::import_export::import_tag_definitions_json,
            commands::import_export::import_tag_definitions_json_with_mapping,
            commands::import_export::preview_tag_definitions_json_import,
            commands::import_export::preflight_tag_definitions_csv_import,
            commands::import_export::preflight_tag_definitions_csv_import_with_mapping,
            commands::import_export::preflight_tag_definitions_json_import,
            commands::import_export::preflight_tag_definitions_json_import_with_mapping,
            commands::import_export::export_tag_definitions_csv,
            commands::import_export::export_tag_definitions_json,
            commands::import_export::import_custom_field_definitions_csv,
            commands::import_export::import_custom_field_definitions_csv_with_mapping,
            commands::import_export::import_custom_field_definitions_json,
            commands::import_export::import_custom_field_definitions_json_with_mapping,
            commands::import_export::preview_custom_field_definitions_json_import,
            commands::import_export::preflight_custom_field_definitions_csv_import,
            commands::import_export::preflight_custom_field_definitions_csv_import_with_mapping,
            commands::import_export::preflight_custom_field_definitions_json_import,
            commands::import_export::preflight_custom_field_definitions_json_import_with_mapping,
            commands::import_export::export_custom_field_definitions_csv,
            commands::import_export::export_custom_field_definitions_json,
            commands::import_export::import_tag_links_csv,
            commands::import_export::import_tag_links_csv_with_mapping,
            commands::import_export::import_tag_links_json,
            commands::import_export::import_tag_links_json_with_mapping,
            commands::import_export::preview_tag_links_json_import,
            commands::import_export::preflight_tag_links_csv_import,
            commands::import_export::preflight_tag_links_csv_import_with_mapping,
            commands::import_export::preflight_tag_links_json_import,
            commands::import_export::preflight_tag_links_json_import_with_mapping,
            commands::import_export::export_tag_links_csv,
            commands::import_export::export_tag_links_json,
            commands::import_export::import_organizations_csv,
            commands::import_export::import_organizations_csv_with_mapping,
            commands::import_export::import_organizations_json,
            commands::import_export::import_organizations_json_with_mapping,
            commands::import_export::preview_organizations_json_import,
            commands::import_export::preflight_organizations_csv_import,
            commands::import_export::preflight_organizations_csv_import_with_mapping,
            commands::import_export::preflight_organizations_json_import,
            commands::import_export::preflight_organizations_json_import_with_mapping,
            commands::import_export::export_organizations_csv,
            commands::import_export::export_organizations_json,
            commands::settings_commands::get_settings,
            commands::settings_commands::get_setting,
            commands::settings_commands::update_setting,
            commands::email_commands::test_email_server_connection,
            commands::sync_commands::get_sync_status,
            commands::sync_commands::trigger_sync,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running 900CRM application");
}
