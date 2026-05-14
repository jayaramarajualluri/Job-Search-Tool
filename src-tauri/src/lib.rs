//! Job Search Tool — Tauri backend entrypoint.
//!
//! Module map:
//!   config       — paths for app data, DB, logs, user root folder
//!   error        — unified `AppError` returned across the IPC boundary
//!   db           — SQLite pool + migrations + per-entity repositories
//!   domain       — strongly-typed business entities (Job, Company, …)
//!   ingestion    — source connectors + normalizer + runner
//!   ranking      — scoring engine (recency, location, sponsorship, skills)
//!   resume       — canonical profile + tailoring + rendering
//!   files        — cross-platform folder/file organizer
//!   secrets      — OS keychain wrapper (Keychain / Credential Manager)
//!   commands     — thin Tauri command handlers wiring UI ↔ core modules

use tauri::Manager;

pub mod config;
pub mod db;
pub mod domain;
pub mod error;

pub mod files;
pub mod ingestion;
pub mod ranking;
pub mod resume;
pub mod secrets;

pub mod commands;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize the Tauri application. Called from `main.rs`.
pub fn run() {
    // Logging: default INFO, respect RUST_LOG if set.  Secret fields are
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
            let handle = app.handle().clone();
            let data_dir = config::app_data_dir(&handle)?;
            let db = db::Database::open(&data_dir)?;
            db.migrate()?;
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Misc
            commands::ping,

            // Jobs
            commands::jobs::list_jobs,
            commands::jobs::get_job,
            commands::jobs::update_job_status,

            // Companies
            commands::companies::list_companies,

            // Accounts
            commands::accounts::list_accounts,
            commands::accounts::create_account,
            commands::accounts::update_account,
            commands::accounts::save_account_password,
            commands::accounts::clear_account_password,
            commands::accounts::reveal_account_password,
            commands::accounts::mark_account_used,
            commands::accounts::delete_account,

            // Settings
            commands::settings::load_settings,
            commands::settings::save_settings,

            // Ingestion
            commands::ingestion::run_ingestion,
            commands::ingestion::import_url,
            commands::ingestion::import_text,
            commands::ingestion::resolve_linkedin_url,
            commands::ingestion::list_runs,

            // Resumes
            commands::resumes::load_canonical_profile,
            commands::resumes::save_canonical_profile,
            commands::resumes::list_resumes_for_job,
            commands::resumes::tailor_resume_for_job,

            // Files
            commands::files::open_path,
            commands::files::open_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
