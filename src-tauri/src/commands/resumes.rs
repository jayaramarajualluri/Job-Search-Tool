//! Resume commands: load/save canonical profile, tailor for a job, list
//! variants for a job.
//!
//! Tailor flow:
//!   1. Rule-based bullet selection (TailoringRequest).
//!   2. If settings.ai_rewrite_enabled AND the AI key is in keychain,
//!      pipe bullets per role/project through Claude with strict honesty
//!      constraints.  Failures fall back per-section to rule-based output.
//!   3. Render HTML.
//!   4. Best-effort headless-Chrome → PDF.  Prefer PDF path when it
//!      succeeds, otherwise return the HTML path with the error noted in
//!      the success log.

use crate::db::repo;
use crate::db::Database;
use crate::domain::resume::{CanonicalProfile, Resume, ReuseStrategy};
use crate::error::{AppError, AppResult};
use crate::ingestion::normalize;
use crate::resume::{ai_rewrite, pdf, renderer, reuse, tailor, TailoringRequest};
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

/// Tailor the canonical profile against the given job, optionally pipe
/// the bullets through Claude (when enabled), render HTML, and best-effort
/// render PDF.  Returns the absolute path to whichever file (PDF preferred)
/// is the canonical resume output for that job.
#[tauri::command]
pub async fn tailor_resume_for_job(
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

    // Extract JD skills for tailoring + AI prompt.
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

    let mut tailored = tailor::tailor(TailoringRequest {
        profile: &profile,
        jd_skills: &jd_skills,
        jd_keywords: &[],
        max_experience_bullets: 4,
        max_projects: 3,
    });

    // Optional AI rewrite pass.  Per-section best-effort — if Claude is
    // unavailable, malformed, or returns the wrong bullet count, we keep
    // the rule-based output for that section.
    if settings.ai_rewrite_enabled {
        match crate::secrets::keyring::get_named("ai_api_key") {
            Ok(Some(api_key)) => {
                let jd_excerpt = job.jd_text.as_deref().unwrap_or("");
                for exp in tailored.experience.iter_mut() {
                    if exp.bullets.is_empty() {
                        continue;
                    }
                    match ai_rewrite::rewrite_bullets(
                        &api_key,
                        &settings.ai_model,
                        &exp.bullets,
                        &jd_skills,
                        jd_excerpt,
                    )
                    .await
                    {
                        Ok(rewritten) => exp.bullets = rewritten,
                        Err(e) => tracing::warn!(error = %e, role = %exp.title, "AI rewrite failed; keeping rule-based"),
                    }
                }
                for proj in tailored.projects.iter_mut() {
                    if proj.bullets.is_empty() {
                        continue;
                    }
                    match ai_rewrite::rewrite_bullets(
                        &api_key,
                        &settings.ai_model,
                        &proj.bullets,
                        &jd_skills,
                        jd_excerpt,
                    )
                    .await
                    {
                        Ok(rewritten) => proj.bullets = rewritten,
                        Err(e) => tracing::warn!(error = %e, project = %proj.name, "AI rewrite failed; keeping rule-based"),
                    }
                }
            }
            Ok(None) => {
                tracing::warn!("ai_rewrite_enabled but no API key in keychain; skipping AI step");
            }
            Err(e) => {
                tracing::warn!(error = %e, "keychain read failed for ai_api_key; skipping AI step");
            }
        }
    }

    // Reuse-similarity decision (same as before).
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

    // Write HTML.
    let Some(role_folder) = job.role_folder_path.as_deref() else {
        return Err(AppError::invalid(
            "job has no role_folder_path; run ingestion first",
        ));
    };
    let html_path = PathBuf::from(role_folder).join("tailored_resume.html");
    renderer::render_html_to(&tailored, &html_path)?;

    // Best-effort PDF.
    let pdf_path = PathBuf::from(role_folder).join("tailored_resume.pdf");
    let final_path = match pdf::html_to_pdf(&html_path, &pdf_path) {
        Ok(()) => {
            tracing::info!(path = %pdf_path.display(), "rendered PDF");
            pdf_path
        }
        Err(e) => {
            tracing::warn!(error = %e, "PDF render failed; falling back to HTML");
            html_path.clone()
        }
    };

    // Bind owned strings before the DB calls.
    let final_path_str = final_path.to_string_lossy().into_owned();
    let source_profile_path = settings.resume_source_inputs.canonical_profile_path.clone();
    let jd_file_path = job.jd_file_path.clone().unwrap_or_default();
    let cover_letter_path = job.cover_letter_file_path.clone();

    {
        let c = db.conn()?;
        repo::resumes::insert(
            &c,
            job_id,
            "master",
            &tailored.variant_name,
            source_profile_path.as_deref(),
            Some(&final_path_str),
            strategy,
            similarity_score,
        )?;
        repo::jobs::update_paths(
            &c,
            job_id,
            role_folder,
            &jd_file_path,
            Some(&final_path_str),
            cover_letter_path.as_deref(),
        )?;
    }
    Ok(final_path_str)
}
