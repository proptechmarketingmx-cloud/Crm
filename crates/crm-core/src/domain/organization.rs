use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationId(pub String);

pub use crate::storage::organizations::Organization;
