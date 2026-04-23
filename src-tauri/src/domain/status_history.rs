use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusHistoryEntry {
    pub id: i64,
    pub job_id: i64,
    pub from_status: Option<String>,
    pub to_status: String,
    pub at: String,
    pub notes: Option<String>,
}
