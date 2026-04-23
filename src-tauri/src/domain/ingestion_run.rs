use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestionRun {
    pub id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub source_summary: Option<SourceSummary>,
    pub total_fetched: u32,
    pub total_filtered: u32,
    pub total_prepared: u32,
    pub notes: Option<String>,
}

/// Per-source outcome map.  Key is the source_name (e.g. "greenhouse").
pub type SourceSummary = BTreeMap<String, SourceOutcome>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SourceOutcome {
    pub fetched: u32,
    pub filtered: u32,
    pub prepared: u32,
    pub errors: Vec<String>,
}
