//! IPC surface.  Each submodule groups commands for one domain area.
//! Commands are thin wrappers that extract state + call into core modules.
//! They MUST NOT contain business logic (see repo/ingestion/resume/...).

use crate::error::AppResult;

pub mod accounts;
pub mod companies;
pub mod files;
pub mod ingestion;
pub mod jobs;
pub mod resumes;
pub mod settings;

#[tauri::command]
pub async fn ping() -> AppResult<&'static str> {
    Ok("pong")
}
