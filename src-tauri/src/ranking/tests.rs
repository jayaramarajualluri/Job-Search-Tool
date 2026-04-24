//! Ranking sanity tests.  These run under `cargo test`; they exercise
//! the scoring pieces end-to-end on representative inputs.

#![cfg(test)]

use crate::domain::job::{
    Job, JobStatus, RecencyBucket, SponsorshipConfidence, WorkMode,
};
use crate::domain::settings::AppSettings;
use crate::ranking::{location, recency, score_job, sponsorship, ScoreInputs};

fn mk_job(
    state: Option<&str>,
    work_mode: WorkMode,
    bucket: Option<RecencyBucket>,
    sponsorship: SponsorshipConfidence,
    location_text: Option<&str>,
    title: &str,
) -> Job {
    Job {
        id: 1,
        company_id: 1,
        source_name: "manual".into(),
        source_url: None,
        apply_url: None,
        role_title: title.into(),
        normalized_role_title: title.to_lowercase(),
        job_external_id: None,
        location: location_text.map(str::to_string),
        state_or_region: state.map(str::to_string),
        work_mode,
        posted_date: None,
        recency_bucket: bucket,
        jd_text: None,
        jd_summary: None,
        sponsorship_confidence: sponsorship,
        sponsorship_reason: None,
        match_score: None,
        match_explanation: None,
        role_folder_path: None,
        jd_file_path: None,
        resume_file_path: None,
        cover_letter_file_path: None,
        status: JobStatus::New,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

#[test]
fn remote_us_outranks_ca_onsite() {
    let remote = mk_job(
        None,
        WorkMode::Remote,
        Some(RecencyBucket::Today),
        SponsorshipConfidence::SponsorUnclear,
        Some("Remote, United States"),
        "Backend Engineer",
    );
    let onsite_ca = mk_job(
        Some("CA"),
        WorkMode::Onsite,
        Some(RecencyBucket::Today),
        SponsorshipConfidence::SponsorUnclear,
        Some("San Francisco, CA"),
        "Backend Engineer",
    );

    let settings = AppSettings::default();
    let (remote_score, _) = score_job(ScoreInputs {
        job: &remote,
        jd_skills: &[],
        settings: &settings,
    });
    let (ca_score, _) = score_job(ScoreInputs {
        job: &onsite_ca,
        jd_skills: &[],
        settings: &settings,
    });
    assert!(
        remote_score > ca_score,
        "remote-US should outrank onsite-CA: {remote_score} vs {ca_score}"
    );
}

#[test]
fn ny_included_not_penalized_below_other_states() {
    // NY and IL should rank equally on location.
    let ny = mk_job(
        Some("NY"),
        WorkMode::Onsite,
        Some(RecencyBucket::Today),
        SponsorshipConfidence::SponsorUnclear,
        Some("New York, NY"),
        "Backend Engineer",
    );
    let il = mk_job(
        Some("IL"),
        WorkMode::Onsite,
        Some(RecencyBucket::Today),
        SponsorshipConfidence::SponsorUnclear,
        Some("Chicago, IL"),
        "Backend Engineer",
    );
    let s = AppSettings::default();
    assert!((location::score(&ny, &s) - location::score(&il, &s)).abs() < 1e-6);
}

#[test]
fn florida_central_gets_extra_boost() {
    let tampa = mk_job(
        Some("FL"),
        WorkMode::Onsite,
        Some(RecencyBucket::Today),
        SponsorshipConfidence::SponsorUnclear,
        Some("Tampa, FL"),
        "Backend Engineer",
    );
    let other_fl = mk_job(
        Some("FL"),
        WorkMode::Onsite,
        Some(RecencyBucket::Today),
        SponsorshipConfidence::SponsorUnclear,
        Some("Jacksonville, FL"),
        "Backend Engineer",
    );
    let s = AppSettings::default();
    assert!(location::score(&tampa, &s) > location::score(&other_fl, &s));
}

#[test]
fn explicit_no_sponsorship_zero_contribution() {
    let job = mk_job(
        Some("CA"),
        WorkMode::Remote,
        Some(RecencyBucket::Today),
        SponsorshipConfidence::ExplicitNoSponsorship,
        Some("Remote, US"),
        "Backend Engineer",
    );
    assert_eq!(sponsorship::score(&job), 0.0);
}

#[test]
fn recency_window_membership() {
    assert!(recency::in_window(Some(RecencyBucket::Today), 7));
    assert!(recency::in_window(Some(RecencyBucket::Week), 7));
    assert!(!recency::in_window(Some(RecencyBucket::TwoWeeks), 7));
    assert!(recency::in_window(Some(RecencyBucket::TwoWeeks), 14));
}
