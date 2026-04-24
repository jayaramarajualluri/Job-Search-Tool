use serde::{Deserialize, Serialize};

/// Single key/value row.  `value` is a JSON string.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingEntry {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

/// Strongly-typed settings, hydrated from the key/value rows.  The UI edits
/// this shape and the backend writes one row per field into `settings`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Absolute path to the root folder where job asset trees are written.
    pub root_folder: String,

    /// Preferred role titles and close-fit alternatives.
    pub preferred_roles: Vec<String>,
    /// Skills the user has — master list.
    pub target_skills: Vec<String>,
    /// Synonyms / aliases, canonical → aliases.  Also persisted in DB but
    /// cached here for fast read.
    pub skill_aliases: Vec<SkillAliasPair>,

    /// Preferred US locations; remote-anywhere-US implicitly ranks highest.
    pub preferred_locations: Vec<String>,
    /// One of: remote_first | hybrid_ok | onsite_ok.  Interpreted by ranker.
    pub remote_preference: RemotePreference,

    /// How strict to be about sponsorship signals.
    pub sponsorship_sensitivity: SponsorshipSensitivity,
    /// If true, explicit-no-sponsorship jobs are auto-excluded from results.
    pub exclude_explicit_no_sponsorship: bool,

    /// Recency window expansion thresholds.  We expand when the count of
    /// jobs above `min_match_score` in the current window is below N.
    pub recency_thresholds: RecencyThresholds,
    /// Minimum match score (0..100) to count a job as "good enough" for
    /// recency-window expansion decisions.
    pub min_match_score: f32,
    /// Minimum "good matches" needed before expanding to next recency window.
    pub min_good_matches_before_expand: u32,

    /// Per-run cap to bound folder/file/resume generation work.
    pub max_jobs_to_prepare_per_run: u32,

    /// Folder naming template; tokens: {jobId}, {role}, {slug}, {date}.
    pub folder_naming_format: String,

    /// Paths of master resume inputs (canonical JSON, optional .tex, PDF).
    pub resume_source_inputs: ResumeSourceInputs,

    /// Whether the LinkedIn discovery-only flow is enabled.
    pub linkedin_discovery_enabled: bool,

    /// Board slugs to poll during scheduled ingestion, grouped by ATS.
    pub board_slugs: BoardSlugs,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BoardSlugs {
    #[serde(default)]
    pub greenhouse: Vec<String>,
    #[serde(default)]
    pub lever: Vec<String>,
    #[serde(default)]
    pub ashby: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillAliasPair {
    pub canonical: String,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemotePreference {
    RemoteFirst,
    HybridOk,
    OnsiteOk,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SponsorshipSensitivity {
    /// Drop explicit-no-sponsorship; boost explicit-sponsor.
    Strict,
    /// Keep everything except explicit-no-sponsorship (default).
    Balanced,
    /// Keep everything; only rank lower.
    Permissive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecencyThresholds {
    /// Hard upper bound in days for expansion; default 28.
    pub max_days: u32,
    /// Ordered stops.  Default: [2, 7, 14, 21, 28].
    pub stops: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResumeSourceInputs {
    pub canonical_profile_path: Option<String>,
    pub latex_source_path: Option<String>,
    pub pdf_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            root_folder: crate::config::default_root_folder()
                .to_string_lossy()
                .into_owned(),
            preferred_roles: vec![
                "Software Engineer".into(),
                "Backend Engineer".into(),
                "Data Engineer".into(),
                "Platform Engineer".into(),
                "Machine Learning Engineer".into(),
                "Cloud Engineer".into(),
            ],
            target_skills: vec![],
            skill_aliases: vec![],
            preferred_locations: vec![
                "Remote".into(),
                "California".into(),
                "Florida".into(),
            ],
            remote_preference: RemotePreference::RemoteFirst,
            sponsorship_sensitivity: SponsorshipSensitivity::Balanced,
            exclude_explicit_no_sponsorship: true,
            recency_thresholds: RecencyThresholds {
                max_days: 28,
                stops: vec![2, 7, 14, 21, 28],
            },
            min_match_score: 55.0,
            min_good_matches_before_expand: 20,
            max_jobs_to_prepare_per_run: 50,
            folder_naming_format: "{jobId|role}_{slug}".into(),
            resume_source_inputs: ResumeSourceInputs::default(),
            linkedin_discovery_enabled: true,
            board_slugs: BoardSlugs::default(),
        }
    }
}
