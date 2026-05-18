//! Companies repository.

use crate::db::DbConn;
use crate::domain::Company;
use crate::error::AppResult;
use crate::ingestion::normalize::normalize_name;
use rusqlite::{params, OptionalExtension, Row};

pub fn list(c: &DbConn) -> AppResult<Vec<Company>> {
    let mut stmt = c.prepare(
        "SELECT id, name, normalized_name, company_folder_path, notes, created_at, updated_at
         FROM companies ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get(c: &DbConn, id: i64) -> AppResult<Option<Company>> {
    Ok(c.query_row(
        "SELECT id, name, normalized_name, company_folder_path, notes, created_at, updated_at
         FROM companies WHERE id = ?1",
        [id],
        map_row,
    )
    .optional()?)
}

pub fn find_by_normalized(c: &DbConn, normalized: &str) -> AppResult<Option<Company>> {
    Ok(c.query_row(
        "SELECT id, name, normalized_name, company_folder_path, notes, created_at, updated_at
         FROM companies WHERE normalized_name = ?1",
        [normalized],
        map_row,
    )
    .optional()?)
}

/// Insert or fetch existing.  Returns the company row.
pub fn upsert_by_name(c: &DbConn, name: &str) -> AppResult<Company> {
    let normalized = normalize_name(name);
    if let Some(existing) = find_by_normalized(c, &normalized)? {
        return Ok(existing);
    }
    c.execute(
        "INSERT INTO companies(name, normalized_name) VALUES (?1, ?2)",
        params![name, normalized],
    )?;
    let id = c.last_insert_rowid();
    Ok(get(c, id)?.expect("company was just inserted"))
}

pub fn update_notes(c: &DbConn, id: i64, notes: Option<&str>) -> AppResult<()> {
    c.execute(
        "UPDATE companies SET notes = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![notes, id],
    )?;
    Ok(())
}

pub fn set_folder_path(c: &DbConn, id: i64, path: &str) -> AppResult<()> {
    c.execute(
        "UPDATE companies SET company_folder_path = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![path, id],
    )?;
    Ok(())
}

fn map_row(r: &Row<'_>) -> rusqlite::Result<Company> {
    Ok(Company {
        id: r.get(0)?,
        name: r.get(1)?,
        normalized_name: r.get(2)?,
        company_folder_path: r.get(3)?,
        notes: r.get(4)?,
        created_at: r.get(5)?,
        updated_at: r.get(6)?,
    })
}
