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
        return Err(AppError::invalid("job has no role_folder_path; run ingestion first"));
    };
    let out_path = PathBuf::from(role_folder).join("tailored_resume.html");
    renderer::render_html_to(&tailored, &out_path)?;

    // Persist resume row + update job.resume_file_path.
    {
        let c = db.conn()?;
        repo::resumes::insert(
            &c,
            job_id,
            "master",
            &tailored.variant_name,
            settings.resume_source_inputs.canonical_profile_path.as_deref(),
            Some(&out_path.to_string_lossy()),
            strategy,
            similarity_score,
        )?;
        repo::jobs::update_paths(
            &c,
            job_id,
            role_folder,
            job.jd_file_path.as_deref().unwrap_or(""),
            Some(&out_path.to_string_lossy()),
            job.cover_letter_file_path.as_deref(),
        )?;
    }
    Ok(out_path.to_string_lossy().into_owned())
}
