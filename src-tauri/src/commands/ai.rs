//! AI commands.  The API key is stored in the OS keychain under the
//! service "jobsearchtool" with the logical name "ai_api_key".  The key
//! value itself is never returned to the frontend — the UI only learns
//! whether it's set.

use crate::error::AppResult;
use crate::secrets::keyring;

const AI_KEY: &str = "ai_api_key";

#[tauri::command]
pub fn set_ai_api_key(api_key: String) -> AppResult<()> {
    keyring::put_named(AI_KEY, &api_key)
}

#[tauri::command]
pub fn has_ai_api_key() -> AppResult<bool> {
    Ok(keyring::get_named(AI_KEY)?.is_some())
}

#[tauri::command]
pub fn clear_ai_api_key() -> AppResult<()> {
    keyring::delete_named(AI_KEY)
}
