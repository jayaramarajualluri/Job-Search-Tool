use serde::{Deserialize, Serialize};

/// Where a posting was discovered.  Kept as a free-form string in the DB so
/// new sources can be added without a migration, but these are the canonical
/// values the code emits.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceName {
    Greenhouse,
    Lever,
    Ashby,
    Indeed,
    Linkedin,
    Manual,
    EmployerPortal,
    Other,
}

impl SourceName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Greenhouse => "greenhouse",
            Self::Lever => "lever",
            Self::Ashby => "ashby",
            Self::Indeed => "indeed",
            Self::Linkedin => "linkedin",
            Self::Manual => "manual",
            Self::EmployerPortal => "employer_portal",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkMode {
    Remote,
    Hybrid,
    Onsite,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecencyBucket {
    Today,
    Yesterday,
    Week,
    TwoWeeks,
    Month,
    Older,
}

/// Sponsorship confidence for a STEM OPT candidate.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SponsorshipConfidence {
    ExplicitSponsor,
    SponsorLikely,
    SponsorUnclear,
    SponsorUnlikely,
    ExplicitNoSponsorship,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    New,
    ResumePrepared,
    ReadyToApply,
    Applied,
    OaReceived,
    Interview,
    Rejected,
    Closed,
    Skipped,
}

/// Per-contribution breakdown emitted by the scoring engine (stored as JSON
/// in `jobs.match_explanation`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchExplanation {
    pub title: f32,
    pub skills: f32,
    pub keywords: f32,
    pub experience: f32,
    pub location: f32,
    pub work_mode: f32,
    pub sponsorship: f32,
    pub recency: f32,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: i64,
    pub company_id: i64,

    pub source_name: String,
    pub source_url: Option<String>,
    pub apply_url: Option<String>,

    pub role_title: String,
    pub normalized_role_title: String,
    pub job_external_id: Option<String>,

    pub location: Option<String>,
    pub state_or_region: Option<String>,
    pub work_mode: WorkMode,

    pub posted_date: Option<String>,            // YYYY-MM-DD
    pub recency_bucket: Option<RecencyBucket>,

    pub jd_text: Option<String>,
    pub jd_summary: Option<String>,

    pub sponsorship_confidence: SponsorshipConfidence,
    pub sponsorship_reason: Option<String>,

    pub match_score: Option<f32>,
    pub match_explanation: Option<MatchExplanation>,

    pub role_folder_path: Option<String>,
    pub jd_file_path: Option<String>,
    pub resume_file_path: Option<String>,
    pub cover_letter_file_path: Option<String>,

    pub status: JobStatus,
    pub created_at: String,
    pub updated_at: String,
}

/// Shape the ingestion layer produces before the row is persisted.  Omits
/// computed fields (recency_bucket, match_score, paths, status) that are
/// filled later in the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobIngest {
    pub company_name: String,
    pub source_name: SourceName,
    pub source_url: Option<String>,
    pub apply_url: Option<String>,
    pub role_title: String,
    pub job_external_id: Option<String>,
    pub location: Option<String>,
    pub work_mode: WorkMode,
    pub posted_date: Option<String>,
    pub jd_text: Option<String>,
    pub required_skills: Vec<String>,
    pub preferred_skills: Vec<String>,
}
