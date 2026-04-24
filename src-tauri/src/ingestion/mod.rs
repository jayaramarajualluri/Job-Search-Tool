//! Job ingestion.  A `Source` produces `JobIngest` records that the
//! normalizer + ranker turn into full `Job` rows.  Sources are dispatched
//! via an enum so we avoid the `async_trait` dependency and keep the call
//! sites simple.
//!
//! Adding a new source:
//!   1. Create `ingestion/<name>.rs` with `async fn fetch(client, slug)`.
//!   2. Add a variant to `Source`.
//!   3. Dispatch it in `Source::{name,fetch}` below.

use crate::domain::job::JobIngest;
use crate::error::AppResult;

pub mod ashby;
pub mod greenhouse;
pub mod http;
pub mod indeed;
pub mod lever;
pub mod linkedin_discovery;
pub mod manual;
pub mod normalize;
pub mod runner;

/// A configured source + its board slug (for ATS boards) or a free-form
/// config (for manual/discovery paths).
#[derive(Debug, Clone)]
pub enum Source {
    Greenhouse { slug: String },
    Lever { slug: String },
    Ashby { slug: String },
}

impl Source {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Greenhouse { .. } => "greenhouse",
            Self::Lever { .. } => "lever",
            Self::Ashby { .. } => "ashby",
        }
    }

    pub async fn fetch(&self, client: &reqwest::Client) -> AppResult<Vec<JobIngest>> {
        match self {
            Self::Greenhouse { slug } => greenhouse::fetch(client, slug).await,
            Self::Lever { slug } => lever::fetch(client, slug).await,
            Self::Ashby { slug } => ashby::fetch(client, slug).await,
        }
    }
}
