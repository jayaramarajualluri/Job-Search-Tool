//! End-to-end ingestion runner: pull from configured sources, normalize,
//! score, persist, and generate folders/files.  This is the single glue
//! point between the ingestion layer, the normalizer, the ranker, the
//! file organizer, and the DB.

use crate::db::repo;
use crate::db::Database;
use crate::domain::ingestion_run::{SourceOutcome, SourceSummary};
use crate::domain::job::{JobIngest, SponsorshipConfidence, WorkMode};
use crate::domain::settings::AppSettings;
use crate::error::AppResult;
use crate::files;
use crate::ingestion::normalize;
use crate::ingestion::{http, Source};
use crate::ranking::{recency, score_job, ScoreInputs};
use chrono::Utc;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestionOutcome {
    pub run_id: i64,
    pub total_fetched: u32,
    pub total_filtered: u32,
    pub total_prepared: u32,
}

/// Run the configured sources and persist every resulting job.  `sources`
/// comes from settings (not hard-coded) so the user owns the list of boards.
pub async fn run(db: &Database, settings: &AppSettings, sources: &[Source]) -> AppResult<IngestionOutcome> {
    let run_id = {
        let c = db.conn()?;
        repo::ingestion_runs::start(&c)?
    };

    let client = http::client()?;
    let mut summary: SourceSummary = Default::default();
    let mut total_fetched = 0u32;
    let mut total_filtered = 0u32;
    let mut total_prepared = 0u32;

    let cap = settings.max_jobs_to_prepare_per_run as usize;

    for src in sources {
        let name = src.name().to_string();
        let entry = summary.entry(name.clone()).or_insert_with(SourceOutcome::default);

        match src.fetch(client).await {
            Ok(list) => {
                entry.fetched = list.len() as u32;
                total_fetched += entry.fetched;
                for ingest in list {
                    let already_at_cap = total_prepared as usize >= cap;
                    match persist_one(db, settings, ingest, false, already_at_cap).await {
                        Ok(PersistOutcome::Prepared(_)) => {
                            entry.prepared += 1;
                            total_prepared += 1;
                            entry.filtered += 1;
                            total_filtered += 1;
                        }
                        Ok(PersistOutcome::FilteredOut) => {
                            entry.filtered += 1;
                            total_filtered += 1;
                        }
                        Err(e) => entry.errors.push(truncate_err(&e.to_string())),
                    }
                }
            }
            Err(e) => entry.errors.push(truncate_err(&e.to_string())),
        }
    }

    {
        let c = db.conn()?;
        repo::ingestion_runs::finish(
            &c,
            run_id,
            &summary,
            (total_fetched, total_filtered, total_prepared),
            None,
        )?;
    }
    Ok(IngestionOutcome {
        run_id,
        total_fetched,
        total_filtered,
        total_prepared,
    })
}

/// Run ingestion for a single manually-supplied `JobIngest` (URL or text
/// import path).  Reuses the same normalize → persist → file path.
/// Returns the new job id on success, or `-1` if the input was filtered out
/// by the recency window or by the explicit-no-sponsorship rule.
pub async fn ingest_one(
    db: &Database,
    settings: &AppSettings,
    ingest: JobIngest,
) -> AppResult<i64> {
    match persist_one(db, settings, ingest, true, false).await? {
        PersistOutcome::Prepared(id) => Ok(id),
        PersistOutcome::FilteredOut => Ok(-1),
    }
}

enum PersistOutcome {
    Prepared(i64),
    FilteredOut,
}

