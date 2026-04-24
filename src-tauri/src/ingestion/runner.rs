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

    for src in sources {
        let name = src.name().to_string();
        let entry = summary.entry(name.clone()).or_insert_with(SourceOutcome::default);

        match src.fetch(client).await {
            Ok(list) => {
                entry.fetched = list.len() as u32;
                total_fetched += entry.fetched;
                for ingest in list {
                    match persist_one(db, settings, ingest).await {
                        Ok(PersistOutcome::Prepared) => {
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
pub async fn ingest_one(
    db: &Database,
    settings: &AppSettings,
    ingest: JobIngest,
) -> AppResult<i64> {
    match persist_one(db, settings, ingest).await? {
        PersistOutcome::Prepared => {
            // Fetch the newest job id we just wrote — list() is already
            // score-desc, so the row we just updated is easily findable.
            // Callers that need the id can query separately; here we return 0
            // as a sentinel, or extend persist_one to return the id.
            Ok(0)
        }
        PersistOutcome::FilteredOut => Ok(-1),
    }
}

enum PersistOutcome {
    Prepared,
    FilteredOut,
}

async fn persist_one(
    db: &Database,
    settings: &AppSettings,
    mut ingest: JobIngest,
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

    // File tree (only if score meets threshold or posted today/yesterday —
    // we always give today's jobs a folder so the user can triage fast).
    let should_prepare = score >= settings.min_match_score
        || matches!(
            bucket,
            Some(crate::domain::job::RecencyBucket::Today)
                | Some(crate::domain::job::RecencyBucket::Yesterday)
        );
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
                // Derive the per-company folder (one level above role).
                if let Some(parent) = std::path::Path::new(&paths.role_folder).parent() {
                    repo::companies::set_folder_path(&c, company.id, &parent.to_string_lossy())?;
                }
            }
        }
    }

    Ok(PersistOutcome::Prepared)
}

fn summarize(jd: Option<&str>) -> Option<String> {
    let text = jd?;
    // First non-empty ~2 sentences, capped to 400 chars.
    let mut out = String::new();
    for line in text.lines() {
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

fn truncate_err(s: &str) -> String {
    s.chars().take(200).collect()
}
