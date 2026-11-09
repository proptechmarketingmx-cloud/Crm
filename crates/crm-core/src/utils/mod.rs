//! Utility helpers for the 900CRM application.
//!
//! This module re-exports all utility sub-modules:
//!
//! - [`errors`] — [`CrmError`] enum and [`CrmResult<T>`] type alias.
//! - [`uuid`] — UUID v4 generation helpers.
//! - [`csv`] — flat CSV/JSON row parsing and writing for contact, deal, and organization import-export.
//! - [`datetime`] — ISO 8601 timestamp helpers and human-readable formatting.

pub mod csv;
pub mod datetime;
pub mod errors;
pub mod uuid;
