// Prevents an additional console window from appearing on Windows in release mode.
// This attribute must appear on the very first line of main.rs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! 900CRM — Tauri v2 application entry point.
//!
//! This file is the Rust binary entry point. It delegates all logic to
//! the library crate (`ninehundredcrm_lib`), following the Tauri v2
//! recommended binary/library split pattern.
//!
//! # Responsibilities
//!
//! 1. Calls [`ninehundredcrm_lib::run`] which:
//!    - Initializes `env_logger` for structured logging.
//!    - Opens (or creates) the SQLite database.
//!    - Constructs the shared [`ninehundredcrm_lib::AppState`].
//!    - Registers all Tauri command handlers.
//!    - Boots the Tauri event loop.
//!
//! # Library crate split
//!
//! All business logic lives in `ninehundredcrm_lib` so it can be unit-tested
//! without starting a full Tauri application. This file is intentionally thin.
//!
//! # Logging
//!
//! Log level is controlled by the `RUST_LOG` environment variable.
//! In development builds the default level is `debug`; in release builds
//! it is `info`.
//!
//! ```sh
//! RUST_LOG=900crm=debug ./900CRM
//! RUST_LOG=trace ./900CRM   # verbose — includes all crate internals
//! ```

use ninehundredcrm_lib::run;

/// Application entry point.
///
/// Delegates immediately to [`run`] in the library crate.
fn main() {
    run();
}
