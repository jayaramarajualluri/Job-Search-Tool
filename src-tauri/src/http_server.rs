//! Axum HTTP API — mirrors every Tauri command as a REST endpoint.
//! All handlers share Arc<Database> via axum State.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::fs as tfs;

use crate::db::repo;
use crate::db::Database;
use crate::domain::account::AccountInput;
use crate::domain::resume::CanonicalProfile;
use crate::domain::settings::AppSettings;
use crate::error::AppError;
use crate::ingestion::{runner, Source};
use crate::resume::{renderer, tailor, TailoringRequest};
use crate::{files, ingestion::normalize};
use std::path::PathBuf;

pub type Db = Arc<Database>;

// ── Error wrapper ────────────────────────────────────────────────────────────

pub struct ApiError(AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self { Self(e) }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Invalid(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.0.to_string()).into_response()
    }
}

type R<T> = Result<Json<T>, ApiError>;

// ── Router ────────────────────────────────────────────────────────────────────

async fn ping() -> &'static str { "pong" }

pub fn router() -> Router<Db> {
    Router::new()
        .route("/ping", get(ping))
        // jobs
        .route("/jobs", get(list_jobs))
        .route("/jobs/:id", get(get_job))
        .route("/jobs/:id/status", patch(update_job_status))
        // companies
        .route("/companies", get(list_companies))
        .route("/companies/:id/notes", patch(update_company_notes))
        // accounts
        .route("/accounts", get(list_accounts).post(create_account))
        .route("/accounts/:id", put(update_account).delete(delete_account))
        .route("/accounts/:id/password", post(save_account_password)
            .delete(clear_account_password)
            .get(reveal_account_password))
        .route("/accounts/:id/used", post(mark_account_used))
        // settings
        .route("/settings", get(load_settings).put(save_settings))
        // ingestion
        .route("/ingestion/run", post(run_ingestion))
        .route("/ingestion/import-url", post(import_url))
        .route("/ingestion/import-text", post(import_text))
        .route("/ingestion/runs", get(list_runs))
        // resumes
        .route("/resumes/canonical-profile", get(load_canonical_profile).post(save_canonical_profile))
        .route("/resumes/for-job/:job_id", get(list_resumes_for_job))
        .route("/resumes/tailor/:job_id", post(tailor_resume_for_job))
        .route("/resumes/tailor-ai/:job_id", post(tailor_resume_for_job_ai))
        // files (serve file content for download/preview)
        .route("/files/serve", get(serve_file))
}

// ── Jobs ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct JobsQuery {
    #[serde(rename = "minScore")]   min_score: Option<f32>,
    status: Option<String>,
    #[serde(rename = "companyId")]  company_id: Option<i64>,
    #[serde(rename = "includeExplicitNoSponsorship")]
    include_explicit_no_sponsorship: Option<bool>,
}

async fn list_jobs(State(db): State<Db>, Query(q): Query<JobsQuery>) -> R<serde_json::Value> {
    let c = db.conn()?;
    let jobs = repo::jobs::list_filtered(
        &c,
        q.min_score,
        q.status.as_deref(),
        None,   // recency — add later if needed
        q.company_id,
        q.include_explicit_no_sponsorship,
    )?;
    Ok(Json(serde_json::to_value(jobs).unwrap()))
}

async fn get_job(State(db): State<Db>, Path(id): Path<i64>) -> R<serde_json::Value> {
    let c = db.conn()?;
    let job = repo::jobs::get(&c, id)?;
    Ok(Json(serde_json::to_value(job).unwrap()))
}

#[derive(Deserialize)]
struct StatusBody { status: String, notes: Option<String> }

