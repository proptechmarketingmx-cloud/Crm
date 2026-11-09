use crm_core::search::SearchResult;
use tauri::State;

use crate::{commands::lock_core, AppState};

const DEFAULT_GLOBAL_SEARCH_LIMIT: u32 = 30;
const MAX_GLOBAL_SEARCH_LIMIT: u32 = 100;

#[tauri::command(rename_all = "snake_case")]
pub async fn global_search(
    state: State<'_, AppState>,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<SearchResult>, String> {
    let core = lock_core(&state)?;
    core.global_search(&query, Some(global_search_limit(limit)))
        .map_err(|e| e.to_string())
}

fn global_search_limit(limit: Option<u32>) -> u32 {
    limit
        .unwrap_or(DEFAULT_GLOBAL_SEARCH_LIMIT)
        .clamp(1, MAX_GLOBAL_SEARCH_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::{global_search_limit, DEFAULT_GLOBAL_SEARCH_LIMIT, MAX_GLOBAL_SEARCH_LIMIT};

    #[test]
    fn global_search_limit_defaults_and_clamps_to_storage_bounds() {
        assert_eq!(global_search_limit(None), DEFAULT_GLOBAL_SEARCH_LIMIT);
        assert_eq!(global_search_limit(Some(0)), 1);
        assert_eq!(global_search_limit(Some(25)), 25);
        assert_eq!(global_search_limit(Some(750)), MAX_GLOBAL_SEARCH_LIMIT);
    }
}
