//! Tauri-independent core for 900CRM.
//!
//! This crate owns business rules, typed services, SQLite storage, audit
//! foundations, and MCP-readiness primitives. Desktop and future optional
//! integrations call this crate instead of reaching into SQLite directly.

pub mod audit;
pub mod crm_engine;
pub mod domain;
pub mod errors;
pub mod import_export;
pub mod permissions;
pub mod result;
pub mod search;
pub mod services;
pub mod storage;
pub mod utils;

pub use services::CrmCore;
