//! Domain model namespace for the target schema.
//!
//! Existing persisted structs are still exported from `storage` during this
//! foundation sprint. New target-schema modules are intentionally lightweight
//! so follow-up sprints can move persistence DTOs into stable domain types.

pub mod activity;
pub mod audit;
pub mod contact;
pub mod deal;
pub mod external_client;
pub mod note;
pub mod organization;
pub mod pipeline;
pub mod proposed_action;
pub mod tag;
