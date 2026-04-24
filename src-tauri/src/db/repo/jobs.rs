//! Jobs repository.

use crate::db::DbConn;
use crate::domain::job::{
    Job, JobStatus, MatchExplanation, RecencyBucket, SponsorshipConfidence, WorkMode,
};
use crate::error::AppResult;
use rusqlite::{params, OptionalExtension, Row};
use serde::Deserialize;

/// Input for persisting a new (or refreshed) job.  Output fields like
/// `match_score`, folder paths, and `status` are set later in the pipeline.
#[derive(Debug, Clone)]
pub struct JobInsert {
    pub company_id: i64,
    pub source_name: String,
    pub source_url: Option<String>,
    pub apply_url: Option<String>,
    pub role_title: String,
    pub normalized_role_title: String,
    pub job_external_id: Option<String>,
    pub location: Option<String>,
    pub state_or_region: Option<String>,
    pub work_mode: WorkMode,
    pub posted_date: Option<String>,
    pub recency_bucket: Option<RecencyBucket>,
    pub jd_text: Option<String>,
    pub jd_summary: Option<String>,
    pub sponsorship_confidence: SponsorshipConfidence,
    pub sponsorship_reason: Option<String>,
}

pub fn list(c: &DbConn) -> AppResult<Vec<Job>> {
    let mut stmt = c.prepare(SELECT_BASE)?;
    let rows = stmt.query_map([], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get(c: &DbConn, id: i64) -> AppResult<Option<Job>> {
    let sql = format!("{} WHERE id = ?1", SELECT_BASE);
    Ok(c.query_row(&sql, [id], map_row).optional()?)
}

pub fn find_by_external(
    c: &DbConn,
    company_id: i64,
    external_id: &str,
) -> AppResult<Option<Job>> {
    let sql = format!(
        "{} WHERE company_id = ?1 AND job_external_id = ?2",
        SELECT_BASE
    );
    Ok(c.query_row(&sql, params![company_id, external_id], map_row)
        .optional()?)
}

/// Insert-or-refresh on (company_id, external_id).  Returns the row id.
pub fn upsert(c: &DbConn, ins: &JobInsert) -> AppResult<i64> {
    if let Some(ext) = ins.job_external_id.as_deref() {
        if let Some(existing) = find_by_external(c, ins.company_id, ext)? {
            update_fields(c, existing.id, ins)?;
            return Ok(existing.id);
        }
    }
    c.execute(
        "INSERT INTO jobs(
            company_id, source_name, source_url, apply_url, role_title,
            normalized_role_title, job_external_id, location, state_or_region,
            work_mode, posted_date, recency_bucket, jd_text, jd_summary,
            sponsorship_confidence, sponsorship_reason
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
        params![
            ins.company_id,
            ins.source_name,
            ins.source_url,
            ins.apply_url,
            ins.role_title,
            ins.normalized_role_title,
            ins.job_external_id,
            ins.location,
            ins.state_or_region,
            workmode_str(ins.work_mode),
            ins.posted_date,
            recency_str(ins.recency_bucket),
            ins.jd_text,
            ins.jd_summary,
            sponsorship_str(ins.sponsorship_confidence),
            ins.sponsorship_reason,
        ],
    )?;
    Ok(c.last_insert_rowid())
}

fn update_fields(c: &DbConn, id: i64, ins: &JobInsert) -> AppResult<()> {
    c.execute(
        "UPDATE jobs SET
            source_name = ?1, source_url = ?2, apply_url = ?3,
            role_title = ?4, normalized_role_title = ?5,
            location = ?6, state_or_region = ?7, work_mode = ?8,
            posted_date = ?9, recency_bucket = ?10,
            jd_text = ?11, jd_summary = ?12,
            sponsorship_confidence = ?13, sponsorship_reason = ?14,
            updated_at = datetime('now')
         WHERE id = ?15",
        params![
            ins.source_name,
            ins.source_url,
            ins.apply_url,
            ins.role_title,
            ins.normalized_role_title,
            ins.location,
            ins.state_or_region,
            workmode_str(ins.work_mode),
            ins.posted_date,
            recency_str(ins.recency_bucket),
            ins.jd_text,
            ins.jd_summary,
            sponsorship_str(ins.sponsorship_confidence),
            ins.sponsorship_reason,
            id,
        ],
    )?;
    Ok(())
}

pub fn update_match(
    c: &DbConn,
    id: i64,
    score: f32,
    explanation: &MatchExplanation,
) -> AppResult<()> {
    let expl = serde_json::to_string(explanation)?;
    c.execute(
        "UPDATE jobs SET match_score = ?1, match_explanation = ?2,
            updated_at = datetime('now') WHERE id = ?3",
        params![score as f64, expl, id],
    )?;
    Ok(())
}

pub fn update_paths(
    c: &DbConn,
    id: i64,
    role_folder: &str,
    jd_file: &str,
    resume_file: Option<&str>,
    cover_letter_file: Option<&str>,
) -> AppResult<()> {
    c.execute(
        "UPDATE jobs SET role_folder_path = ?1, jd_file_path = ?2,
            resume_file_path = ?3, cover_letter_file_path = ?4,
            updated_at = datetime('now') WHERE id = ?5",
        params![role_folder, jd_file, resume_file, cover_letter_file, id],
    )?;
    Ok(())
}

pub fn update_status(
    c: &DbConn,
    id: i64,
    new_status: JobStatus,
    notes: Option<&str>,
) -> AppResult<()> {
    let existing: Option<String> = c
        .query_row(
            "SELECT status FROM jobs WHERE id = ?1",
            [id],
            |r| r.get::<_, String>(0),
        )
        .optional()?;
    c.execute(
        "UPDATE jobs SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![status_str(new_status), id],
    )?;
    c.execute(
        "INSERT INTO status_history(job_id, from_status, to_status, notes)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, existing, status_str(new_status), notes],
    )?;
    Ok(())
}

/// Return the set of jobs that satisfy the current recency window.
#[derive(Debug, Clone, Deserialize)]
pub struct ListFilter {
    pub min_score: Option<f32>,
    pub status: Option<JobStatus>,
    pub recency: Option<Vec<RecencyBucket>>,
    pub company_id: Option<i64>,
    pub include_explicit_no_sponsorship: bool,
}

pub fn list_filtered(c: &DbConn, f: &ListFilter) -> AppResult<Vec<Job>> {
    let mut sql = String::from(SELECT_BASE);
    sql.push_str(" WHERE 1=1");
    if f.min_score.is_some() {
        sql.push_str(" AND match_score >= :min_score");
    }
    if f.status.is_some() {
        sql.push_str(" AND status = :status");
    }
    if let Some(list) = &f.recency {
        if !list.is_empty() {
            sql.push_str(" AND recency_bucket IN (");
            for (i, _) in list.iter().enumerate() {
                if i > 0 { sql.push(','); }
                sql.push_str(&format!(":r{}", i));
            }
            sql.push(')');
        }
    }
    if f.company_id.is_some() {
        sql.push_str(" AND company_id = :company_id");
    }
    if !f.include_explicit_no_sponsorship {
        sql.push_str(" AND sponsorship_confidence <> 'explicit_no_sponsorship'");
    }
    sql.push_str(" ORDER BY match_score DESC, posted_date DESC");

    let mut stmt = c.prepare(&sql)?;
    let mut binds: Vec<(String, Box<dyn rusqlite::ToSql>)> = Vec::new();
    if let Some(ms) = f.min_score {
        binds.push((":min_score".into(), Box::new(ms as f64)));
    }
    if let Some(st) = f.status {
        binds.push((":status".into(), Box::new(status_str(st).to_string())));
    }
    if let Some(list) = &f.recency {
        for (i, b) in list.iter().enumerate() {
            binds.push((format!(":r{}", i), Box::new(recency_str(Some(*b)).unwrap().to_string())));
        }
    }
    if let Some(cid) = f.company_id {
        binds.push((":company_id".into(), Box::new(cid)));
    }

    let refs: Vec<(&str, &dyn rusqlite::ToSql)> =
        binds.iter().map(|(k, v)| (k.as_str(), v.as_ref())).collect();
    let rows = stmt.query_map(&refs[..], map_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

// ---------------------------------------------------------------------------
// Enum conversions
// ---------------------------------------------------------------------------

fn workmode_str(w: WorkMode) -> &'static str {
    match w {
        WorkMode::Remote => "remote",
        WorkMode::Hybrid => "hybrid",
        WorkMode::Onsite => "onsite",
        WorkMode::Unknown => "unknown",
    }
}
fn parse_workmode(s: &str) -> WorkMode {
    match s {
        "remote" => WorkMode::Remote,
        "hybrid" => WorkMode::Hybrid,
        "onsite" => WorkMode::Onsite,
        _ => WorkMode::Unknown,
    }
}

fn recency_str(r: Option<RecencyBucket>) -> Option<&'static str> {
    r.map(|b| match b {
        RecencyBucket::Today => "today",
        RecencyBucket::Yesterday => "yesterday",
        RecencyBucket::Week => "week",
        RecencyBucket::TwoWeeks => "two_weeks",
        RecencyBucket::Month => "month",
        RecencyBucket::Older => "older",
    })
}
fn parse_recency(s: Option<String>) -> Option<RecencyBucket> {
    s.as_deref().and_then(|v| match v {
        "today" => Some(RecencyBucket::Today),
        "yesterday" => Some(RecencyBucket::Yesterday),
        "week" => Some(RecencyBucket::Week),
        "two_weeks" => Some(RecencyBucket::TwoWeeks),
        "month" => Some(RecencyBucket::Month),
        "older" => Some(RecencyBucket::Older),
        _ => None,
    })
}

fn sponsorship_str(s: SponsorshipConfidence) -> &'static str {
    match s {
        SponsorshipConfidence::ExplicitSponsor => "explicit_sponsor",
        SponsorshipConfidence::SponsorLikely => "sponsor_likely",
        SponsorshipConfidence::SponsorUnclear => "sponsor_unclear",
        SponsorshipConfidence::SponsorUnlikely => "sponsor_unlikely",
        SponsorshipConfidence::ExplicitNoSponsorship => "explicit_no_sponsorship",
    }
}
fn parse_sponsorship(s: &str) -> SponsorshipConfidence {
    match s {
        "explicit_sponsor" => SponsorshipConfidence::ExplicitSponsor,
        "sponsor_likely" => SponsorshipConfidence::SponsorLikely,
        "sponsor_unlikely" => SponsorshipConfidence::SponsorUnlikely,
        "explicit_no_sponsorship" => SponsorshipConfidence::ExplicitNoSponsorship,
        _ => SponsorshipConfidence::SponsorUnclear,
    }
}

fn status_str(s: JobStatus) -> &'static str {
    match s {
        JobStatus::New => "new",
        JobStatus::ResumePrepared => "resume_prepared",
        JobStatus::ReadyToApply => "ready_to_apply",
        JobStatus::Applied => "applied",
        JobStatus::OaReceived => "oa_received",
        JobStatus::Interview => "interview",
        JobStatus::Rejected => "rejected",
        JobStatus::Closed => "closed",
        JobStatus::Skipped => "skipped",
    }
}
fn parse_status(s: &str) -> JobStatus {
    match s {
        "resume_prepared" => JobStatus::ResumePrepared,
        "ready_to_apply" => JobStatus::ReadyToApply,
        "applied" => JobStatus::Applied,
        "oa_received" => JobStatus::OaReceived,
        "interview" => JobStatus::Interview,
        "rejected" => JobStatus::Rejected,
        "closed" => JobStatus::Closed,
        "skipped" => JobStatus::Skipped,
        _ => JobStatus::New,
    }
}

