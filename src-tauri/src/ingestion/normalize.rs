//! Turn raw `JobIngest` input into the shape the DB expects: strip HTML,
//! parse dates, detect state / work mode / sponsorship, and collect skills.

use crate::domain::job::{RecencyBucket, SponsorshipConfidence, WorkMode};
use chrono::NaiveDate;
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::Html;

// ---------------------------------------------------------------------------
// HTML → text
// ---------------------------------------------------------------------------

pub fn html_to_text(html: &str) -> String {
    let doc = Html::parse_fragment(html);
    let mut buf = String::with_capacity(html.len());
    for node in doc.root_element().text() {
        buf.push_str(node);
        buf.push('\n');
    }
    // Collapse consecutive blank lines.
    let cleaned: Vec<&str> = buf.lines().map(str::trim).collect();
    let mut out = String::new();
    let mut blank = false;
    for line in cleaned {
        if line.is_empty() {
            if !blank {
                out.push('\n');
            }
            blank = true;
        } else {
            out.push_str(line);
            out.push('\n');
            blank = false;
        }
    }
    out.trim().to_string()
}

// ---------------------------------------------------------------------------
// Dates
// ---------------------------------------------------------------------------

/// Accept a variety of ISO-ish timestamps and emit `YYYY-MM-DD`.
pub fn iso_to_date(s: &str) -> Option<String> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.format("%Y-%m-%d").to_string());
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(s) {
        return Some(dt.format("%Y-%m-%d").to_string());
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d.format("%Y-%m-%d").to_string());
    }
    None
}

pub fn recency_bucket(posted: Option<&str>, today: NaiveDate) -> Option<RecencyBucket> {
    let p = NaiveDate::parse_from_str(posted?, "%Y-%m-%d").ok()?;
    let days = (today - p).num_days();
    Some(match days {
        d if d <= 0 => RecencyBucket::Today,
        1 => RecencyBucket::Yesterday,
        2..=7 => RecencyBucket::Week,
        8..=14 => RecencyBucket::TwoWeeks,
        15..=28 => RecencyBucket::Month,
        _ => RecencyBucket::Older,
    })
}

// ---------------------------------------------------------------------------
// Normalized names (companies, role titles)
// ---------------------------------------------------------------------------

