//! Settings commands.

use crate::db::repo;
use crate::db::Database;
use crate::domain::settings::AppSettings;
use crate::error::AppResult;
use tauri::State;

#[tauri::command]
pub fn load_settings(db: State<'_, Database>) -> AppResult<AppSettings> {
    let c = db.conn()?;
    repo::settings::load(&c)
}

#[tauri::command]
pub fn save_settings(db: State<'_, Database>, settings: AppSettings) -> AppResult<()> {
    let c = db.conn()?;
    repo::settings::save(&c, &settings)
}
