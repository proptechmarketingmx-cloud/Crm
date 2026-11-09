//! CRM business logic engine for 900CRM.
//!
//! The `crm_engine` module sits between the Tauri command handlers and the
//! raw storage layer. It contains validation, domain rules, and
//! higher-level orchestration that would be too complex to live in either
//! the commands (presentation layer) or storage (persistence layer).
//!
//! # Sub-modules
//!
//! | Sub-module    | Responsibility |
//! |---------------|----------------|
//! | [`contacts`]  | Validation, duplicate detection, merge logic |
//! | [`deals`]     | Stage definitions, transition rules, win rate |
//! | [`activities`]| Scheduling, overdue detection, summary stats |
//! | [`pipeline`]  | Pipeline config, metrics, conversion rates |
//! | [`search`]    | Unified cross-entity search |
//!
//! # CrmEngine
//!
//! The [`CrmEngine`] struct holds no mutable state itself (all state lives in
//! the `Database`). Desktop `AppState` stores `Mutex<Option<CrmCore>>`.
//! [`crate::CrmCore`] owns the engine and the database together. Command
//! handlers lock that single optional core slot; there is no separate
//! `engine` lock and no dual-lock order.
//!
//! # Storage boundary
//!
//! Persistent queries belong in `crate::storage`. This namespace should stay
//! focused on validation, domain rules, and orchestration.

pub mod activities;
pub mod contacts;
pub mod deals;
pub mod pipeline;
pub mod search;

// ─────────────────────────────────────────────────────────────────────────────
// CrmEngine
// ─────────────────────────────────────────────────────────────────────────────

/// The CRM business logic engine.
///
/// `CrmEngine` is a stateless orchestrator — all persistent state lives in
/// the `Database`. It is owned by [`crate::CrmCore`]. The desktop shell
/// stores that core in `AppState` as `Mutex<Option<CrmCore>>`. Command
/// handlers obtain it through that lock rather than `state.engine`.
#[derive(Debug)]
pub struct CrmEngine {
    /// The default pipeline stage definitions in order.
    ///
    /// Used when creating deals without an explicit stage and for pipeline
    /// metrics calculations.
    pub default_stages: Vec<pipeline::StageDefinition>,
}

impl CrmEngine {
    /// Creates a new `CrmEngine` with the default pipeline stage configuration.
    ///
    /// This is called from [`crate::CrmCore::open`] when the desktop core is
    /// constructed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use crate::crm_engine::CrmEngine;
    ///
    /// let engine = CrmEngine::new();
    /// assert_eq!(engine.default_stages.len(), 6);
    /// ```
    pub fn new() -> Self {
        Self {
            default_stages: pipeline::default_stages(),
        }
    }

    /// Returns the default probability for a pipeline stage by name.
    ///
    /// Falls back to `20` for unknown stage names.
    pub fn default_probability(&self, stage: &str) -> i32 {
        self.default_stages
            .iter()
            .find(|s| s.name == stage)
            .map(|s| s.default_probability)
            .unwrap_or(20)
    }

    /// Returns the ordered position of a stage (0-based), or `usize::MAX` if
    /// the stage name is not found.
    ///
    /// Used for ordering stages in pipeline views.
    pub fn stage_order(&self, stage: &str) -> usize {
        self.default_stages
            .iter()
            .position(|s| s.name == stage)
            .unwrap_or(usize::MAX)
    }
}

impl Default for CrmEngine {
    fn default() -> Self {
        Self::new()
    }
}
