//! Settings repository.  Single JSON document stored under the fixed key
//! `app_settings` so we can evolve the shape without schema migrations.

use crate::db::DbConn;
use crate::domain::settings::AppSettings;
use crate::error::AppResult;
use rusqlite::{params, OptionalExtension};

const KEY: &str = "app_settings";

pub fn load(c: &DbConn) -> AppResult<AppSettings> {
    let raw: Option<String> = c
        .query_row("SELECT value FROM settings WHERE key = ?1", [KEY], |r| r.get(0))
        .optional()?;
    let s = raw
        .as_deref()
        .and_then(|v| serde_json::from_str::<AppSettings>(v).ok())
        .unwrap_or_default();
    Ok(s)
}

pub fn save(c: &DbConn, s: &AppSettings) -> AppResult<()> {
    let json = serde_json::to_string(s)?;
    c.execute(
        "INSERT INTO settings(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value,
            updated_at = datetime('now')",
        params![KEY, json],
    )?;
    Ok(())
}
