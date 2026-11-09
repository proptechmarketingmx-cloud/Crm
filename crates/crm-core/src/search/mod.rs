//! Search foundation namespace.
//!
//! `crm_engine::search` owns cross-entity search orchestration while
//! `storage::search` owns repository queries. This namespace marks the future
//! stable service boundary for global search.

pub use crate::crm_engine::search::{SearchEntityType, SearchResult};
