//! Resume commands: load/save canonical profile, tailor for a job, list
//! variants for a job.

use crate::db::repo;
use crate::db::Database;
use crate::domain::resume::{CanonicalProfile, Resume, ReuseStrategy};
use crate::error::{AppError, AppResult};
use crate::ingestion::normalize;
use crate::resume::{renderer, reuse, tailor, TailoringRequest};
use std::fs;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn load_canonical_profile(db: State<'_, Database>) -> AppResult<Option<CanonicalProfile>> {
    let c = db.conn()?;
    let settings = repo::settings::load(&c)?;
    let Some(path) = settings.resume_source_inputs.canonical_profile_path else {
        return Ok(None);
    };
    let text = fs::read_to_string(&path)?;
    let profile: CanonicalProfile = serde_json::from_str(&text)?;
    Ok(Some(profile))
}

#[tauri::command]
pub fn save_canonical_profile(
    db: State<'_, Database>,
    profile: CanonicalProfile,
) -> AppResult<String> {
    let c = db.conn()?;
    let mut settings = repo::settings::load(&c)?;
    let path = settings
        .resume_source_inputs
        .canonical_profile_path
        .clone()
        .unwrap_or_else(|| {
            PathBuf::from(&settings.root_folder)
                .join("master_resume.json")
                .to_string_lossy()
                .into_owned()
        });
    if let Some(parent) = std::path::Path::new(&path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(&profile)?)?;
    settings.resume_source_inputs.canonical_profile_path = Some(path.clone());
    repo::settings::save(&c, &settings)?;
    Ok(path)
}

#[tauri::command]
pub fn list_resumes_for_job(
    db: State<'_, Database>,
    job_id: i64,
) -> AppResult<Vec<Resume>> {
    let c = db.conn()?;
    repo::resumes::list_for_job(&c, job_id)
}

/// Tailor the canonical profile against the given job and write
/// `tailored_resume.html` into the job's role folder.  Returns the path.
#[tauri::command]
pub fn tailor_resume_for_job(
    db: State<'_, Database>,
    job_id: i64,
) -> AppResult<String> {
    let (job, settings, profile) = {
        let c = db.conn()?;
        let job = repo::jobs::get(&c, job_id)?
            .ok_or_else(|| AppError::not_found(format!("job {job_id}")))?;
        let settings = repo::settings::load(&c)?;
        let profile_path = settings
            .resume_source_inputs
            .canonical_profile_path
            .clone()
            .ok_or_else(|| {
                AppError::invalid("set canonical_profile_path in settings first")
            })?;
        let text = fs::read_to_string(&profile_path)?;
        let profile: CanonicalProfile = serde_json::from_str(&text)?;
        (job, settings, profile)
    };

    // Extract skills from the job's JD for the tailoring pass.
    let aliases: Vec<(String, Vec<String>)> = settings
        .skill_aliases
        .iter()
        .map(|p| (p.canonical.clone(), p.aliases.clone()))
        .collect();
    let jd_skills = normalize::extract_skills(
        job.jd_text.as_deref(),
        &settings.target_skills,
        &aliases,
    );

    let tailored = tailor::tailor(TailoringRequest {
        profile: &profile,
        jd_skills: &jd_skills,
        jd_keywords: &[],
        max_experience_bullets: 4,
        max_projects: 3,
    });

    // Decide: reuse an existing variant if a prior JD at the same company
    // is highly similar.
    let mut strategy = ReuseStrategy::New;
    let mut similarity_score: Option<f32> = None;
    {
        let c = db.conn()?;
        let siblings = repo::jobs::list(&c)?
            .into_iter()
            .filter(|j| j.company_id == job.company_id && j.id != job.id);
        let mut best: f32 = 0.0;
        for s in siblings {
            if let (Some(a), Some(b)) = (s.jd_text.as_deref(), job.jd_text.as_deref()) {
                let sim = reuse::similarity(a, b);
                if sim > best {
                    best = sim;
                }
            }
        }
        if best > 0.0 {
            similarity_score = Some(best);
            strategy = match reuse::decide(best) {
                reuse::ReuseDecision::Reuse => ReuseStrategy::Reused,
                reuse::ReuseDecision::EditedReuse => ReuseStrategy::EditedReuse,
                reuse::ReuseDecision::New => ReuseStrategy::New,
            };
        }
    }

    // Write the HTML resume into the role folder.
    let Some(role_folder) = job.role_folder_path.as_deref() else {
        return Err(AppError::invalid(
            "job has no role_folder_path; run ingestion first",
        ));
    };
    let out_path = PathBuf::from(role_folder).join("tailored_resume.html");
    renderer::render_html_to(&tailored, &out_path)?;

    // Bind owned strings so their references outlive the query calls below.
    let out_path_str = out_path.to_string_lossy().into_owned();
    let source_profile_path = settings.resume_source_inputs.canonical_profile_path.clone();
    let jd_file_path = job.jd_file_path.clone().unwrap_or_default();
    let cover_letter_path = job.cover_letter_file_path.clone();

    // Persist resume row + update job.resume_file_path.
    {
        let c = db.conn()?;
        repo::resumes::insert(
            &c,
            job_id,
            "master",
            &tailored.variant_name,
            source_profile_path.as_deref(),
            Some(&out_path_str),
            strategy,
            similarity_score,
        )?;
        repo::jobs::update_paths(
            &c,
            job_id,
            role_folder,
            &jd_file_path,
            Some(&out_path_str),
            cover_letter_path.as_deref(),
        )?;
    }
    Ok(out_path_str)
}

