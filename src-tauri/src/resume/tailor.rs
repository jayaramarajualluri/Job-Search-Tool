//! Tailor a `CanonicalProfile` to a job description.
//!
//! Rules (honesty first):
//!   - Never invent skills, companies, titles, dates, bullets, or metrics.
//!   - Only select/reorder bullets and skills that already exist in the
//!     canonical profile.
//!   - A bullet "matches" the JD when any of its tags (or any token inside
//!     it) overlaps the JD skills set.
//!   - Skills block is filtered+reordered: matched skills first, in category.
//!   - Target a 1-page resume by capping selected bullets per role and
//!     projects count; caller can tune caps via `TailoringRequest`.

use crate::domain::resume::{CanonicalProfile, ExperienceEntry, ProjectEntry, SkillGroup};
use serde::Serialize;
use std::collections::BTreeSet;

/// Inputs to the tailoring pass.  `jd_skills` is the already-extracted,
/// already-alias-resolved set from the normalizer.
#[derive(Debug, Clone)]
pub struct TailoringRequest<'a> {
    pub profile: &'a CanonicalProfile,
    pub jd_skills: &'a [String],
    pub jd_keywords: &'a [String],
    pub max_experience_bullets: usize,
    pub max_projects: usize,
}

impl<'a> TailoringRequest<'a> {
    pub fn new(profile: &'a CanonicalProfile) -> Self {
        Self {
            profile,
            jd_skills: &[],
            jd_keywords: &[],
            max_experience_bullets: 4,
            max_projects: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TailoredResume {
    pub name: String,
    pub headline: Option<String>,
    pub summary: Option<String>,
    pub contact: crate::domain::resume::Contact,
    pub skills: Vec<SkillGroup>,
    pub experience: Vec<TailoredExperience>,
    pub projects: Vec<TailoredProject>,
    pub education: Vec<crate::domain::resume::EducationEntry>,
    pub certifications: Vec<crate::domain::resume::CertificationEntry>,
    pub variant_name: String,
    pub tokens_matched: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TailoredExperience {
    pub company: String,
    pub title: String,
    pub location: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TailoredProject {
    pub name: String,
    pub link: Option<String>,
    pub tools: Option<String>,
    pub bullets: Vec<String>,
}

pub fn tailor(req: TailoringRequest<'_>) -> TailoredResume {
    let needle: BTreeSet<String> = req
        .jd_skills
        .iter()
        .chain(req.jd_keywords.iter())
        .map(|s| s.to_lowercase())
        .collect();

    let skills = reorder_skills(&req.profile.skills, &needle);
    let experience = req
        .profile
        .experience
        .iter()
        .map(|e| tailor_experience(e, &needle, req.max_experience_bullets))
        .collect::<Vec<_>>();
    let mut projects = req
        .profile
        .projects
        .iter()
        .map(|p| (project_relevance(p, &needle), tailor_project(p, &needle)))
        .collect::<Vec<_>>();
    projects.sort_by(|a, b| b.0.cmp(&a.0));
    let projects: Vec<_> = projects
        .into_iter()
        .take(req.max_projects)
        .map(|(_, p)| p)
        .collect();

    let tokens_matched = count_matches(&skills, &experience, &projects, &needle);
    let variant_name = format!("tailored-{}", short_hash(req.jd_skills));

    TailoredResume {
        name: req.profile.name.clone(),
        headline: req.profile.headline.clone(),
        summary: req.profile.summary.clone(),
        contact: req.profile.contact.clone(),
        skills,
        experience,
        projects,
        education: req.profile.education.clone(),
        certifications: req.profile.certifications.clone(),
        variant_name,
        tokens_matched,
    }
}

fn reorder_skills(groups: &[SkillGroup], needle: &BTreeSet<String>) -> Vec<SkillGroup> {
    groups
        .iter()
        .map(|g| {
            let hits: Vec<&String> = g
                .skills
                .iter()
                .filter(|s| needle.contains(&s.to_lowercase()))
                .collect();
            let misses: Vec<&String> = g
                .skills
                .iter()
                .filter(|s| !needle.contains(&s.to_lowercase()))
                .collect();
            let mut out = Vec::with_capacity(g.skills.len());
            out.append(&mut hits.into_iter().cloned().collect());
            out.append(&mut misses.into_iter().cloned().collect());
            SkillGroup {
                category: g.category.clone(),
                skills: out,
            }
        })
        .filter(|g| !g.skills.is_empty())
        .collect()
}

fn tailor_experience(
    e: &ExperienceEntry,
    needle: &BTreeSet<String>,
    max_bullets: usize,
) -> TailoredExperience {
    // Score each bullet: +1 per tag hit, +1 per keyword-in-text hit.
    let scored: Vec<(i32, &String)> = e
        .bullets
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let mut score = bullet_score(b, needle);
            if let Some(tags) = e.bullet_tags.get(i) {
                for t in tags {
                    if needle.contains(&t.to_lowercase()) {
                        score += 2;
                    }
                }
            }
            (score, b)
        })
        .collect();

    // Keep order of the highest-scoring bullets, then fall back to
    // original order to avoid shuffling unrelated context.
    let mut indexed: Vec<(usize, i32, &String)> = scored
        .iter()
        .enumerate()
        .map(|(i, (s, b))| (i, *s, *b))
        .collect();
    indexed.sort_by(|a, b| b.1.cmp(&a.1));
    let mut kept: Vec<(usize, &String)> =
        indexed.iter().take(max_bullets).map(|(i, _, b)| (*i, *b)).collect();
    kept.sort_by_key(|(i, _)| *i);

    TailoredExperience {
        company: e.company.clone(),
        title: e.title.clone(),
        location: e.location.clone(),
        start_date: e.start_date.clone(),
        end_date: e.end_date.clone(),
        bullets: kept.into_iter().map(|(_, b)| b.clone()).collect(),
    }
}

fn tailor_project(p: &ProjectEntry, needle: &BTreeSet<String>) -> TailoredProject {
    let bullets: Vec<String> = p
        .bullets
        .iter()
        .enumerate()
        .filter(|(i, b)| {
            bullet_score(b, needle) > 0
                || p.bullet_tags
                    .get(*i)
                    .map(|ts| ts.iter().any(|t| needle.contains(&t.to_lowercase())))
                    .unwrap_or(false)
        })
        .map(|(_, b)| b.clone())
        .collect();

    TailoredProject {
        name: p.name.clone(),
        link: p.link.clone(),
        tools: p.tools.clone(),
        bullets: if bullets.is_empty() {
            p.bullets.iter().take(2).cloned().collect()
        } else {
            bullets
        },
    }
}

fn project_relevance(p: &ProjectEntry, needle: &BTreeSet<String>) -> i32 {
    let mut score = 0;
    for b in &p.bullets {
        score += bullet_score(b, needle);
    }
    for ts in &p.bullet_tags {
        for t in ts {
            if needle.contains(&t.to_lowercase()) {
                score += 2;
            }
        }
    }
    score
}

fn bullet_score(bullet: &str, needle: &BTreeSet<String>) -> i32 {
    let hay = bullet.to_lowercase();
    needle.iter().filter(|n| hay.contains(n.as_str())).count() as i32
}

fn count_matches(
    skills: &[SkillGroup],
    experience: &[TailoredExperience],
    projects: &[TailoredProject],
    needle: &BTreeSet<String>,
) -> usize {
    let mut n = 0;
    for g in skills {
        for s in &g.skills {
            if needle.contains(&s.to_lowercase()) {
                n += 1;
            }
        }
    }
    for e in experience {
        for b in &e.bullets {
            n += bullet_score(b, needle) as usize;
        }
    }
    for p in projects {
        for b in &p.bullets {
            n += bullet_score(b, needle) as usize;
        }
    }
    n
}

fn short_hash(parts: &[String]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for p in parts {
        h.update(p.as_bytes());
        h.update(b"\n");
    }
    hex::encode(&h.finalize()[..4])
}
