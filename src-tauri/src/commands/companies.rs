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

#[tauri::command]
pub fn update_company_notes(
    db: State<'_, Database>,
    id: i64,
    notes: Option<String>,
) -> AppResult<()> {
    let c = db.conn()?;
    repo::companies::update_notes(&c, id, notes.as_deref())
}
