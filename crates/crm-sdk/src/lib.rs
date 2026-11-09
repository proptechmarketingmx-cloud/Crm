//! Narrow local SDK facade for future 900CRM integrations.
//!
//! This crate intentionally exposes no listener, credentials, network behavior,
//! direct write methods, or proposed-action decisions. It is a local facade over
//! [`crm_core::CrmCore`] so future runtime packages can call reviewed read and
//! draft core services without raw SQL.

use std::path::Path;

use crm_core::{
    errors::CrmError,
    result::CrmResult,
    search::SearchResult,
    storage::{
        activities::Activity, contacts::ContactListResult, deals::Deal,
        organizations::Organization, proposed_actions::ProposedAction,
    },
    CrmCore,
};
use serde::Serialize;

pub use crm_core::{errors::CrmError as SdkError, storage::contacts::ContactListParams};

/// Tool name for listing contacts through the read-only SDK.
pub const CONTACTS_LIST_TOOL: &str = "contacts.list";
/// Tool name for listing organizations through the read-only SDK.
pub const ORGANIZATIONS_LIST_TOOL: &str = "organizations.list";
/// Tool name for listing deals through the read-only SDK.
pub const DEALS_LIST_TOOL: &str = "deals.list";
/// Tool name for listing activities through the read-only SDK.
pub const ACTIVITIES_LIST_TOOL: &str = "activities.list";
/// Tool name for global search through the read-only SDK.
pub const SEARCH_GLOBAL_TOOL: &str = "search.global";
/// Tool name for creating a pending activity proposed action through the SDK.
pub const CREATE_ACTIVITY_DRAFT_TOOL: &str = "create_activity_draft";

/// Initial read-only tool names exported for future MCP/runtime reuse.
pub const INITIAL_READ_TOOL_NAMES: &[&str] = &[
    CONTACTS_LIST_TOOL,
    ORGANIZATIONS_LIST_TOOL,
    DEALS_LIST_TOOL,
    ACTIVITIES_LIST_TOOL,
    SEARCH_GLOBAL_TOOL,
];

/// Initial draft tool names exported for future MCP/runtime reuse.
pub const INITIAL_DRAFT_TOOL_NAMES: &[&str] = &[CREATE_ACTIVITY_DRAFT_TOOL];

/// Local SDK facade over [`CrmCore`].
pub struct ReadOnlyCrmSdk {
    core: CrmCore,
    external_client_id: String,
}

/// Product-facing alias for the reviewed SDK surface.
pub type CrmSdk = ReadOnlyCrmSdk;

impl ReadOnlyCrmSdk {
    /// Opens the local CRM data store for a previously reviewed external client.
    pub fn open(
        app_data_dir: impl AsRef<Path>,
        external_client_id: impl Into<String>,
    ) -> CrmResult<Self> {
        let external_client_id = normalize_external_client_id(external_client_id.into())?;
        Ok(Self {
            core: CrmCore::open(app_data_dir.as_ref())?,
            external_client_id,
        })
    }

    /// Returns the external client id used for permission checks.
    pub fn external_client_id(&self) -> &str {
        &self.external_client_id
    }

    /// Lists contacts after verifying `contacts.list` read permission.
    pub fn contacts_list(&self, params: Option<ContactListParams>) -> CrmResult<ContactListResult> {
        self.require_read_permission(CONTACTS_LIST_TOOL)?;
        let result = self.core.list_contacts(params)?;
        self.record_read_result(CONTACTS_LIST_TOOL, count_as_u32(result.contacts.len()))?;
        Ok(result)
    }

    /// Lists organizations after verifying `organizations.list` read permission.
    pub fn organizations_list(&self) -> CrmResult<Vec<Organization>> {
        self.require_read_permission(ORGANIZATIONS_LIST_TOOL)?;
        let result = self.core.list_organizations()?;
        self.record_read_result(ORGANIZATIONS_LIST_TOOL, count_as_u32(result.len()))?;
        Ok(result)
    }

