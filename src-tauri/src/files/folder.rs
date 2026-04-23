//! Build the per-job folder tree and write the supporting files.

use super::paths::{role_folder_name, safe_join, safe_slug};
use crate::domain::job::JobIngest;
use crate::error::AppResult;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Paths persisted back to the `jobs` row.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobPaths {
    pub company_folder: String,
    pub role_folder: String,
    pub jd_file: String,
    pub jd_summary_file: String,
    pub metadata_file: String,
    pub apply_link_file: String,
}

/// Create the per-job folder tree and write JD/metadata/apply-link files.
/// Caller is responsible for writing resume files via the resume renderer.
pub fn build_job_tree(
    root: &Path,
    posted_date: Option<&str>,
    ingest: &JobIngest,
    jd_summary: Option<&str>,
) -> AppResult<JobPaths> {
    let date = posted_date
        .map(|s| s.to_string())
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

    let company_seg = safe_slug(&ingest.company_name);
    let role_seg = role_folder_name(
        ingest.job_external_id.as_deref(),
        &ingest.role_title,
        first_keyword(ingest.jd_text.as_deref()),
    );

    let company_dir = safe_join(root, &[&date, &company_seg])?;
    let role_dir = safe_join(&company_dir, &[&role_seg])?;
    fs::create_dir_all(&role_dir)?;

    // jd.txt
    let jd_path = role_dir.join("jd.txt");
    if let Some(text) = ingest.jd_text.as_deref() {
        fs::write(&jd_path, text)?;
    } else {
        // Keep the file so the folder has a predictable shape even empty.
        fs::write(&jd_path, "")?;
    }

    // jd_summary.txt
    let sum_path = role_dir.join("jd_summary.txt");
    fs::write(&sum_path, jd_summary.unwrap_or(""))?;

    // metadata.json
    let meta_path = role_dir.join("metadata.json");
    let meta = JobMetadata::from(ingest, posted_date);
    fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;

    // apply_link.url — Windows shortcut (.url) format, also readable on macOS.
    let apply_link_path = role_dir.join("apply_link.url");
    let url = ingest
        .apply_url
        .as_deref()
        .or(ingest.source_url.as_deref())
        .unwrap_or("");
    fs::write(
        &apply_link_path,
        format!("[InternetShortcut]\r\nURL={url}\r\n"),
    )?;

    Ok(JobPaths {
        company_folder: to_string_path(&company_dir),
        role_folder: to_string_path(&role_dir),
        jd_file: to_string_path(&jd_path),
        jd_summary_file: to_string_path(&sum_path),
        metadata_file: to_string_path(&meta_path),
        apply_link_file: to_string_path(&apply_link_path),
    })
}

fn to_string_path(p: &PathBuf) -> String {
    p.to_string_lossy().into_owned()
}

#[derive(Serialize)]
struct JobMetadata<'a> {
    company: &'a str,
    role: &'a str,
    job_external_id: Option<&'a str>,
    source_name: &'static str,
    source_url: Option<&'a str>,
    apply_url: Option<&'a str>,
    location: Option<&'a str>,
    posted_date: Option<&'a str>,
    created_at: String,
}

impl<'a> JobMetadata<'a> {
    fn from(ingest: &'a JobIngest, posted_date: Option<&'a str>) -> Self {
        Self {
            company: &ingest.company_name,
            role: &ingest.role_title,
            job_external_id: ingest.job_external_id.as_deref(),
            source_name: ingest.source_name.as_str(),
            source_url: ingest.source_url.as_deref(),
            apply_url: ingest.apply_url.as_deref(),
            location: ingest.location.as_deref(),
            posted_date,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Pick a short keyword from the JD to use as a folder name fallback.  We
/// grab the first "title-like" line (capitalized, <=80 chars).
fn first_keyword(jd: Option<&str>) -> Option<&str> {
    jd.and_then(|text| {
        text.lines()
            .map(str::trim)
            .find(|l| !l.is_empty() && l.len() <= 80)
    })
}