async fn update_job_status(
    State(db): State<Db>,
    Path(id): Path<i64>,
    Json(body): Json<StatusBody>,
) -> Result<StatusCode, ApiError> {
    let c = db.conn()?;
    repo::jobs::update_status(&c, id, &body.status, body.notes.as_deref())?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Companies ─────────────────────────────────────────────────────────────────

async fn list_companies(State(db): State<Db>) -> R<serde_json::Value> {
    let c = db.conn()?;
    Ok(Json(serde_json::to_value(repo::companies::list(&c)?).unwrap()))
}

#[derive(Deserialize)]
struct NotesBody { notes: Option<String> }

async fn update_company_notes(
    State(db): State<Db>,
    Path(id): Path<i64>,
    Json(body): Json<NotesBody>,
) -> Result<StatusCode, ApiError> {
    let c = db.conn()?;
    repo::companies::update_notes(&c, id, body.notes.as_deref())?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Accounts ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AccountsQuery { #[serde(rename = "companyId")] company_id: Option<i64> }

async fn list_accounts(State(db): State<Db>, Query(q): Query<AccountsQuery>) -> R<serde_json::Value> {
    let c = db.conn()?;
    Ok(Json(serde_json::to_value(repo::accounts::list(&c, q.company_id)?).unwrap()))
}

async fn create_account(
    State(db): State<Db>,
    Json(input): Json<AccountInput>,
) -> R<serde_json::Value> {
    let c = db.conn()?;
    let id = repo::accounts::create(
        &c,
        input.company_id,
        input.portal_type,
        input.login_url.as_deref(),
        input.username.as_deref(),
        input.requires_2fa,
        input.notes.as_deref(),
    )?;
    // Store password directly in credential_key (web mode — no OS keychain)
    if let Some(pw) = &input.password {
        repo::accounts::set_credential_key(&c, id, Some(pw))?;
    }
    let acc = repo::accounts::get(&c, id)?.expect("just created");
    Ok(Json(serde_json::to_value(acc).unwrap()))
}

#[derive(Deserialize)]
struct UpdateAccountBody {
    #[serde(rename = "loginUrl")]   login_url: Option<String>,
    username: Option<String>,
    #[serde(rename = "requires2fa")] requires_2fa: bool,
    notes: Option<String>,
}

async fn update_account(
    State(db): State<Db>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateAccountBody>,
) -> Result<StatusCode, ApiError> {
    let c = db.conn()?;
    repo::accounts::update(
        &c, id,
        body.login_url.as_deref(),
        body.username.as_deref(),
        body.requires_2fa,
        body.notes.as_deref(),
    )?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct PasswordBody { password: String }

async fn save_account_password(
    State(db): State<Db>,
    Path(id): Path<i64>,
    Json(body): Json<PasswordBody>,
) -> Result<StatusCode, ApiError> {
    let c = db.conn()?;
    repo::accounts::set_credential_key(&c, id, Some(&body.password))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn clear_account_password(
    State(db): State<Db>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let c = db.conn()?;
    repo::accounts::set_credential_key(&c, id, None)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn reveal_account_password(
    State(db): State<Db>,
    Path(id): Path<i64>,
) -> R<serde_json::Value> {
    let c = db.conn()?;
    let acc = repo::accounts::get(&c, id)?
        .ok_or_else(|| AppError::not_found("account"))?;
    Ok(Json(serde_json::json!(acc.credential_key)))
}

async fn mark_account_used(
    State(db): State<Db>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let c = db.conn()?;
    repo::accounts::mark_used(&c, id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_account(
    State(db): State<Db>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let c = db.conn()?;
    repo::accounts::delete(&c, id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Settings ──────────────────────────────────────────────────────────────────

async fn load_settings(State(db): State<Db>) -> R<AppSettings> {
    let c = db.conn()?;
    Ok(Json(repo::settings::load(&c)?))
}

async fn save_settings(
    State(db): State<Db>,
    Json(settings): Json<AppSettings>,
) -> Result<StatusCode, ApiError> {
    let c = db.conn()?;
    repo::settings::save(&c, &settings)?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Ingestion ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SourceSpec {
    kind: String,
    slug: String,
}

async fn run_ingestion(
    State(db): State<Db>,
    Json(sources): Json<Vec<SourceSpec>>,
) -> R<runner::IngestionOutcome> {
    let settings = { let c = db.conn()?; repo::settings::load(&c)? };
    let srcs: Vec<Source> = sources.into_iter().filter_map(|s| match s.kind.as_str() {
        "greenhouse" => Some(Source::Greenhouse { slug: s.slug }),
        "lever"      => Some(Source::Lever      { slug: s.slug }),
        "ashby"      => Some(Source::Ashby      { slug: s.slug }),
        _            => None,
    }).collect();
    let outcome = runner::run(&db, &settings, &srcs).await?;
    Ok(Json(outcome))
}

#[derive(Deserialize)]
struct ImportUrlBody {
    url: String,
    #[serde(rename = "companyName")] company_name: String,
    #[serde(rename = "roleTitle")]   role_title: String,
    #[serde(rename = "jdText")]      jd_text: Option<String>,
}

async fn import_url(
    State(db): State<Db>,
    Json(body): Json<ImportUrlBody>,
) -> R<i64> {
    let settings = { let c = db.conn()?; repo::settings::load(&c)? };
    let ingest = crate::ingestion::manual::from_url(
        &body.url, &body.company_name, &body.role_title, body.jd_text,
    )?;
    let id = runner::ingest_one(&db, &settings, ingest).await?;
    Ok(Json(id))
}

#[derive(Deserialize)]
struct ImportTextBody {
    #[serde(rename = "companyName")] company_name: String,
    #[serde(rename = "roleTitle")]   role_title: String,
    #[serde(rename = "jdText")]      jd_text: String,
}

async fn import_text(
    State(db): State<Db>,
    Json(body): Json<ImportTextBody>,
) -> R<i64> {
    let settings = { let c = db.conn()?; repo::settings::load(&c)? };
    let ingest = crate::ingestion::manual::from_text(
        &body.company_name, &body.role_title, body.jd_text,
    );
    let id = runner::ingest_one(&db, &settings, ingest).await?;
    Ok(Json(id))
}

async fn list_runs(State(db): State<Db>) -> R<serde_json::Value> {
    let c = db.conn()?;
    Ok(Json(serde_json::to_value(repo::ingestion_runs::list(&c)?).unwrap()))
}

// ── Resumes ───────────────────────────────────────────────────────────────────

async fn load_canonical_profile(State(db): State<Db>) -> R<serde_json::Value> {
    let c = db.conn()?;
    let settings = repo::settings::load(&c)?;
    let Some(path) = settings.resume_source_inputs.canonical_profile_path else {
        return Ok(Json(serde_json::Value::Null));
    };
    let text = std::fs::read_to_string(&path)?;
    let profile: CanonicalProfile = serde_json::from_str(&text)?;
    Ok(Json(serde_json::to_value(profile).unwrap()))
}

async fn save_canonical_profile(
    State(db): State<Db>,
    Json(profile): Json<CanonicalProfile>,
) -> R<String> {
    let c = db.conn()?;
    let mut settings = repo::settings::load(&c)?;
    let path = settings.resume_source_inputs.canonical_profile_path
        .clone()
        .unwrap_or_else(|| {
            PathBuf::from(&settings.root_folder)
                .join("master_resume.json")
                .to_string_lossy()
                .into_owned()
        });
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(&profile)?)?;
    settings.resume_source_inputs.canonical_profile_path = Some(path.clone());
    repo::settings::save(&c, &settings)?;
    Ok(Json(path))
}

async fn list_resumes_for_job(
    State(db): State<Db>,
    Path(job_id): Path<i64>,
) -> R<serde_json::Value> {
    let c = db.conn()?;
    Ok(Json(serde_json::to_value(repo::resumes::list_for_job(&c, job_id)?).unwrap()))
}

async fn tailor_resume_for_job(
    State(db): State<Db>,
    Path(job_id): Path<i64>,
) -> R<String> {
    use crate::resume::reuse;
    use crate::domain::resume::ReuseStrategy;

    let (job, settings, profile) = {
        let c = db.conn()?;
        let job = repo::jobs::get(&c, job_id)?
            .ok_or_else(|| AppError::not_found(format!("job {job_id}")))?;
        let settings = repo::settings::load(&c)?;
        let path = settings.resume_source_inputs.canonical_profile_path.clone()
            .ok_or_else(|| AppError::invalid("set canonical_profile_path in settings first"))?;
        let profile: CanonicalProfile = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
        (job, settings, profile)
    };

    let aliases: Vec<(String, Vec<String>)> = settings.skill_aliases.iter()
        .map(|p| (p.canonical.clone(), p.aliases.clone())).collect();
    let jd_skills = normalize::extract_skills(
        job.jd_text.as_deref(), &settings.target_skills, &aliases,
    );

    let tailored = tailor::tailor(TailoringRequest {
        profile: &profile,
        jd_skills: &jd_skills,
        jd_keywords: &[],
        max_experience_bullets: 4,
        max_projects: 3,
    });

    let role_folder = job.role_folder_path.as_deref()
        .ok_or_else(|| AppError::invalid("job has no role_folder_path; run ingestion first"))?;
    let out_path = PathBuf::from(role_folder).join("tailored_resume.html");
    renderer::render_html_to(&tailored, &out_path)?;

    let final_path = renderer::html_to_pdf(&out_path).unwrap_or(out_path.clone());
    let out_str = final_path.to_string_lossy().into_owned();

    // similarity / reuse strategy
    let (strategy, sim) = {
        let c = db.conn()?;
        let siblings = repo::jobs::list(&c)?.into_iter()
            .filter(|j| j.company_id == job.company_id && j.id != job.id);
        let mut best: f32 = 0.0;
        for s in siblings {
            if let (Some(a), Some(b)) = (s.jd_text.as_deref(), job.jd_text.as_deref()) {
                let sim = reuse::similarity(a, b);
                if sim > best { best = sim; }
            }
        }
        let strat = match reuse::decide(best) {
            reuse::ReuseDecision::Reuse => ReuseStrategy::Reused,
            reuse::ReuseDecision::EditedReuse => ReuseStrategy::EditedReuse,
            reuse::ReuseDecision::New => ReuseStrategy::New,
        };
        (strat, if best > 0.0 { Some(best) } else { None })
    };

    {
        let c = db.conn()?;
        let src_path = settings.resume_source_inputs.canonical_profile_path.clone();
        let jd_file = job.jd_file_path.clone().unwrap_or_default();
        repo::resumes::insert(&c, job_id, "master", &tailored.variant_name,
            src_path.as_deref(), Some(&out_str), strategy, sim)?;
        repo::jobs::update_paths(&c, job_id, role_folder, &jd_file,
            Some(&out_str), job.cover_letter_file_path.as_deref())?;
    }
    Ok(Json(out_str))
}

async fn tailor_resume_for_job_ai(
    State(db): State<Db>,
    Path(job_id): Path<i64>,
) -> R<String> {
    use crate::domain::resume::ReuseStrategy;

    let (job, api_key, role_folder, source_profile_path, profile) = {
        let c = db.conn()?;
        let job = repo::jobs::get(&c, job_id)?
            .ok_or_else(|| AppError::not_found(format!("job {job_id}")))?;
        let settings = repo::settings::load(&c)?;
        let api_key = settings.anthropic_api_key.clone()
            .ok_or_else(|| AppError::invalid("No Anthropic API key — add it in Settings"))?;
        let path = settings.resume_source_inputs.canonical_profile_path.clone()
            .ok_or_else(|| AppError::invalid("set canonical_profile_path in settings first"))?;
        let profile: CanonicalProfile = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
        let role_folder = job.role_folder_path.clone()
            .ok_or_else(|| AppError::invalid("job has no role_folder_path; run ingestion first"))?;
        (job, api_key, role_folder, settings.resume_source_inputs.canonical_profile_path.clone(), profile)
    };

    let jd_text = job.jd_text.as_deref().unwrap_or("").to_string();
    let role_title = job.role_title.clone();
    let profile_json = serde_json::to_string_pretty(&profile)?;

    let system = "You are an expert resume writer. Given a canonical resume profile in JSON \
        and a job description, rewrite the resume to best match the role. Rules: never invent \
        facts; only rephrase existing bullets; keep the same JSON schema; rewrite the summary \
        to address this role directly; return ONLY the JSON object with no markdown or fences.";

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| AppError::Other(e.to_string()))?;

    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "max_tokens": 4096,
        "system": system,
        "messages": [{"role": "user", "content":
            format!("Role: {role_title}\n\nJob Description:\n{jd_text}\n\nCanonical Profile:\n{profile_json}")
        }]
    });

    let resp = client.post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body).send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::invalid(format!("Anthropic API error {status}: {text}")).into());
    }

    let resp_json: serde_json::Value = resp.json().await?;
    let content = resp_json["content"][0]["text"].as_str()
        .ok_or_else(|| AppError::invalid("unexpected Anthropic response"))?;

    let tailored_profile: CanonicalProfile = serde_json::from_str(content)
        .map_err(|e| AppError::invalid(format!("AI returned invalid JSON: {e}")))?;

    let tailored = tailor::tailor(TailoringRequest {
        profile: &tailored_profile,
        jd_skills: &[],
        jd_keywords: &[],
        max_experience_bullets: usize::MAX,
        max_projects: usize::MAX,
    });

    let out_path = PathBuf::from(&role_folder).join("tailored_resume_ai.html");
    renderer::render_html_to(&tailored, &out_path)?;
    let final_path = renderer::html_to_pdf(&out_path).unwrap_or(out_path.clone());
    let out_str = final_path.to_string_lossy().into_owned();

    {
        let c = db.conn()?;
        let jd_file = job.jd_file_path.clone().unwrap_or_default();
        let variant = format!("ai-tailored-{}", &job.role_title[..job.role_title.len().min(20)]);
        repo::resumes::insert(&c, job_id, "ai", &variant,
            source_profile_path.as_deref(), Some(&out_str), ReuseStrategy::New, None)?;
        repo::jobs::update_paths(&c, job_id, &role_folder, &jd_file,
            Some(&out_str), job.cover_letter_file_path.as_deref())?;
    }
    Ok(Json(out_str))
}

// ── File serving ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ServeFileQuery { path: String }

async fn serve_file(Query(q): Query<ServeFileQuery>) -> Response {
    let path = PathBuf::from(&q.path);
    match tfs::read(&path).await {
        Ok(bytes) => {
            let mime = match path.extension().and_then(|e| e.to_str()) {
                Some("pdf")  => "application/pdf",
                Some("html") => "text/html",
                Some("json") => "application/json",
                _            => "application/octet-stream",
            };
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            let disposition = format!("inline; filename=\"{filename}\"");
            Response::builder()
                .header(header::CONTENT_TYPE, mime)
                .header(header::CONTENT_DISPOSITION, disposition)
                .body(Body::from(bytes))
                .unwrap()
        }
        Err(_) => (StatusCode::NOT_FOUND, "file not found").into_response(),
    }
}

// ── Boilerplate From impls ────────────────────────────────────────────────────

impl From<rusqlite::Error> for ApiError {
    fn from(e: rusqlite::Error) -> Self { Self(AppError::Db(e)) }
}
impl From<r2d2::Error> for ApiError {
    fn from(e: r2d2::Error) -> Self { Self(AppError::Pool(e)) }
}
impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self { Self(AppError::Io(e)) }
}
impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self { Self(AppError::Serde(e)) }
}
impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self { Self(AppError::Http(e)) }
}
