use crate::crm_engine::search::{self, SearchResult};
use crate::result::CrmResult;

use super::CrmCore;

impl CrmCore {
    pub fn global_search(&self, query: &str, limit: Option<u32>) -> CrmResult<Vec<SearchResult>> {
        search::unified_search(
            &self.db.conn,
            query,
            limit.unwrap_or_else(search::default_limit),
        )
    }
}
