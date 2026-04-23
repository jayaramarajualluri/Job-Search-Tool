//! LinkedIn discovery-only flow.  The user pastes a LinkedIn job URL; we try
//! to resolve the official employer / ATS URL and treat *that* as the source
//! of truth.  If we cannot resolve, we keep the LI URL as a provisional
//! pointer and mark the resolution uncertain — the app still works without
//! LinkedIn access.

use crate::error::{AppError, AppResult};
use scraper::{Html, Selector};
use url::Url;

const ATS_HOSTS: &[&str] = &[
    "boards.greenhouse.io",
    "job-boards.greenhouse.io",
    "jobs.lever.co",
    "jobs.ashbyhq.com",
    "myworkdayjobs.com",
    "icims.com",
    "smartrecruiters.com",
    "jobs.smartrecruiters.com",
    "careers.oracle.com",
    "workday.com",
];

#[derive(Debug, Clone)]
pub struct Resolution {
    /// Best-guess canonical URL (either an ATS URL or the LI URL).
    pub canonical_url: String,
    /// Set if we are confident `canonical_url` is an ATS/employer URL.
    pub resolved_to_ats: bool,
    pub ats_host: Option<String>,
    pub title_hint: Option<String>,
    pub company_hint: Option<String>,
}

pub async fn resolve(client: &reqwest::Client, li_url: &str) -> AppResult<Resolution> {
    let parsed = Url::parse(li_url)?;
    if !parsed
        .host_str()
        .map(|h| h.ends_with("linkedin.com"))
        .unwrap_or(false)
    {
        return Err(AppError::invalid("not a linkedin URL"));
    }

    // Follow redirects; LinkedIn sometimes links via a redirect hop.
    let resp = client.get(li_url).send().await?;
    let final_url = resp.url().clone();
    let body = resp.text().await?;

    // 1) If the redirect landed off-LinkedIn, that *is* the resolution.
    if let Some(host) = final_url.host_str() {
        if let Some(h) = match_ats(host) {
            return Ok(Resolution {
                canonical_url: final_url.to_string(),
                resolved_to_ats: true,
                ats_host: Some(h.to_string()),
                title_hint: None,
                company_hint: None,
            });
        }
    }

    // 2) Parse the LI page for og:url / canonical / apply-link hints.
    let doc = Html::parse_document(&body);
    let og = meta_content(&doc, r#"meta[property="og:url"]"#);
    let canonical = attr(&doc, r#"link[rel="canonical"]"#, "href");
    let title = meta_content(&doc, r#"meta[property="og:title"]"#);
    let company =
        meta_content(&doc, r#"meta[name="author"]"#).or_else(|| {
            attr(&doc, r#"a[data-tracking-control-name="public_jobs_topcard-org-name"]"#, "href")
        });

    // The "apply" anchor: LI wraps external apply URLs here.
    let apply_href = attr(&doc, "a.apply-button", "href")
        .or_else(|| attr(&doc, "a[data-tracking-control-name=\"public_jobs_apply-link-offsite\"]", "href"));

    for candidate in [apply_href.as_deref(), og.as_deref(), canonical.as_deref()]
        .into_iter()
        .flatten()
    {
        if let Ok(u) = Url::parse(candidate) {
            if let Some(host) = u.host_str() {
                if let Some(h) = match_ats(host) {
                    return Ok(Resolution {
                        canonical_url: u.to_string(),
                        resolved_to_ats: true,
                        ats_host: Some(h.to_string()),
                        title_hint: title.clone(),
                        company_hint: company.clone(),
                    });
                }
            }
        }
    }

    // 3) Couldn't resolve — keep LI URL as provisional, surface the hints.
    Ok(Resolution {
        canonical_url: li_url.to_string(),
        resolved_to_ats: false,
        ats_host: None,
        title_hint: title,
        company_hint: company,
    })
}

fn match_ats(host: &str) -> Option<&'static str> {
    ATS_HOSTS
        .iter()
        .copied()
        .find(|h| host == *h || host.ends_with(&format!(".{}", h)))
}

fn meta_content(doc: &Html, selector: &str) -> Option<String> {
    attr(doc, selector, "content")
}

fn attr(doc: &Html, selector: &str, name: &str) -> Option<String> {
    let sel = Selector::parse(selector).ok()?;
    doc.select(&sel)
        .next()
        .and_then(|el| el.value().attr(name).map(|s| s.to_string()))
}
