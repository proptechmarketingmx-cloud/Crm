//! Permission primitives for optional external clients.

use serde::{Deserialize, Serialize};

pub const DEFAULT_EXTERNAL_CLIENT_PERMISSION_MODE: &str = "disabled";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalClientPermissionMode {
    Disabled,
    ReadOnly,
    DraftOnly,
    WriteWithConfirmation,
    WriteAllowed,
}

impl ExternalClientPermissionMode {
    pub fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "disabled" => Some(Self::Disabled),
            "read_only" => Some(Self::ReadOnly),
            "draft_only" => Some(Self::DraftOnly),
            "write_with_confirmation" => Some(Self::WriteWithConfirmation),
            "write_allowed" => Some(Self::WriteAllowed),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::ReadOnly => "read_only",
            Self::DraftOnly => "draft_only",
            Self::WriteWithConfirmation => "write_with_confirmation",
            Self::WriteAllowed => "write_allowed",
        }
    }

    pub fn is_supported_initial_mode(self) -> bool {
        matches!(self, Self::Disabled | Self::ReadOnly | Self::DraftOnly)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolPermissionGrant {
    pub can_read: bool,
    pub can_write: bool,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolPermissionDecisionReason {
    Allowed,
    ClientDisabled,
    UnsupportedClientMode,
    MissingToolPermission,
    ReadNotAllowed,
    WriteNotAllowed,
    ConfirmationNotRequired,
}

impl ToolPermissionDecisionReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::ClientDisabled => "client_disabled",
            Self::UnsupportedClientMode => "unsupported_client_mode",
            Self::MissingToolPermission => "missing_tool_permission",
            Self::ReadNotAllowed => "read_not_allowed",
            Self::WriteNotAllowed => "write_not_allowed",
            Self::ConfirmationNotRequired => "confirmation_not_required",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolPermissionEvaluation {
    pub allowed: bool,
    pub mode: ExternalClientPermissionMode,
    pub tool_name: String,
    pub reason: ToolPermissionDecisionReason,
}

impl ToolPermissionEvaluation {
    fn allowed(mode: ExternalClientPermissionMode, tool_name: &str) -> Self {
        Self {
            allowed: true,
            mode,
            tool_name: tool_name.to_string(),
            reason: ToolPermissionDecisionReason::Allowed,
        }
    }

    fn denied(
        mode: ExternalClientPermissionMode,
        tool_name: &str,
        reason: ToolPermissionDecisionReason,
    ) -> Self {
        Self {
            allowed: false,
            mode,
            tool_name: tool_name.to_string(),
            reason,
        }
    }
}

pub fn is_supported_initial_permission_mode(mode: &str) -> bool {
    ExternalClientPermissionMode::from_storage_value(mode)
        .is_some_and(ExternalClientPermissionMode::is_supported_initial_mode)
}

pub fn evaluate_tool_read_permission(
    mode: ExternalClientPermissionMode,
    tool_name: &str,
    grant: Option<ToolPermissionGrant>,
) -> ToolPermissionEvaluation {
    match mode {
        ExternalClientPermissionMode::Disabled => ToolPermissionEvaluation::denied(
            mode,
            tool_name,
            ToolPermissionDecisionReason::ClientDisabled,
        ),
        ExternalClientPermissionMode::ReadOnly | ExternalClientPermissionMode::DraftOnly => {
            match grant {
                Some(grant) if grant.can_read => ToolPermissionEvaluation::allowed(mode, tool_name),
                Some(_) => ToolPermissionEvaluation::denied(
                    mode,
                    tool_name,
                    ToolPermissionDecisionReason::ReadNotAllowed,
                ),
                None => ToolPermissionEvaluation::denied(
                    mode,
                    tool_name,
                    ToolPermissionDecisionReason::MissingToolPermission,
                ),
            }
        }
        ExternalClientPermissionMode::WriteWithConfirmation
        | ExternalClientPermissionMode::WriteAllowed => ToolPermissionEvaluation::denied(
            mode,
            tool_name,
            ToolPermissionDecisionReason::UnsupportedClientMode,
        ),
    }
}

pub fn evaluate_tool_draft_permission(
    mode: ExternalClientPermissionMode,
    tool_name: &str,
    grant: Option<ToolPermissionGrant>,
) -> ToolPermissionEvaluation {
    match mode {
        ExternalClientPermissionMode::DraftOnly => match grant {
            Some(grant) if grant.can_write && grant.requires_confirmation => {
                ToolPermissionEvaluation::allowed(mode, tool_name)
            }
            Some(grant) if grant.can_write => ToolPermissionEvaluation::denied(
                mode,
                tool_name,
                ToolPermissionDecisionReason::ConfirmationNotRequired,
            ),
            Some(_) => ToolPermissionEvaluation::denied(
                mode,
                tool_name,
                ToolPermissionDecisionReason::WriteNotAllowed,
            ),
            None => ToolPermissionEvaluation::denied(
                mode,
                tool_name,
                ToolPermissionDecisionReason::MissingToolPermission,
            ),
        },
        ExternalClientPermissionMode::Disabled => ToolPermissionEvaluation::denied(
            mode,
            tool_name,
            ToolPermissionDecisionReason::ClientDisabled,
        ),
        ExternalClientPermissionMode::ReadOnly => ToolPermissionEvaluation::denied(
            mode,
            tool_name,
            ToolPermissionDecisionReason::WriteNotAllowed,
        ),
        ExternalClientPermissionMode::WriteWithConfirmation
        | ExternalClientPermissionMode::WriteAllowed => ToolPermissionEvaluation::denied(
            mode,
            tool_name,
            ToolPermissionDecisionReason::UnsupportedClientMode,
        ),
    }
}
