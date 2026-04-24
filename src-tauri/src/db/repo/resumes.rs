//! Resumes repository.

use crate::db::DbConn;
use crate::domain::resume::{Resume, ReuseStrategy};
use crate::error::AppResult;
use rusqlite::{params, OptionalExtension, Row};

pub fn list_for_job(c: &DbConn, job_id: i64) -> AppResult<Vec<Resume>> {
    let mut stmt = c.prepare(&format!(
        "{} WHERE job_id = ?1 ORDER BY created_at DESC",
        SELECT_BASE
    ))?;
    let rows = stmt.query_map([job_id], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get(c: &DbConn, id: i64) -> AppResult<Option<Resume>> {
    Ok(c.query_row(
        &format!("{} WHERE id = ?1", SELECT_BASE),
        [id],
        map_row,
    )
    .optional()?)
}

pub fn list(c: &DbConn) -> AppResult<Vec<Resume>> {
    let mut stmt = c.prepare(&format!("{} ORDER BY id DESC", SELECT_BASE))?;
    let rows = stmt.query_map([], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn insert(
    c: &DbConn,
    job_id: i64,
    base_resume_name: &str,
    variant_name: &str,
    source_resume_path: Option<&str>,
    output_resume_path: Option<&str>,
    reuse_strategy: ReuseStrategy,
    similarity_score: Option<f32>,
) -> AppResult<i64> {
    c.execute(
        "INSERT INTO resumes(job_id, base_resume_name, resume_variant_name,
            source_resume_path, output_resume_path, reuse_strategy, similarity_score)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            job_id,
            base_resume_name,
            variant_name,
            source_resume_path,
            output_resume_path,
            reuse_str(reuse_strategy),
            similarity_score.map(|v| v as f64),
        ],
    )?;
    Ok(c.last_insert_rowid())
}

fn reuse_str(r: ReuseStrategy) -> &'static str {
    match r {
        ReuseStrategy::New => "new",
        ReuseStrategy::Reused => "reused",
        ReuseStrategy::EditedReuse => "edited_reuse",
    }
}
fn parse_reuse(s: &str) -> ReuseStrategy {
    match s {
        "reused" => ReuseStrategy::Reused,
        "edited_reuse" => ReuseStrategy::EditedReuse,
        _ => ReuseStrategy::New,
    }
}

const SELECT_BASE: &str = "SELECT id, job_id, base_resume_name, resume_variant_name,
    source_resume_path, output_resume_path, reuse_strategy, similarity_score,
    created_at, updated_at FROM resumes";

fn map_row(r: &Row<'_>) -> rusqlite::Result<Resume> {
    Ok(Resume {
        id: r.get(0)?,
        job_id: r.get(1)?,
        base_resume_name: r.get(2)?,
        resume_variant_name: r.get(3)?,
        source_resume_path: r.get(4)?,
        output_resume_path: r.get(5)?,
        reuse_strategy: parse_reuse(&r.get::<_, String>(6)?),
        similarity_score: r.get::<_, Option<f64>>(7)?.map(|v| v as f32),
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
    })
}
