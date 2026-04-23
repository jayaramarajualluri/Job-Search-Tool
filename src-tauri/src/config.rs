//! App-wide paths. The data directory holds the SQLite DB and logs. The job
//! asset root is a user-configurable path stored in `settings` and defaults
//! to `<documents>/JobSearchTool`.

use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Resolve the per-user app data directory and ensure it exists.
pub fn app_data_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Config(format!("no app_data_dir: {e}")))?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Default location for the job-asset folder tree. User overrides via Settings.
pub fn default_root_folder() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|d| d.document_dir().map(|p| p.join("JobSearchTool")))
        .unwrap_or_else(|| PathBuf::from("JobSearchTool"))
}
