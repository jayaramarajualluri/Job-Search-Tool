//! Per-entity repositories.  Stage 6 fills in the concrete query bodies;
//! Stage 2 only needs the module layout to exist so the domain types can
//! compile cleanly alongside a place to grow.

pub mod companies;
pub mod jobs;
pub mod accounts;
pub mod resumes;
pub mod settings;
pub mod ingestion_runs;
