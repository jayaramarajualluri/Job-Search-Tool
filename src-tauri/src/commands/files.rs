//! File/URL opening helpers.  Delegates to `tauri_plugin_opener` so the
//! OS default handler is used (Finder on macOS, Explorer on Windows).

use crate::error::{AppError, AppResult};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub fn open_path(app: AppHandle, path: String) -> AppResult<()> {
    // Basic validation: must be an existing absolute path.
    let p = std::path::PathBuf::from(&path);
    if !p.exists() {
        return Err(AppError::not_found(path));
    }
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> AppResult<()> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(AppError::invalid("only http/https URLs are allowed"));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| AppError::Other(e.to_string()))
}
