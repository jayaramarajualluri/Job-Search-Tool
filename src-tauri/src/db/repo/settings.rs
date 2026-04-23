//! Settings repository.  Query bodies are filled in during Stage 6.

#![allow(dead_code)]

use crate::db::DbConn;
use crate::domain::SettingEntry;
use crate::error::AppResult;

pub fn list(_c: &DbConn) -> AppResult<Vec<SettingEntry>> {
    Ok(vec![])
}
