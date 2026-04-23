//! Cross-platform folder/file organizer.
//!
//! Layout:
//!   <root>/YYYY-MM-DD/<CompanyName>/<jobId_or_role>_<slug>/
//!       jd.txt
//!       jd_summary.txt
//!       metadata.json
//!       apply_link.url
//!       tailored_resume.{html,pdf}   (written by the resume module)
//!       cover_letter.txt              (optional)
//!
//! Path segments are sanitized before use, and every join is validated to
//! prevent traversal.

pub mod folder;
pub mod paths;
