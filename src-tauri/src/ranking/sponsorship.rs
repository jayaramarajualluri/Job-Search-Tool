//! Sponsorship sub-score.

use crate::domain::job::{Job, SponsorshipConfidence};

pub fn score(job: &Job) -> f32 {
    match job.sponsorship_confidence {
        SponsorshipConfidence::ExplicitSponsor => 1.00,
        SponsorshipConfidence::SponsorLikely => 0.90,
        SponsorshipConfidence::SponsorUnclear => 0.70,
        SponsorshipConfidence::SponsorUnlikely => 0.40,
        SponsorshipConfidence::ExplicitNoSponsorship => 0.00,
    }
}
