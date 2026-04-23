//! Skills / keywords / experience sub-scores.
//!
//! - Skills: fraction of the user's target skills that appear in the JD
//!   (case-insensitive, alias-aware; the caller resolved the list).
//! - Keywords: a softer overlap that doesn't penalize for scale.
//! - Experience: bag-of-words proxy for seniority keywords.

pub fn score(jd_skills: &[String], target_skills: &[String]) -> (f32, f32, f32) {
    if target_skills.is_empty() {
        return (0.5, 0.5, 0.5);
    }
    let jd: std::collections::BTreeSet<String> =
        jd_skills.iter().map(|s| s.to_lowercase()).collect();
    let target: std::collections::BTreeSet<String> =
        target_skills.iter().map(|s| s.to_lowercase()).collect();

    let intersect = target.intersection(&jd).count() as f32;
    let coverage = if target.is_empty() {
        0.0
    } else {
        intersect / target.len() as f32
    };

    // Jaccard — balances under/over-specification.
    let union = target.union(&jd).count() as f32;
    let jacc = if union == 0.0 { 0.0 } else { intersect / union };

    // Experience: lightweight proxy.  The JD-keyword pipeline fills this
    // out later; for now keep it neutral so it doesn't dominate.
    let experience = 0.6;

    (coverage, jacc, experience)
}
