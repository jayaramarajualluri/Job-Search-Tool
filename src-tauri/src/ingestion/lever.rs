//! Lever public board connector.
//! `GET https://api.lever.co/v0/postings/{slug}?mode=json`

use crate::domain::job::{JobIngest, SourceName, WorkMode};
use crate::error::AppResult;
use serde::Deserialize;

#[derive(Deserialize)]
struct Posting {
    id: String,
    text: String,
    #[serde(rename = "hostedUrl")]
    hosted_url: Option<String>,
    #[serde(rename = "applyUrl")]
    apply_url: Option<String>,
    #[serde(rename = "descriptionPlain")]
    description_plain: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: Option<i64>,       // ms since epoch
    #[serde(default)]
    categories: Categories,
}

#[derive(Deserialize, Default)]
struct Categories {
    location: Option<String>,
    commitment: Option<String>,    // Full-time, etc.
    #[serde(default)]
    #[serde(rename = "allLocations")]
    all_locations: Vec<String>,
}

pub async fn fetch(client: &reqwest::Client, slug: &str) -> AppResult<Vec<JobIngest>> {
    let url = format!("https://api.lever.co/v0/postings/{}?mode=json", slug);
    let postings: Vec<Posting> =
        client.get(url).send().await?.error_for_status()?.json().await?;
    Ok(postings.into_iter().map(|p| to_ingest(slug, p)).collect())
}

fn to_ingest(slug: &str, p: Posting) -> JobIngest {
    let location = p
        .categories
        .location
        .or_else(|| p.categories.all_locations.into_iter().next());
    let posted = p.created_at.map(|ms| {
        let secs = ms / 1000;
        chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)
            .unwrap_or_default()
            .format("%Y-%m-%d")
            .to_string()
    });
    JobIngest {
        company_name: slug.to_string(),
        source_name: SourceName::Lever,
        source_url: p.hosted_url.clone(),
        apply_url: p.apply_url.or(p.hosted_url),
        role_title: p.text,
        job_external_id: Some(p.id),
        location,
        work_mode: WorkMode::Unknown,
        posted_date: posted,
        jd_text: p.description_plain,
        required_skills: vec![],
        preferred_skills: vec![],
    }
}
