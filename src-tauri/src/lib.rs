//! Job Search Tool — Tauri backend entrypoint.
//!
//! Module map:
//!   config       — paths for app data, DB, logs, user root folder
//!   error        — unified `AppError` returned across the IPC boundary
//!   db           — SQLite pool + migrations + per-entity repositories
//!   domain       — strongly-typed business entities (Job, Company, …)
//!   ingestion    — source connectors (Greenhouse, Lever, Ashby, manual, LI)
//!   ranking      — scoring engine (recency, location, sponsorship, skills)
//!   resume       — canonical profile + tailoring + rendering
//!   files        — cross-platform folder/file organizer
//!   secrets      — OS keychain wrapper (Keychain / Credential Manager)
//!   commands     — thin Tauri command handlers wiring UI ↔ core modules

pub mod config;
pub mod db;
pub mod domain;
pub mod error;

// The following modules are filled in during later stages.
pub mod files;
pub mod ingestion;
pub mod ranking;
pub mod resume;
pub mod secrets;

pub mod commands;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize the Tauri application. Called from `main.rs`.
pub fn run() {
    // Logging: default INFO, respect RUST_LOG if set. Secret fields are
    // redacted at the call site — we never log tokens/passwords as-is.
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,job_search_tool_lib=debug"));
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Resolve/create the app data directory and open the DB. Failing
            // here is fatal — the app has nothing to show without a DB.
            let data_dir = config::app_data_dir(app.handle())?;
            let db = db::Database::open(&data_dir)?;
            db.migrate()?;
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            // Stage 6 wires the full command surface.
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