pub fn normalize_name(name: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    let decomp: String = name.nfkd().filter(|c| !c.is_control()).collect();
    let lower = decomp.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut prev_dash = false;
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

// ---------------------------------------------------------------------------
// Work mode detection
// ---------------------------------------------------------------------------

static RE_REMOTE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(remote|fully remote|work from home|wfh|distributed)\b").unwrap());
static RE_HYBRID: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bhybrid\b").unwrap());
static RE_ONSITE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\b(on[-\s]?site|in[-\s]?office|in person)\b").unwrap());

pub fn detect_work_mode(title: &str, location: Option<&str>, jd: Option<&str>) -> WorkMode {
    let hay = format!(
        "{} | {} | {}",
        title,
        location.unwrap_or(""),
        jd.map(|s| &s[..s.len().min(2000)]).unwrap_or("")
    );
    if RE_REMOTE.is_match(&hay) {
        WorkMode::Remote
    } else if RE_HYBRID.is_match(&hay) {
        WorkMode::Hybrid
    } else if RE_ONSITE.is_match(&hay) {
        WorkMode::Onsite
    } else {
        WorkMode::Unknown
    }
}

// ---------------------------------------------------------------------------
// US state detection
// ---------------------------------------------------------------------------

pub fn detect_state(location: Option<&str>) -> Option<String> {
    let loc = location?.trim();
    if loc.is_empty() {
        return None;
    }
    // Try trailing ", XX"
    if let Some((_, tail)) = loc.rsplit_once(',') {
        let tail = tail.trim().trim_end_matches('.');
        let upper = tail.to_ascii_uppercase();
        if upper.len() == 2 && US_STATE_CODES.contains(&upper.as_str()) {
            return Some(upper);
        }
    }
    // Try full-name match anywhere in the string
    let lower = loc.to_lowercase();
    for (name, code) in US_STATE_NAMES {
        if lower.contains(name) {
            return Some((*code).to_string());
        }
    }
    None
}

pub fn is_us_remote(location: Option<&str>) -> bool {
    let Some(loc) = location else { return false };
    let l = loc.to_lowercase();
    if !RE_REMOTE.is_match(&l) {
        return false;
    }
    l.contains("us") || l.contains("u.s") || l.contains("united states")
        || l.contains("anywhere") || !l.contains("canada") && !l.contains("emea")
            && !l.contains("uk") && !l.contains("europe")
}

// ---------------------------------------------------------------------------
// Sponsorship signal detection
// ---------------------------------------------------------------------------

static RE_NO_SPONSOR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(no\s+sponsorship|cannot\s+sponsor|not\s+able\s+to\s+sponsor|unrestricted\s+(work\s+)?authorization|must\s+be\s+(a\s+)?(us|u\.s\.|united\s+states)\s+(citizen|national)|us\s+citizen(ship)?\s+required|green\s+card\s+required|security\s+clearance|active\s+clearance)").unwrap()
});
static RE_EXPLICIT_SPONSOR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(will\s+sponsor|sponsors?\s+(h[-\s]?1b|work\s+visas?)|visa\s+sponsorship\s+(available|provided|offered))").unwrap()
});
static RE_LIKELY_SPONSOR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(open\s+to\s+sponsorship|sponsorship\s+considered|may\s+sponsor)").unwrap()
});
static RE_UNLIKELY_SPONSOR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(prefer\s+us\s+authorization|sponsorship\s+not\s+guaranteed|limited\s+sponsorship)").unwrap()
});

pub fn detect_sponsorship(jd: Option<&str>) -> (SponsorshipConfidence, Option<String>) {
    let Some(text) = jd else {
        return (SponsorshipConfidence::SponsorUnclear, None);
    };
    if let Some(m) = RE_NO_SPONSOR.find(text) {
        return (
            SponsorshipConfidence::ExplicitNoSponsorship,
            Some(format!("matched: \"{}\"", truncate(m.as_str(), 80))),
        );
    }
    if let Some(m) = RE_EXPLICIT_SPONSOR.find(text) {
        return (
            SponsorshipConfidence::ExplicitSponsor,
            Some(format!("matched: \"{}\"", truncate(m.as_str(), 80))),
        );
    }
    if let Some(m) = RE_LIKELY_SPONSOR.find(text) {
        return (
            SponsorshipConfidence::SponsorLikely,
            Some(format!("matched: \"{}\"", truncate(m.as_str(), 80))),
        );
    }
    if let Some(m) = RE_UNLIKELY_SPONSOR.find(text) {
        return (
            SponsorshipConfidence::SponsorUnlikely,
            Some(format!("matched: \"{}\"", truncate(m.as_str(), 80))),
        );
    }
    (SponsorshipConfidence::SponsorUnclear, None)
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n).collect::<String>() + "…"
    }
}

// ---------------------------------------------------------------------------
// Skill extraction
// ---------------------------------------------------------------------------

/// Find occurrences of `target_skills` (or their aliases) in `jd`.  Case-
/// insensitive whole-word match.  Returns the canonical spellings.
pub fn extract_skills(
    jd: Option<&str>,
    target_skills: &[String],
    aliases: &[(String, Vec<String>)],
) -> Vec<String> {
    let Some(text) = jd else { return vec![] };
    let hay = text.to_lowercase();
    let mut found: std::collections::BTreeSet<String> = Default::default();

    for skill in target_skills {
        if contains_word(&hay, &skill.to_lowercase()) {
            found.insert(skill.clone());
        }
    }
    for (canonical, aliases) in aliases {
        for a in aliases {
            if contains_word(&hay, &a.to_lowercase()) {
                found.insert(canonical.clone());
                break;
            }
        }
    }
    found.into_iter().collect()
}

