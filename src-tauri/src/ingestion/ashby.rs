//! Ashby public board connector.
//! `GET https://api.ashbyhq.com/posting-api/job-board/{slug}`

use crate::domain::job::{JobIngest, SourceName, WorkMode};
use crate::error::AppResult;
use serde::Deserialize;

#[derive(Deserialize)]
struct Resp {
    jobs: Vec<Posting>,
}

#[derive(Deserialize)]
struct Posting {
    id: String,
    title: String,
    #[serde(rename = "jobUrl")]
    job_url: Option<String>,
    #[serde(rename = "applicationUrl")]
    application_url: Option<String>,
    #[serde(rename = "descriptionHtml")]
    description_html: Option<String>,
    #[serde(rename = "locationName")]
    location_name: Option<String>,
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
    #[serde(default)]
    #[serde(rename = "isRemote")]
    is_remote: Option<bool>,
}

pub async fn fetch(client: &reqwest::Client, slug: &str) -> AppResult<Vec<JobIngest>> {
    let url = format!("https://api.ashbyhq.com/posting-api/job-board/{}", slug);
    let resp: Resp = client.get(url).send().await?.error_for_status()?.json().await?;
    Ok(resp.jobs.into_iter().map(|p| to_ingest(slug, p)).collect())
}

fn to_ingest(slug: &str, p: Posting) -> JobIngest {
    let jd = p.description_html.as_deref().map(super::normalize::html_to_text);
    JobIngest {
        company_name: slug.to_string(),
        source_name: SourceName::Ashby,
        source_url: p.job_url.clone(),
        apply_url: p.application_url.or(p.job_url),
        role_title: p.title,
        job_external_id: Some(p.id),
        location: p.location_name,
        work_mode: match p.is_remote {
            Some(true) => WorkMode::Remote,
            _ => WorkMode::Unknown,
        },
        posted_date: p.published_date.as_deref().and_then(super::normalize::iso_to_date),
        jd_text: jd,
        required_skills: vec![],
        preferred_skills: vec![],
    }
}