// ---------------------------------------------------------------------------
// Row mapping
// ---------------------------------------------------------------------------

const SELECT_BASE: &str = "SELECT
    id, company_id, source_name, source_url, apply_url, role_title,
    normalized_role_title, job_external_id, location, state_or_region,
    work_mode, posted_date, recency_bucket, jd_text, jd_summary,
    sponsorship_confidence, sponsorship_reason, match_score, match_explanation,
    role_folder_path, jd_file_path, resume_file_path, cover_letter_file_path,
    status, created_at, updated_at FROM jobs";

fn map_row(r: &Row<'_>) -> rusqlite::Result<Job> {
    let explanation_json: Option<String> = r.get(18)?;
    let match_explanation = explanation_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<MatchExplanation>(s).ok());
    let work_mode = parse_workmode(&r.get::<_, String>(10)?);
    let sponsorship = parse_sponsorship(&r.get::<_, String>(15)?);
    let status = parse_status(&r.get::<_, String>(23)?);

    Ok(Job {
        id: r.get(0)?,
        company_id: r.get(1)?,
        source_name: r.get(2)?,
        source_url: r.get(3)?,
        apply_url: r.get(4)?,
        role_title: r.get(5)?,
        normalized_role_title: r.get(6)?,
        job_external_id: r.get(7)?,
        location: r.get(8)?,
        state_or_region: r.get(9)?,
        work_mode,
        posted_date: r.get(11)?,
        recency_bucket: parse_recency(r.get(12)?),
        jd_text: r.get(13)?,
        jd_summary: r.get(14)?,
        sponsorship_confidence: sponsorship,
        sponsorship_reason: r.get(16)?,
        match_score: r.get::<_, Option<f64>>(17)?.map(|v| v as f32),
        match_explanation,
        role_folder_path: r.get(19)?,
        jd_file_path: r.get(20)?,
        resume_file_path: r.get(21)?,
        cover_letter_file_path: r.get(22)?,
        status,
        created_at: r.get(24)?,
        updated_at: r.get(25)?,
    })
}
