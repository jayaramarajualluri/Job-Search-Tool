//! Resume canonical profile + tailoring + rendering + reuse detection.
//!
//! Honest tailoring: we only select/reorder/re-emphasize existing truthful
//! content.  We never add skills, bullets, or metrics the profile doesn't
//! already declare.

pub mod renderer;
pub mod reuse;
pub mod tailor;

pub use tailor::{TailoredResume, TailoringRequest};
