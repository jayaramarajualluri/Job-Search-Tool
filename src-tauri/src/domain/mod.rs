//! Business entities.  These are the types that cross the IPC boundary and
//! are persisted to SQLite.  All string enums use snake_case on the wire
//! and map to the CHECK vocabularies declared in 001_init.sql.

pub mod account;
pub mod company;
pub mod ingestion_run;
pub mod job;
pub mod resume;
pub mod settings;
pub mod status_history;

pub use account::{Account, PortalType};
pub use company::Company;
pub use ingestion_run::{IngestionRun, SourceSummary};
pub use job::{Job, JobStatus, RecencyBucket, SponsorshipConfidence, WorkMode};
pub use resume::{Resume, ReuseStrategy};
pub use settings::SettingEntry;
pub use status_history::StatusHistoryEntry;
