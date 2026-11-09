//! Audit foundation namespace.
//!
//! User-visible audit rows are persisted by `storage::audit` and are written
//! by service methods. This module keeps audit concepts available without
//! coupling desktop code to storage internals.

pub const ACTOR_DESKTOP_APP: &str = "desktop_app";
pub const ACTOR_IMPORT: &str = "import";
pub const ACTOR_MCP_CLIENT: &str = "mcp_client";
pub const ACTOR_SYSTEM: &str = "system";
