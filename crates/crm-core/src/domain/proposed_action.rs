use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposedActionStatus {
    Pending,
    Approved,
    Rejected,
    Executed,
    Failed,
    Cancelled,
}
