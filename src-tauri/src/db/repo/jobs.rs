//! Jobs repository.  Query bodies are filled in during Stage 6.

#![allow(dead_code)]

use crate::db::DbConn;
use crate::domain::Job;
use crate::error::AppResult;

pub fn list(_c: &DbConn) -> AppResult<Vec<Job>> {
    Ok(vec![])
}
