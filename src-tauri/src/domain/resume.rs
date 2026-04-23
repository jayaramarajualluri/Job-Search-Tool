use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReuseStrategy {
    New,
    Reused,
    EditedReuse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resume {
    pub id: i64,
    pub job_id: i64,
    pub base_resume_name: String,
    pub resume_variant_name: String,
    pub source_resume_path: Option<String>,
    pub output_resume_path: Option<String>,
    pub reuse_strategy: ReuseStrategy,
    pub similarity_score: Option<f32>,
    pub created_at: String,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Canonical resume profile — the truthful source of a resume.  Tailoring may
// reorder / re-emphasize, but never fabricate.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalProfile {
    pub name: String,
    pub headline: Option<String>,
    pub contact: Contact,
    pub summary: Option<String>,
    pub skills: Vec<SkillGroup>,
    pub experience: Vec<ExperienceEntry>,
    pub projects: Vec<ProjectEntry>,
    pub education: Vec<EducationEntry>,
    pub certifications: Vec<CertificationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub linkedin: Option<String>,
    pub github: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillGroup {
    pub category: String,          // e.g. "Languages", "Cloud", "ML"
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperienceEntry {
    pub company: String,
    pub title: String,
    pub location: Option<String>,
    pub start_date: String,        // YYYY-MM or YYYY-MM-DD
    pub end_date: Option<String>,  // None => present
    /// Full truthful bullet list.  Tailoring selects/reorders a subset.
    pub bullets: Vec<String>,
    /// Optional per-bullet skill tags, parallel to `bullets`.  Used to
    /// weight which bullets to keep when tailoring to a JD.
    #[serde(default)]
    pub bullet_tags: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectEntry {
    pub name: String,
    pub link: Option<String>,
    pub bullets: Vec<String>,
    #[serde(default)]
    pub bullet_tags: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EducationEntry {
    pub institution: String,
    pub degree: String,
    pub field: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub gpa: Option<String>,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificationEntry {
    pub name: String,
    pub issuer: Option<String>,
    pub date: Option<String>,
}
