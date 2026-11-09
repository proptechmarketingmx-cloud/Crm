//! SQLite persistence layer for 900CRM.
//!
//! This module provides database access for all CRM entities. It exports the
//! [`Database`] struct (re-exported here for convenience) and organises CRUD
//! operations into focused sub-modules:
//!
//! | Sub-module   | Responsibility |
//! |--------------|----------------|
//! | [`db`]       | Connection setup, PRAGMAs, schema migrations |
//! | [`contacts`] | Contact CRUD + FTS5 search |
//! | [`deals`]    | Deal CRUD + pipeline stage operations |
//! | [`activities`] | Activity CRUD + scheduling queries |
//! | [`notes`]    | Entity-attached notes CRUD |
//! | [`organizations`] | Organization CRUD |
//! | [`tags`]     | Tag management and entity tagging |
//! | [`settings`] | Key-value application settings |
//! | [`sync`]     | Offline-first sync changelog |
//!
//! All storage functions accept a `&rusqlite::Connection` reference (obtained
//! from `db.conn`) so they can be composed without extra locking overhead.

pub mod activities;
pub mod audit;
pub mod contacts;
pub mod custom_fields;
pub mod dashboard;
pub mod db;
pub mod deals;
pub mod external_client_permissions;
pub mod external_clients;
pub mod links;
pub mod migration_readiness;
pub mod notes;
pub mod organizations;
pub mod proposed_actions;
pub mod reporting;
pub mod saved_views;
pub mod search;
pub mod settings;
pub mod sync;
pub mod tags;

/// Re-export the [`Database`] struct at the storage module level.
///
/// Command handlers and [`crate::AppState`] use `storage::Database` without
/// needing to descend into `storage::db::Database`.
pub use db::Database;
