//! Ingestion-runs repository.

use crate::db::DbConn;
use crate::domain::ingestion_run::{IngestionRun, SourceSummary};
use crate::error::AppResult;
use rusqlite::{params, Row};

pub fn list(c: &DbConn) -> AppResult<Vec<IngestionRun>> {
    let mut stmt = c.prepare(&format!(
        "{} ORDER BY started_at DESC LIMIT 50",
        SELECT_BASE
    ))?;
    let rows = stmt.query_map([], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn start(c: &DbConn) -> AppResult<i64> {
    c.execute(
        "INSERT INTO ingestion_runs(started_at) VALUES (datetime('now'))",
        [],
    )?;
    Ok(c.last_insert_rowid())
}

pub fn finish(
    c: &DbConn,
    id: i64,
    summary: &SourceSummary,
    totals: (u32, u32, u32),
    notes: Option<&str>,
) -> AppResult<()> {
    let json = serde_json::to_string(summary)?;
    c.execute(
        "UPDATE ingestion_runs SET
            ended_at = datetime('now'),
            source_summary = ?1,
            total_fetched = ?2, total_filtered = ?3, total_prepared = ?4,
            notes = ?5
         WHERE id = ?6",
        params![json, totals.0, totals.1, totals.2, notes, id],
    )?;
    Ok(())
}

const SELECT_BASE: &str = "SELECT id, started_at, ended_at, source_summary,
    total_fetched, total_filtered, total_prepared, notes FROM ingestion_runs";

fn map_row(r: &Row<'_>) -> rusqlite::Result<IngestionRun> {
    let json: Option<String> = r.get(3)?;
    let source_summary = json
        .as_deref()
        .and_then(|s| serde_json::from_str::<SourceSummary>(s).ok());
    Ok(IngestionRun {
        id: r.get(0)?,
        started_at: r.get(1)?,
        ended_at: r.get(2)?,
        source_summary,
        total_fetched: r.get::<_, i64>(4)? as u32,
        total_filtered: r.get::<_, i64>(5)? as u32,
        total_prepared: r.get::<_, i64>(6)? as u32,
        notes: r.get(7)?,
    })
}
