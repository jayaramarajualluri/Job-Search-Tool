//! Recency sub-score.  Today is best; after 28 days we cut sharply.

use crate::domain::job::RecencyBucket;

pub fn score(b: Option<RecencyBucket>) -> f32 {
    match b {
        Some(RecencyBucket::Today) => 1.00,
        Some(RecencyBucket::Yesterday) => 0.95,
        Some(RecencyBucket::Week) => 0.85,
        Some(RecencyBucket::TwoWeeks) => 0.65,
        Some(RecencyBucket::Month) => 0.40,
        Some(RecencyBucket::Older) => 0.15,
        None => 0.00,
    }
}

/// Minimum recency bucket for a given "stop" in days.  Used by the window
/// expander: stage 1 keeps today+yesterday, stage 2 extends to a week, etc.
pub fn bucket_for_days(days: u32) -> RecencyBucket {
    match days {
        0..=1 => RecencyBucket::Yesterday,
        2..=7 => RecencyBucket::Week,
        8..=14 => RecencyBucket::TwoWeeks,
        15..=28 => RecencyBucket::Month,
        _ => RecencyBucket::Older,
    }
}

/// Should bucket `b` be included in the window currently expanded to
/// `max_days`?
pub fn in_window(b: Option<RecencyBucket>, max_days: u32) -> bool {
    let Some(b) = b else { return false };
    let rank = match b {
        RecencyBucket::Today => 0,
        RecencyBucket::Yesterday => 1,
        RecencyBucket::Week => 7,
        RecencyBucket::TwoWeeks => 14,
        RecencyBucket::Month => 28,
        RecencyBucket::Older => u32::MAX,
    };
    rank <= max_days
}
