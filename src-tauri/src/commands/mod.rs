//! IPC surface.  Each submodule groups commands for one domain area.  Stage
//! 6 wires the full set into `invoke_handler!`.  Until then, only `ping`
//! is registered so the frontend has a known end-to-end path.

use crate::error::AppResult;

#[tauri::command]
pub async fn ping() -> AppResult<&'static str> {
    Ok("pong")
}