fn contains_word(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    // `\b` in Rust regex is ASCII-only, which is fine for tech keywords.
    let pat = format!(r"(?i)\b{}\b", regex::escape(needle));
    Regex::new(&pat).map(|r| r.is_match(haystack)).unwrap_or(false)
}

// ---------------------------------------------------------------------------
// US state tables
// ---------------------------------------------------------------------------

const US_STATE_CODES: &[&str] = &[
    "AL","AK","AZ","AR","CA","CO","CT","DE","DC","FL","GA","HI","ID","IL","IN",
    "IA","KS","KY","LA","ME","MD","MA","MI","MN","MS","MO","MT","NE","NV","NH",
    "NJ","NM","NY","NC","ND","OH","OK","OR","PA","RI","SC","SD","TN","TX","UT",
    "VT","VA","WA","WV","WI","WY","PR",
];

const US_STATE_NAMES: &[(&str, &str)] = &[
    ("alabama","AL"),("alaska","AK"),("arizona","AZ"),("arkansas","AR"),
    ("california","CA"),("colorado","CO"),("connecticut","CT"),("delaware","DE"),
    ("district of columbia","DC"),("florida","FL"),("georgia","GA"),("hawaii","HI"),
    ("idaho","ID"),("illinois","IL"),("indiana","IN"),("iowa","IA"),("kansas","KS"),
    ("kentucky","KY"),("louisiana","LA"),("maine","ME"),("maryland","MD"),
    ("massachusetts","MA"),("michigan","MI"),("minnesota","MN"),("mississippi","MS"),
    ("missouri","MO"),("montana","MT"),("nebraska","NE"),("nevada","NV"),
    ("new hampshire","NH"),("new jersey","NJ"),("new mexico","NM"),("new york","NY"),
    ("north carolina","NC"),("north dakota","ND"),("ohio","OH"),("oklahoma","OK"),
    ("oregon","OR"),("pennsylvania","PA"),("rhode island","RI"),("south carolina","SC"),
    ("south dakota","SD"),("tennessee","TN"),("texas","TX"),("utah","UT"),
    ("vermont","VT"),("virginia","VA"),("washington","WA"),("west virginia","WV"),
    ("wisconsin","WI"),("wyoming","WY"),("puerto rico","PR"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_no_sponsorship() {
        let (c, _) = detect_sponsorship(Some("Must be US citizen or permanent resident."));
        assert_eq!(c, SponsorshipConfidence::ExplicitNoSponsorship);
    }

    #[test]
    fn detects_explicit_sponsorship() {
        let (c, _) = detect_sponsorship(Some("We will sponsor H-1B visas for this role."));
        assert_eq!(c, SponsorshipConfidence::ExplicitSponsor);
    }

    #[test]
    fn state_from_suffix() {
        assert_eq!(detect_state(Some("San Francisco, CA")), Some("CA".into()));
        assert_eq!(detect_state(Some("Austin, Texas")), Some("TX".into()));
    }

    #[test]
    fn recency_buckets() {
        let today = NaiveDate::from_ymd_opt(2026, 4, 23).unwrap();
        assert_eq!(
            recency_bucket(Some("2026-04-23"), today),
            Some(RecencyBucket::Today)
        );
        assert_eq!(
            recency_bucket(Some("2026-04-22"), today),
            Some(RecencyBucket::Yesterday)
        );
        assert_eq!(
            recency_bucket(Some("2026-04-18"), today),
            Some(RecencyBucket::Week)
        );
        assert_eq!(
            recency_bucket(Some("2026-04-10"), today),
            Some(RecencyBucket::TwoWeeks)
        );
    }

    #[test]
    fn normalize_company() {
        assert_eq!(normalize_name("Acme Co., Inc."), "acme-co-inc");
        assert_eq!(normalize_name("Über Corp"), "uber-corp");
    }
}
