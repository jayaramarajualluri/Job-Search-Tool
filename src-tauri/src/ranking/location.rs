//! Location sub-score.  Remote-anywhere-US wins; CA and FL get a boost;
//! everything else in the US stays eligible (including NY).  Non-US is
//! heavily penalized but not zero.

use crate::domain::job::{Job, WorkMode};
use crate::domain::settings::AppSettings;

pub fn score(job: &Job, _s: &AppSettings) -> f32 {
    // Remote-anywhere-US dominates.
    if job.work_mode == WorkMode::Remote {
        if is_remote_us(job.location.as_deref()) {
            return 1.00;
        }
        // Remote but geo-restricted or unspecified — still a strong match.
        return 0.85;
    }

    match job.state_or_region.as_deref() {
        Some("CA") => 0.88,
        Some("FL") => {
            // Small extra boost for central/south Florida hints.
            if let Some(l) = job.location.as_deref() {
                let lower = l.to_lowercase();
                if lower.contains("miami")
                    || lower.contains("orlando")
                    || lower.contains("tampa")
                    || lower.contains("fort lauderdale")
                    || lower.contains("west palm")
                {
                    return 0.90;
                }
            }
            0.85
        }
        Some(code) if is_us_state_code(code) => 0.70, // NY included here
        Some(_) | None => {
            // Unknown — slight preference for unknowns that *mention* the US.
            if let Some(l) = job.location.as_deref() {
                let l = l.to_lowercase();
                if l.contains("united states") || l.contains("usa") || l.contains("u.s") {
                    return 0.55;
                }
                if l.contains("canada") || l.contains("uk") || l.contains("emea")
                    || l.contains("europe") || l.contains("india") || l.contains("apac")
                {
                    return 0.20;
                }
            }
            0.45
        }
    }
}

fn is_remote_us(loc: Option<&str>) -> bool {
    let Some(l) = loc else { return true }; // remote + no location = assume US
    let l = l.to_lowercase();
    if l.contains("canada") || l.contains("emea") || l.contains("europe")
        || l.contains("uk") || l.contains("india") || l.contains("apac")
    {
        return false;
    }
    l.contains("us") || l.contains("u.s") || l.contains("united states")
        || l.contains("anywhere") || l.contains("remote")
}

fn is_us_state_code(c: &str) -> bool {
    matches!(
        c,
        "AL"|"AK"|"AZ"|"AR"|"CA"|"CO"|"CT"|"DE"|"DC"|"FL"|"GA"|"HI"|"ID"|"IL"|"IN"
        |"IA"|"KS"|"KY"|"LA"|"ME"|"MD"|"MA"|"MI"|"MN"|"MS"|"MO"|"MT"|"NE"|"NV"|"NH"
        |"NJ"|"NM"|"NY"|"NC"|"ND"|"OH"|"OK"|"OR"|"PA"|"RI"|"SC"|"SD"|"TN"|"TX"|"UT"
        |"VT"|"VA"|"WA"|"WV"|"WI"|"WY"|"PR"
    )
}
