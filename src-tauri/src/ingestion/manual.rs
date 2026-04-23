//! Manual import: user pastes a URL or raw JD text.  For URLs we sniff the
//! host to pick a better `source_name`; we do not scrape the page for JD
//! content here (that's the caller's job if desired).

use crate::domain::job::{JobIngest, SourceName, WorkMode};
use crate::error::{AppError, AppResult};
use url::Url;

/// Build a `JobIngest` from a user-pasted URL + optional overrides.  Caller
/// supplies the human fields we can't infer (title, company) — the UI asks
/// for these on the Import screen.
pub fn from_url(
    url: &str,
    company_name: &str,
    role_title: &str,
    jd_text: Option<String>,
) -> AppResult<JobIngest> {
    let parsed = Url::parse(url).map_err(|_| AppError::invalid("invalid URL"))?;
    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    let source_name = sniff_source(&host);
    Ok(JobIngest {
        company_name: company_name.trim().to_string(),
        source_name,
        source_url: Some(url.to_string()),
        apply_url: Some(url.to_string()),
        role_title: role_title.trim().to_string(),
        job_external_id: None,
        location: None,
        work_mode: WorkMode::Unknown,
        posted_date: None,
        jd_text,
        required_skills: vec![],
        preferred_skills: vec![],
    })
}

/// Build a `JobIngest` from pasted JD text plus title/company.
pub fn from_text(company_name: &str, role_title: &str, jd_text: String) -> JobIngest {
    JobIngest {
        company_name: company_name.trim().to_string(),
        source_name: SourceName::Manual,
        source_url: None,
        apply_url: None,
        role_title: role_title.trim().to_string(),
        job_external_id: None,
        location: None,
        work_mode: WorkMode::Unknown,
        posted_date: None,
        jd_text: Some(jd_text),
        required_skills: vec![],
        preferred_skills: vec![],
    }
}

fn sniff_source(host: &str) -> SourceName {
    if host.contains("greenhouse.io") {
        SourceName::Greenhouse
    } else if host.contains("lever.co") {
        SourceName::Lever
    } else if host.contains("ashbyhq.com") {
        SourceName::Ashby
    } else if host.contains("indeed.") {
        SourceName::Indeed
    } else if host.contains("linkedin.com") {
        SourceName::Linkedin
    } else if host.contains("myworkdayjobs")
        || host.contains("icims")
        || host.contains("smartrecruiters")
    {
        SourceName::EmployerPortal
    } else {
        SourceName::Other
    }
}
