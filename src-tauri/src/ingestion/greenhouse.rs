//! Greenhouse public board connector.
//! `GET https://boards-api.greenhouse.io/v1/boards/{slug}/jobs?content=true`

use crate::domain::job::{JobIngest, SourceName, WorkMode};
use crate::error::AppResult;
use serde::Deserialize;

#[derive(Deserialize)]
struct Resp {
    jobs: Vec<Posting>,
}

#[derive(Deserialize)]
struct Posting {
    id: i64,
    title: String,
    #[serde(default)]
    location: Option<Location>,
    #[serde(default)]
    updated_at: Option<String>,    // ISO timestamp
    #[serde(default)]
    absolute_url: Option<String>,
    #[serde(default)]
    content: Option<String>,       // HTML JD
    #[serde(default)]
    company_name: Option<String>,
}

#[derive(Deserialize)]
struct Location {
    name: Option<String>,
}

pub async fn fetch(client: &reqwest::Client, slug: &str) -> AppResult<Vec<JobIngest>> {
    let url = format!(
        "https://boards-api.greenhouse.io/v1/boards/{}/jobs?content=true",
        urlencoding(slug)
    );
    let resp: Resp = client.get(url).send().await?.error_for_status()?.json().await?;
    Ok(resp
        .jobs
        .into_iter()
        .map(|p| to_ingest(slug, p))
        .collect())
}

fn to_ingest(slug: &str, p: Posting) -> JobIngest {
    let jd = p.content.as_deref().map(super::normalize::html_to_text);
    let loc = p.location.and_then(|l| l.name);
    let posted = p.updated_at.as_deref().and_then(super::normalize::iso_to_date);
    JobIngest {
        company_name: p.company_name.unwrap_or_else(|| slug.to_string()),
        source_name: SourceName::Greenhouse,
        source_url: p.absolute_url.clone(),
        apply_url: p.absolute_url,
        role_title: p.title,
        job_external_id: Some(p.id.to_string()),
        location: loc,
        work_mode: WorkMode::Unknown,
        posted_date: posted,
        jd_text: jd,
        required_skills: vec![],
        preferred_skills: vec![],
    }
}

fn urlencoding(s: &str) -> String {
    // Greenhouse slugs are lowercase alnum/-; still encode to be safe.
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