    /// Lists deals after verifying `deals.list` read permission.
    pub fn deals_list(&self) -> CrmResult<Vec<Deal>> {
        self.require_read_permission(DEALS_LIST_TOOL)?;
        let result = self.core.list_deals()?;
        self.record_read_result(DEALS_LIST_TOOL, count_as_u32(result.len()))?;
        Ok(result)
    }

    /// Lists activities after verifying `activities.list` read permission.
    pub fn activities_list(&self) -> CrmResult<Vec<Activity>> {
        self.require_read_permission(ACTIVITIES_LIST_TOOL)?;
        let result = self.core.list_activities()?;
        self.record_read_result(ACTIVITIES_LIST_TOOL, count_as_u32(result.len()))?;
        Ok(result)
    }

    /// Runs global search after verifying `search.global` read permission.
    pub fn search_global(&self, query: &str, limit: Option<u32>) -> CrmResult<Vec<SearchResult>> {
        self.require_read_permission(SEARCH_GLOBAL_TOOL)?;
        let result = self.core.global_search(query, limit)?;
        self.record_read_result(SEARCH_GLOBAL_TOOL, count_as_u32(result.len()))?;
        Ok(result)
    }

    /// Creates a pending activity proposed action after draft permission checks.
    ///
    /// This does not create an activity, approve a proposed action, or execute a
    /// proposed action. It preserves the documented core draft input JSON shape
    /// and delegates permission/audit behavior to `crm-core`.
    pub fn create_activity_draft(
        &mut self,
        params: CreateActivityDraftParams,
    ) -> CrmResult<ProposedAction> {
        let params = params.normalized()?;
        let input_json = serde_json::to_string(&params)?;
        self.core.create_external_proposed_action_stub(
            Some(self.external_client_id.clone()),
            CREATE_ACTIVITY_DRAFT_TOOL.to_string(),
            CREATE_ACTIVITY_DRAFT_TOOL.to_string(),
            Some("activity".to_string()),
            None,
            input_json,
            None,
        )
    }

    fn require_read_permission(&self, tool_name: &str) -> CrmResult<()> {
        let evaluation = self
            .core
            .evaluate_external_client_tool_read_permission(&self.external_client_id, tool_name)?;
        if evaluation.allowed {
            return Ok(());
        }

        Err(CrmError::InvalidInput(format!(
            "External client '{}' may not read tool '{}': {}",
            self.external_client_id,
            tool_name,
            evaluation.reason.as_str()
        )))
    }

    fn record_read_result(&self, tool_name: &str, result_count: u32) -> CrmResult<()> {
        self.core.record_external_client_tool_result(
            &self.external_client_id,
            tool_name,
            "read",
            result_count,
            None,
            None,
        )?;
        Ok(())
    }
}

/// Documented input shape for the `create_activity_draft` pending-action tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateActivityDraftParams {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_at: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub linked_entities: Vec<CreateActivityDraftLinkedEntityParam>,
}

impl CreateActivityDraftParams {
    fn normalized(self) -> CrmResult<Self> {
        Ok(Self {
            title: required_draft_string("title", &self.title)?,
            activity_type: optional_draft_string(self.activity_type),
            description: optional_draft_string(self.description),
            due_at: optional_draft_string(self.due_at),
            linked_entities: self
                .linked_entities
                .into_iter()
                .map(CreateActivityDraftLinkedEntityParam::normalized)
                .collect::<CrmResult<Vec<_>>>()?,
        })
    }
}

/// Linked entity reference inside a `create_activity_draft` input payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateActivityDraftLinkedEntityParam {
    pub entity_type: String,
    pub entity_id: String,
}

impl CreateActivityDraftLinkedEntityParam {
    fn normalized(self) -> CrmResult<Self> {
        let entity_type =
            required_draft_string("linked_entities[].entity_type", &self.entity_type)?;
        if !matches!(entity_type.as_str(), "contact" | "organization" | "deal") {
            return Err(CrmError::InvalidInput(format!(
                "create_activity_draft linked_entities[].entity_type '{}' is unsupported",
                entity_type
            )));
        }

        Ok(Self {
            entity_type,
            entity_id: required_draft_string("linked_entities[].entity_id", &self.entity_id)?,
        })
    }
}

