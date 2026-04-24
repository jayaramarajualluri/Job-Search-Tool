//! Job-related commands.

use crate::db::repo;
use crate::db::Database;
use crate::domain::job::{Job, JobStatus, RecencyBucket};
use crate::error::AppResult;
use tauri::State;

#[tauri::command]
pub fn list_jobs(
    db: State<'_, Database>,
    min_score: Option<f32>,
    status: Option<JobStatus>,
    recency: Option<Vec<RecencyBucket>>,
    company_id: Option<i64>,
    include_explicit_no_sponsorship: Option<bool>,
) -> AppResult<Vec<Job>> {
    let c = db.conn()?;
    let settings = repo::settings::load(&c)?;
    let f = repo::jobs::ListFilter {
        min_score,
        status,
        recency,
        company_id,
        include_explicit_no_sponsorship: include_explicit_no_sponsorship
            .unwrap_or(!settings.exclude_explicit_no_sponsorship),
    };
    repo::jobs::list_filtered(&c, &f)
}

#[tauri::command]
pub fn get_job(db: State<'_, Database>, id: i64) -> AppResult<Option<Job>> {
    let c = db.conn()?;
    repo::jobs::get(&c, id)
}

#[tauri::command]
pub fn update_job_status(
    db: State<'_, Database>,
    id: i64,
    status: JobStatus,
    notes: Option<String>,
) -> AppResult<()> {
    let c = db.conn()?;
    repo::jobs::update_status(&c, id, status, notes.as_deref())
}
