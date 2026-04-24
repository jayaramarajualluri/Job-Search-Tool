//! Company commands.

use crate::db::repo;
use crate::db::Database;
use crate::domain::Company;
use crate::error::AppResult;
use tauri::State;

#[tauri::command]
pub fn list_companies(db: State<'_, Database>) -> AppResult<Vec<Company>> {
    let c = db.conn()?;
    repo::companies::list(&c)
}
