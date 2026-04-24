//! Ingestion commands: run all configured sources, import one URL, import
//! pasted text, resolve a LinkedIn URL.

use crate::db::repo;
use crate::db::Database;
use crate::domain::ingestion_run::IngestionRun;
use crate::error::AppResult;
use crate::ingestion::{http, linkedin_discovery, manual, runner, Source};
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum SourceSpec {
    Greenhouse { slug: String },
    Lever { slug: String },
    Ashby { slug: String },
}

impl From<SourceSpec> for Source {
    fn from(s: SourceSpec) -> Self {
        match s {
            SourceSpec::Greenhouse { slug } => Source::Greenhouse { slug },
            SourceSpec::Lever { slug } => Source::Lever { slug },
            SourceSpec::Ashby { slug } => Source::Ashby { slug },
        }
    }
}

#[tauri::command]
pub async fn run_ingestion(
    db: State<'_, Database>,
    sources: Vec<SourceSpec>,
) -> AppResult<runner::IngestionOutcome> {
    let settings = {
        let c = db.conn()?;
        repo::settings::load(&c)?
    };
    let srcs: Vec<Source> = sources.into_iter().map(Source::from).collect();
    runner::run(db.inner(), &settings, &srcs).await
}

#[tauri::command]
pub async fn import_url(
    db: State<'_, Database>,
    url: String,
    company_name: String,
    role_title: String,
    jd_text: Option<String>,
) -> AppResult<i64> {
    let ingest = manual::from_url(&url, &company_name, &role_title, jd_text)?;
    let settings = {
        let c = db.conn()?;
        repo::settings::load(&c)?
    };
    runner::ingest_one(db.inner(), &settings, ingest).await
}

#[tauri::command]
pub async fn import_text(
    db: State<'_, Database>,
    company_name: String,
    role_title: String,
    jd_text: String,
) -> AppResult<i64> {
    let ingest = manual::from_text(&company_name, &role_title, jd_text);
    let settings = {
        let c = db.conn()?;
        repo::settings::load(&c)?
    };
    runner::ingest_one(db.inner(), &settings, ingest).await
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedinResolution {
    pub canonical_url: String,
    pub resolved_to_ats: bool,
    pub ats_host: Option<String>,
    pub title_hint: Option<String>,
    pub company_hint: Option<String>,
}

#[tauri::command]
pub async fn resolve_linkedin_url(url: String) -> AppResult<LinkedinResolution> {
    let client = http::client()?;
    let r = linkedin_discovery::resolve(client, &url).await?;
    Ok(LinkedinResolution {
        canonical_url: r.canonical_url,
        resolved_to_ats: r.resolved_to_ats,
        ats_host: r.ats_host,
        title_hint: r.title_hint,
        company_hint: r.company_hint,
    })
}

#[tauri::command]
pub fn list_runs(db: State<'_, Database>) -> AppResult<Vec<IngestionRun>> {
    let c = db.conn()?;
    repo::ingestion_runs::list(&c)
}