/// Reports whether the SDK has real client behavior.
pub fn is_implemented() -> bool {
    true
}

fn normalize_external_client_id(external_client_id: String) -> CrmResult<String> {
    let external_client_id = external_client_id.trim();
    if external_client_id.is_empty() {
        return Err(CrmError::InvalidInput(
            "External client id is required".to_string(),
        ));
    }

    Ok(external_client_id.to_string())
}

fn required_draft_string(field: &str, value: &str) -> CrmResult<String> {
    optional_draft_string(Some(value.to_string()))
        .ok_or_else(|| CrmError::InvalidInput(format!("create_activity_draft {field} is required")))
}

fn optional_draft_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|trimmed| !trimmed.is_empty())
}

fn count_as_u32(count: usize) -> u32 {
    count.min(u32::MAX as usize) as u32
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crm_core::{
        errors::CrmError,
        storage::{audit::AuditLogEntry, external_clients::ExternalClient},
        CrmCore,
    };

    use super::{
        is_implemented, CreateActivityDraftLinkedEntityParam, CreateActivityDraftParams,
        ReadOnlyCrmSdk, CONTACTS_LIST_TOOL, CREATE_ACTIVITY_DRAFT_TOOL, SEARCH_GLOBAL_TOOL,
    };

    static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn sdk_reports_implemented_behavior() {
        assert!(is_implemented());
    }

    #[test]
    fn disabled_external_client_cannot_read_and_records_audit_evidence() {
        let app_data_dir = test_app_data_dir();
        let mut core = open_core(&app_data_dir);
        let client = core
            .create_external_client_placeholder("Disabled SDK Client", "mcp")
            .expect("client placeholder should be created");
        core.create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Hassan".to_string()),
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
        drop(core);

        let sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should open");
        let err = sdk
            .contacts_list(None)
            .expect_err("disabled client should not read contacts");
        assert_invalid_input_contains(err, "client_disabled");
        drop(sdk);

        assert_permission_audit(
            &app_data_dir,
            &client.id,
            CONTACTS_LIST_TOOL,
            false,
            "denied",
            "evaluate_external_client_read_permission",
            "read",
        );
        assert_no_tool_result_audit(&app_data_dir, &client.id, CONTACTS_LIST_TOOL);
        cleanup(app_data_dir);
    }

    #[test]
    fn read_only_client_without_explicit_permission_cannot_read() {
        let app_data_dir = test_app_data_dir();
        let mut core = open_core(&app_data_dir);
        let client = create_read_only_client(&mut core, "Unpermitted SDK Client");
        drop(core);

        let sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should open");
        let err = sdk
            .contacts_list(None)
            .expect_err("missing tool permission should deny contacts");
        assert_invalid_input_contains(err, "missing_tool_permission");
        drop(sdk);

        assert_permission_audit(
            &app_data_dir,
            &client.id,
            CONTACTS_LIST_TOOL,
            false,
            "denied",
            "evaluate_external_client_read_permission",
            "read",
        );
        assert_no_tool_result_audit(&app_data_dir, &client.id, CONTACTS_LIST_TOOL);
        cleanup(app_data_dir);
    }

    #[test]
    fn read_only_client_with_explicit_permission_can_read_contacts() {
        let app_data_dir = test_app_data_dir();
        let mut core = open_core(&app_data_dir);
        let contact = core
            .create_contact(
                Some("person".to_string()),
                Some("Amina".to_string()),
                Some("Hassan".to_string()),
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
        let client = create_read_only_client(&mut core, "Allowed SDK Client");
        core.upsert_external_client_tool_permission(
            &client.id,
            CONTACTS_LIST_TOOL,
            true,
            false,
            false,
        )
        .expect("contacts list permission should upsert");
        drop(core);

        let sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should open");
        let contacts = sdk
            .contacts_list(None)
            .expect("permitted client should list contacts");
        assert_eq!(contacts.total, 1);
        assert_eq!(contacts.contacts.len(), 1);
        assert_eq!(contacts.contacts[0].id, contact.id);
        drop(sdk);

        assert_permission_audit(
            &app_data_dir,
            &client.id,
            CONTACTS_LIST_TOOL,
            true,
            "allowed",
            "evaluate_external_client_read_permission",
            "read",
        );
        assert_tool_result_audit(&app_data_dir, &client.id, CONTACTS_LIST_TOOL, 1);
        cleanup(app_data_dir);
    }

    #[test]
    fn global_search_uses_the_sdk_permission_boundary() {
        let app_data_dir = test_app_data_dir();
        let mut core = open_core(&app_data_dir);
        core.create_contact(
            Some("person".to_string()),
            Some("Amina".to_string()),
            Some("Searchable".to_string()),
            None,
            Some("searchable@example.com".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("contact should be created");
        let client = create_read_only_client(&mut core, "Search SDK Client");
        core.upsert_external_client_tool_permission(
            &client.id,
            CONTACTS_LIST_TOOL,
            true,
            false,
            false,
        )
        .expect("contacts list permission should upsert");
        drop(core);

        let sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should open");
        let err = sdk
            .search_global("Amina", Some(10))
            .expect_err("contacts permission must not allow global search");
        assert_invalid_input_contains(err, "missing_tool_permission");
        drop(sdk);
        assert_permission_audit(
            &app_data_dir,
            &client.id,
            SEARCH_GLOBAL_TOOL,
            false,
            "denied",
            "evaluate_external_client_read_permission",
            "read",
        );
        assert_no_tool_result_audit(&app_data_dir, &client.id, SEARCH_GLOBAL_TOOL);

        let mut core = open_core(&app_data_dir);
        core.upsert_external_client_tool_permission(
            &client.id,
            SEARCH_GLOBAL_TOOL,
            true,
            false,
            false,
        )
        .expect("search permission should upsert");
        drop(core);

        let sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should reopen");
        let results = sdk
            .search_global("Amina", Some(10))
            .expect("permitted client should search globally");
        assert!(
            results.iter().any(|result| result.title.contains("Amina")),
            "global search should return the seeded contact"
        );
        drop(sdk);
        assert_permission_audit(
            &app_data_dir,
            &client.id,
            SEARCH_GLOBAL_TOOL,
            true,
            "allowed",
            "evaluate_external_client_read_permission",
            "read",
        );
        assert_tool_result_audit(&app_data_dir, &client.id, SEARCH_GLOBAL_TOOL, 1);
        cleanup(app_data_dir);
    }

    #[test]
    fn draft_only_client_with_confirmed_permission_creates_pending_activity_draft() {
        let app_data_dir = test_app_data_dir();
        let mut core = open_core(&app_data_dir);
        let client = create_draft_only_client(&mut core, "Allowed Draft SDK Client");
        core.upsert_external_client_tool_permission(
            &client.id,
            CREATE_ACTIVITY_DRAFT_TOOL,
            false,
            true,
            true,
        )
        .expect("create activity draft permission should upsert");
        drop(core);

        let mut sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should open");
        let proposed_action = sdk
            .create_activity_draft(CreateActivityDraftParams {
                title: "  Call Amina  ".to_string(),
                activity_type: Some(" call ".to_string()),
                description: Some("Confirm next steps".to_string()),
                due_at: Some("2026-06-25T09:00:00Z".to_string()),
                linked_entities: vec![CreateActivityDraftLinkedEntityParam {
                    entity_type: "organization".to_string(),
                    entity_id: "org-1".to_string(),
                }],
            })
            .expect("permitted draft client should create pending proposed action");
        drop(sdk);

        assert_eq!(
            proposed_action.client_id.as_deref(),
            Some(client.id.as_str())
        );
        assert_eq!(proposed_action.tool_name, CREATE_ACTIVITY_DRAFT_TOOL);
        assert_eq!(proposed_action.action_type, CREATE_ACTIVITY_DRAFT_TOOL);
        assert_eq!(proposed_action.status, "pending");
        assert_eq!(proposed_action.approved_at, None);
        assert_eq!(proposed_action.rejected_at, None);
        assert_eq!(proposed_action.executed_at, None);
        let input: serde_json::Value =
            serde_json::from_str(&proposed_action.input_json).expect("input JSON should parse");
        assert_eq!(input["title"], "Call Amina");
        assert_eq!(input["activity_type"], "call");
        assert_eq!(input["linked_entities"][0]["entity_type"], "organization");

        let core = open_core(&app_data_dir);
        assert_eq!(
            core.list_pending_proposed_actions()
                .expect("pending actions should list")
                .len(),
            1
        );
        assert!(
            core.list_activities()
                .expect("activities should list")
                .is_empty(),
            "draft creation must not create an activity"
        );
        drop(core);

        assert_permission_audit(
            &app_data_dir,
            &client.id,
            CREATE_ACTIVITY_DRAFT_TOOL,
            true,
            "allowed",
            "evaluate_external_client_draft_permission",
            "draft",
        );
        cleanup(app_data_dir);
    }

    #[test]
    fn read_only_client_cannot_create_activity_draft_and_records_draft_audit() {
        let app_data_dir = test_app_data_dir();
        let mut core = open_core(&app_data_dir);
        let client = create_read_only_client(&mut core, "Read Only Draft SDK Client");
        core.upsert_external_client_tool_permission(
            &client.id,
            CREATE_ACTIVITY_DRAFT_TOOL,
            true,
            true,
            true,
        )
        .expect("read-only client permission row should upsert");
        drop(core);

        let mut sdk = ReadOnlyCrmSdk::open(&app_data_dir, &client.id).expect("SDK should open");
        let err = sdk
            .create_activity_draft(CreateActivityDraftParams {
                title: "Blocked draft".to_string(),
                activity_type: None,
                description: None,
                due_at: None,
                linked_entities: Vec::new(),
            })
            .expect_err("read-only client should not create drafts");
        assert_invalid_input_contains(err, "write_not_allowed");
        drop(sdk);

        let core = open_core(&app_data_dir);
        assert!(core
            .list_pending_proposed_actions()
            .expect("pending actions should list")
            .is_empty());
        drop(core);
        assert_permission_audit(
            &app_data_dir,
            &client.id,
            CREATE_ACTIVITY_DRAFT_TOOL,
            false,
            "denied",
            "evaluate_external_client_draft_permission",
            "draft",
        );
        cleanup(app_data_dir);
    }

    fn test_app_data_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        let counter = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "900crm-sdk-test-{}-{nanos}-{counter}",
            std::process::id()
        ))
    }

    fn open_core(app_data_dir: &Path) -> CrmCore {
        CrmCore::open(app_data_dir).expect("test core should open")
    }

    fn create_read_only_client(core: &mut CrmCore, name: &str) -> ExternalClient {
        let client = core
            .create_external_client_placeholder(name, "mcp")
            .expect("external client placeholder should be created");
        core.update_external_client_activation(&client.id, true, "read_only")
            .expect("external client should enable read-only");
        client
    }

    fn create_draft_only_client(core: &mut CrmCore, name: &str) -> ExternalClient {
        let client = core
            .create_external_client_placeholder(name, "mcp")
            .expect("external client placeholder should be created");
        core.update_external_client_activation(&client.id, true, "draft_only")
            .expect("external client should enable draft-only");
        client
    }

    fn assert_invalid_input_contains(error: CrmError, expected: &str) {
        match error {
            CrmError::InvalidInput(message) => {
                assert!(message.contains(expected), "{message}");
            }
            other => panic!("expected InvalidInput containing {expected}, got {other:?}"),
        }
    }

    fn assert_permission_audit(
        app_data_dir: &Path,
        client_id: &str,
        tool_name: &str,
        allowed: bool,
        status: &str,
        action: &str,
        access_kind: &str,
    ) {
        let entry = latest_permission_audit(app_data_dir, client_id, tool_name, action);
        assert_eq!(entry.action, action);
        assert_eq!(entry.entity_type.as_deref(), Some("external_client"));
        assert_eq!(entry.entity_id.as_deref(), Some(client_id));

        let after_json = entry
            .after_json
            .as_deref()
            .expect("permission audit should include after_json context");
        assert!(
            after_json.contains(&format!(r#""client_id":"{client_id}""#)),
            "{after_json}"
        );
        assert!(
            after_json.contains(&format!(r#""tool_name":"{tool_name}""#)),
            "{after_json}"
        );
        assert!(
            after_json.contains(&format!(r#""access_kind":"{access_kind}""#)),
            "{after_json}"
        );
        assert!(
            after_json.contains(&format!(r#""allowed":{allowed}"#)),
            "{after_json}"
        );
        assert!(
            after_json.contains(&format!(r#""status":"{status}""#)),
            "{after_json}"
        );
    }

    fn latest_permission_audit(
        app_data_dir: &Path,
        client_id: &str,
        tool_name: &str,
        action: &str,
    ) -> AuditLogEntry {
        let core = open_core(app_data_dir);
        core.list_recent_audit_log(50)
            .expect("audit log should list")
            .into_iter()
            .find(|entry| {
                entry.action == action
                    && entry.entity_id.as_deref() == Some(client_id)
                    && entry
                        .after_json
                        .as_deref()
                        .is_some_and(|json| json.contains(&format!(r#""tool_name":"{tool_name}""#)))
            })
            .expect("permission audit entry should exist")
    }

    fn assert_tool_result_audit(
        app_data_dir: &Path,
        client_id: &str,
        tool_name: &str,
        result_count: u32,
    ) {
        let entry = latest_tool_result_audit(app_data_dir, client_id, tool_name)
            .expect("tool result audit entry should exist");
        assert_eq!(entry.action, "record_external_client_tool_result");
        assert_eq!(entry.actor_type, "mcp_client");
        assert_eq!(entry.entity_type.as_deref(), Some("external_client"));
        assert_eq!(entry.entity_id.as_deref(), Some(client_id));

        let after_json = entry
            .after_json
            .as_deref()
            .expect("tool result audit should include after_json context");
        assert!(
            after_json.contains(&format!(r#""client_id":"{client_id}""#)),
            "{after_json}"
        );
        assert!(
            after_json.contains(&format!(r#""tool_name":"{tool_name}""#)),
            "{after_json}"
        );
        assert!(
            after_json.contains(r#""access_kind":"read""#),
            "{after_json}"
        );
        assert!(
            after_json.contains(r#""status":"succeeded""#),
            "{after_json}"
        );
        assert!(
            after_json.contains(&format!(r#""result_count":{result_count}"#)),
            "{after_json}"
        );
    }

    fn assert_no_tool_result_audit(app_data_dir: &Path, client_id: &str, tool_name: &str) {
        assert!(
            latest_tool_result_audit(app_data_dir, client_id, tool_name).is_none(),
            "denied SDK reads must not record successful tool result audits"
        );
    }

    fn latest_tool_result_audit(
        app_data_dir: &Path,
        client_id: &str,
        tool_name: &str,
    ) -> Option<AuditLogEntry> {
        let core = open_core(app_data_dir);
        core.list_recent_audit_log(50)
            .expect("audit log should list")
            .into_iter()
            .find(|entry| {
                entry.action == "record_external_client_tool_result"
                    && entry.entity_id.as_deref() == Some(client_id)
                    && entry
                        .after_json
                        .as_deref()
                        .is_some_and(|json| json.contains(&format!(r#""tool_name":"{tool_name}""#)))
            })
    }

    fn cleanup(app_data_dir: PathBuf) {
        let _ = std::fs::remove_dir_all(app_data_dir);
    }
}