/// AI-powered resume tailoring via the Anthropic Claude API.
/// Rewrites experience bullets and the summary to best match the JD.
/// Returns the path to the written HTML file.
#[tauri::command]
pub async fn tailor_resume_for_job_ai(
    db: tauri::State<'_, Database>,
    job_id: i64,
) -> AppResult<String> {
    // ── 1. load everything we need under a short-lived db lock ──────────────
    let (job, api_key, role_folder, out_path, source_profile_path, jd_file_path, cover_letter_path, profile) = {
        let c = db.conn()?;
        let job = repo::jobs::get(&c, job_id)?
            .ok_or_else(|| AppError::not_found(format!("job {job_id}")))?;
        let settings = repo::settings::load(&c)?;
        let api_key = settings.anthropic_api_key.clone()
            .ok_or_else(|| AppError::invalid(
                "No Anthropic API key configured. Add it in Settings → Resume → Anthropic API key."
            ))?;
        let profile_path = settings.resume_source_inputs.canonical_profile_path.clone()
            .ok_or_else(|| AppError::invalid(
                "Set canonical_profile_path in Settings → Resume first."
            ))?;
        let profile_text = fs::read_to_string(&profile_path)?;
        let profile: CanonicalProfile = serde_json::from_str(&profile_text)?;
        let role_folder = job.role_folder_path.clone()
            .ok_or_else(|| AppError::invalid("job has no role_folder_path; run ingestion first"))?;
        let out_path = PathBuf::from(&role_folder).join("tailored_resume_ai.html");
        let source_profile_path = settings.resume_source_inputs.canonical_profile_path.clone();
        let jd_file_path = job.jd_file_path.clone().unwrap_or_default();
        let cover_letter_path = job.cover_letter_file_path.clone();
        (job, api_key, role_folder, out_path, source_profile_path, jd_file_path, cover_letter_path, profile)
    };

    let jd_text = job.jd_text.as_deref().unwrap_or("").to_string();
    let role_title = job.role_title.clone();

    // ── 2. build the prompt ──────────────────────────────────────────────────
    let profile_json = serde_json::to_string_pretty(&profile)?;
    let system = "You are an expert resume writer helping a job seeker tailor their resume. \
        Given a canonical resume profile in JSON and a job description, rewrite the resume. \
        Rules you MUST follow:\
        \n- Never invent achievements, companies, titles, dates, metrics, or technologies not in the profile\
        \n- You may reorder, trim, and rephrase existing bullets to emphasize relevance\
        \n- Keep the exact same JSON schema as the input\
        \n- The \"summary\" field (if present) should be rewritten to directly address this role\
        \n- Return ONLY the JSON object with no markdown, no code fences, no explanation";

    let user_msg = format!(
        "Role: {role_title}\n\nJob Description:\n{jd_text}\n\nCanonical Profile:\n{profile_json}"
    );

    // ── 3. call Anthropic API ────────────────────────────────────────────────
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "max_tokens": 4096,
        "system": system,
        "messages": [{"role": "user", "content": user_msg}]
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::invalid(format!(
            "Anthropic API error {status}: {text}"
        )));
    }

    let resp_json: serde_json::Value = resp.json().await?;
    let content = resp_json["content"][0]["text"]
        .as_str()
        .ok_or_else(|| AppError::invalid("Unexpected Anthropic response shape"))?;

    // ── 4. parse response as CanonicalProfile and render ────────────────────
    let tailored_profile: CanonicalProfile = serde_json::from_str(content)
        .map_err(|e| AppError::invalid(format!(
            "AI returned invalid JSON: {e}\n\nResponse was:\n{content}"
        )))?;

    // Convert to TailoredResume via the existing algorithmic path so renderer works
    let tailored = tailor::tailor(TailoringRequest {
        profile: &tailored_profile,
        jd_skills: &[],
        jd_keywords: &[],
        max_experience_bullets: usize::MAX,
        max_projects: usize::MAX,
    });

    renderer::render_html_to(&tailored, &out_path)?;

    let out_path_str = out_path.to_string_lossy().into_owned();

    // ── 5. persist resume row + update job.resume_file_path ─────────────────
    {
        let c = db.conn()?;
        repo::resumes::insert(
            &c,
            job_id,
            "ai",
            &format!("ai-tailored-{}", &job.role_title[..job.role_title.len().min(20)]),
            source_profile_path.as_deref(),
            Some(&out_path_str),
            ReuseStrategy::New,
            None,
        )?;
        repo::jobs::update_paths(
            &c,
            job_id,
            &role_folder,
            &jd_file_path,
            Some(&out_path_str),
            cover_letter_path.as_deref(),
        )?;
    }
    Ok(out_path_str)
}
