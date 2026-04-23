//! Scoring engine.  Each sub-scorer returns `[0, 1]`; `score_job` combines
//! them into a 0..100 `match_score` plus a per-term `MatchExplanation`
//! that the UI renders to explain why a job ranked where it did.

pub mod location;
pub mod recency;
pub mod skills;
pub mod sponsorship;

use crate::domain::job::{Job, MatchExplanation, RecencyBucket, WorkMode};
use crate::domain::settings::{AppSettings, RemotePreference};

/// Weights sum to 1.0.
const W_TITLE: f32 = 0.15;
const W_SKILLS: f32 = 0.25;
const W_KEYWORDS: f32 = 0.10;
const W_EXPERIENCE: f32 = 0.05;
const W_LOCATION: f32 = 0.15;
const W_WORK_MODE: f32 = 0.10;
const W_SPONSORSHIP: f32 = 0.15;
const W_RECENCY: f32 = 0.05;

pub struct ScoreInputs<'a> {
    pub job: &'a Job,
    pub jd_skills: &'a [String],
    pub settings: &'a AppSettings,
}

pub fn score_job(inp: ScoreInputs<'_>) -> (f32, MatchExplanation) {
    let title = title_fit(inp.job, inp.settings);
    let (skills, keywords, experience) =
        skills::score(inp.jd_skills, &inp.settings.target_skills);
    let location = location::score(inp.job, inp.settings);
    let work_mode = work_mode_fit(inp.job.work_mode, inp.settings.remote_preference);
    let sponsorship = sponsorship::score(inp.job);
    let recency = recency::score(inp.job.recency_bucket);

    let composite = W_TITLE * title
        + W_SKILLS * skills
        + W_KEYWORDS * keywords
        + W_EXPERIENCE * experience
        + W_LOCATION * location
        + W_WORK_MODE * work_mode
        + W_SPONSORSHIP * sponsorship
        + W_RECENCY * recency;

    let mut notes = Vec::new();
    if sponsorship == 0.0 {
        notes.push("explicit no-sponsorship signal detected".into());
    }
    if recency == 0.0 {
        notes.push("no posted date available".into());
    }
    if location < 0.5 {
        notes.push("location outside preferred set".into());
    }

    let explanation = MatchExplanation {
        title,
        skills,
        keywords,
        experience,
        location,
        work_mode,
        sponsorship,
        recency,
        notes,
    };

    ((composite * 100.0).clamp(0.0, 100.0), explanation)
}

// ---------------------------------------------------------------------------
// Title fit — overlap between job title tokens and preferred role tokens.
// ---------------------------------------------------------------------------

fn title_fit(job: &Job, s: &AppSettings) -> f32 {
    if s.preferred_roles.is_empty() {
        return 0.5;
    }
    let job_tokens: std::collections::BTreeSet<String> = tokenize(&job.role_title).collect();
    if job_tokens.is_empty() {
        return 0.0;
    }
    let mut best: f32 = 0.0;
    for role in &s.preferred_roles {
        let role_tokens: std::collections::BTreeSet<String> = tokenize(role).collect();
        if role_tokens.is_empty() {
            continue;
        }
        let inter = job_tokens.intersection(&role_tokens).count() as f32;
        let union = job_tokens.union(&role_tokens).count() as f32;
        let jacc = inter / union.max(1.0);
        best = best.max(jacc);
    }
    best
}

fn tokenize(s: &str) -> impl Iterator<Item = String> + '_ {
    s.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() > 2)
        .map(|t| t.to_ascii_lowercase())
}

// ---------------------------------------------------------------------------
// Work mode fit, caller-parameterized by the user's remote preference.
// ---------------------------------------------------------------------------

fn work_mode_fit(mode: WorkMode, pref: RemotePreference) -> f32 {
    match (mode, pref) {
        (WorkMode::Remote, _) => 1.0,
        (WorkMode::Hybrid, RemotePreference::RemoteFirst) => 0.55,
        (WorkMode::Hybrid, RemotePreference::HybridOk) => 0.85,
        (WorkMode::Hybrid, RemotePreference::OnsiteOk) => 0.75,
        (WorkMode::Onsite, RemotePreference::RemoteFirst) => 0.35,
        (WorkMode::Onsite, RemotePreference::HybridOk) => 0.55,
        (WorkMode::Onsite, RemotePreference::OnsiteOk) => 0.80,
        (WorkMode::Unknown, _) => 0.60,
    }
}

/// Map a recency bucket to a human-readable label for the UI.
pub fn recency_label(b: Option<RecencyBucket>) -> &'static str {
    match b {
        Some(RecencyBucket::Today) => "Today",
        Some(RecencyBucket::Yesterday) => "Yesterday",
        Some(RecencyBucket::Week) => "Last 7 days",
        Some(RecencyBucket::TwoWeeks) => "Last 14 days",
        Some(RecencyBucket::Month) => "Last 28 days",
        Some(RecencyBucket::Older) | None => "Older / unknown",
    }
}