async fn persist_one(
    db: &Database,
    settings: &AppSettings,
    mut ingest: JobIngest,
    force_prepare: bool,
    skip_new_file_trees: bool,
) -> AppResult<PersistOutcome> {
    // Detect work mode (if the source didn't already).
    if matches!(ingest.work_mode, WorkMode::Unknown) {
        ingest.work_mode = normalize::detect_work_mode(
            &ingest.role_title,
            ingest.location.as_deref(),
            ingest.jd_text.as_deref(),
        );
    }

    // Sponsorship signal
    let (sponsorship, reason) = normalize::detect_sponsorship(ingest.jd_text.as_deref());
    if matches!(sponsorship, SponsorshipConfidence::ExplicitNoSponsorship)
        && settings.exclude_explicit_no_sponsorship
    {
        return Ok(PersistOutcome::FilteredOut);
    }

    // Skills extraction
    let aliases: Vec<(String, Vec<String>)> = settings
        .skill_aliases
        .iter()
        .map(|p| (p.canonical.clone(), p.aliases.clone()))
        .collect();
    let jd_skills = normalize::extract_skills(
        ingest.jd_text.as_deref(),
        &settings.target_skills,
        &aliases,
    );

    // Recency bucket
    let today = Utc::now().date_naive();
    let bucket = normalize::recency_bucket(ingest.posted_date.as_deref(), today);

    // Upsert company + job
    let state = normalize::detect_state(ingest.location.as_deref());
    let normalized_role = normalize::normalize_name(&ingest.role_title);

    let (job_id, job) = {
        let c = db.conn()?;
        let company = repo::companies::upsert_by_name(&c, &ingest.company_name)?;
        let insert = repo::jobs::JobInsert {
            company_id: company.id,
            source_name: ingest.source_name.as_str().to_string(),
            source_url: ingest.source_url.clone(),
            apply_url: ingest.apply_url.clone(),
            role_title: ingest.role_title.clone(),
            normalized_role_title: normalized_role,
            job_external_id: ingest.job_external_id.clone(),
            location: ingest.location.clone(),
            state_or_region: state,
            work_mode: ingest.work_mode,
            posted_date: ingest.posted_date.clone(),
            recency_bucket: bucket,
            jd_text: ingest.jd_text.clone(),
            jd_summary: summarize(ingest.jd_text.as_deref()),
            sponsorship_confidence: sponsorship,
            sponsorship_reason: reason,
        };
        let job_id = repo::jobs::upsert(&c, &insert)?;
        let job = repo::jobs::get(&c, job_id)?.expect("just inserted");
        (job_id, job)
    };

    // Score
    let (score, explanation) = score_job(ScoreInputs {
        job: &job,
        jd_skills: &jd_skills,
        settings,
    });

    // If recency is out of the configured max window and the job has a
    // date, mark it filtered.  Undated jobs still flow through (we can't
    // tell — better to keep and let the UI show them).
    let max_days = settings.recency_thresholds.max_days;
    if job.posted_date.is_some() && !recency::in_window(bucket, max_days) {
        return Ok(PersistOutcome::FilteredOut);
    }

    {
        let c = db.conn()?;
        repo::jobs::update_match(&c, job_id, score, &explanation)?;
    }

    // File tree: skip if job already has a folder (avoid re-creating on repeat runs),
    // or if we've hit the per-run cap and this isn't a forced single-job import.
    let already_has_folder = job.role_folder_path.is_some();
    let should_prepare = !already_has_folder
        && !skip_new_file_trees
        && (force_prepare
            || score >= settings.min_match_score
            || matches!(
                bucket,
                Some(crate::domain::job::RecencyBucket::Today)
                    | Some(crate::domain::job::RecencyBucket::Yesterday)
            ));

    if should_prepare {
        let root = PathBuf::from(&settings.root_folder);
        let paths = files::folder::build_job_tree(
            &root,
            job.posted_date.as_deref(),
            &ingest,
            summarize(ingest.jd_text.as_deref()).as_deref(),
        )?;
        let c = db.conn()?;
        repo::jobs::update_paths(
            &c,
            job_id,
            &paths.role_folder,
            &paths.jd_file,
            None,
            None,
        )?;
        // Persist company folder if first seen.
        if let Some(company) = repo::companies::get(&c, job.company_id)? {
            if company.company_folder_path.is_none() {
                if let Some(parent) = std::path::Path::new(&paths.role_folder).parent() {
                    let parent_str = parent.to_string_lossy().into_owned();
                    repo::companies::set_folder_path(&c, company.id, &parent_str)?;
                }
            }
        }
    }

    Ok(PersistOutcome::Prepared(job_id))
}

fn summarize(jd: Option<&str>) -> Option<String> {
    let text = jd?;
    let stripped = strip_html(text);
    let mut out = String::new();
    for line in stripped.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(l);
        if out.len() >= 400 {
            break;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out.chars().take(400).collect::<String>())
    }
}

fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Collapse runs of whitespace
    let mut result = String::with_capacity(out.len());
    let mut prev_space = false;
    for ch in out.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                result.push(' ');
            }
            prev_space = true;
        } else {
            result.push(ch);
            prev_space = false;
        }
    }
    result.trim().to_string()
}

fn truncate_err(s: &str) -> String {
    s.chars().take(200).collect()
}
