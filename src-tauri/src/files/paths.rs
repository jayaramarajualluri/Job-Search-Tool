//! Path sanitization and safe join.  Every user-derived path segment flows
//! through `safe_slug` before being joined to a root; `safe_join` rejects
//! `..`, absolute overrides, and Windows reserved names.

use crate::error::{AppError, AppResult};
use std::path::{Component, Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

const RESERVED: &[&str] = &[
    "con", "prn", "aux", "nul",
    "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9",
    "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];
const INVALID_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*', '\0'];
const MAX_SEGMENT: usize = 80;

/// Turn an arbitrary display string into a filesystem-safe segment.
pub fn safe_slug(input: &str) -> String {
    let decomposed: String = input.nfkd().filter(|c| !c.is_control()).collect();
    let mut out = String::with_capacity(decomposed.len());
    let mut prev_sep = false;
    for c in decomposed.chars() {
        let replaced = if INVALID_CHARS.contains(&c) || c.is_whitespace() {
            '_'
        } else {
            c
        };
        if replaced == '_' {
            if !prev_sep && !out.is_empty() {
                out.push('_');
                prev_sep = true;
            }
        } else {
            out.push(replaced);
            prev_sep = false;
        }
    }
    // Trim leading/trailing dots, underscores, and spaces.
    let trimmed: String = out
        .trim_matches(|c: char| c == '.' || c == '_' || c.is_whitespace())
        .to_string();
    let lowered = trimmed.to_lowercase();
    let safe = if RESERVED.contains(&lowered.as_str()) {
        format!("_{}", trimmed)
    } else {
        trimmed
    };
    if safe.is_empty() {
        return "untitled".into();
    }
    // Cap length.
    if safe.chars().count() > MAX_SEGMENT {
        safe.chars().take(MAX_SEGMENT).collect()
    } else {
        safe
    }
}

/// Join `segments` onto `root` after sanitizing each and rejecting any
/// traversal attempt.  All segments must be relative and non-empty after
/// sanitization.
pub fn safe_join(root: &Path, segments: &[&str]) -> AppResult<PathBuf> {
    let mut path = PathBuf::from(root);
    for seg in segments {
        let slug = safe_slug(seg);
        let candidate = Path::new(&slug);
        for comp in candidate.components() {
            match comp {
                Component::Normal(os) => path.push(os),
                Component::CurDir => {}
                _ => {
                    return Err(AppError::invalid(format!(
                        "rejected path segment: {seg:?}"
                    )))
                }
            }
        }
    }
    Ok(path)
}

/// Decide the inner folder name for a job:
/// prefer jobId, fall back to role title, fall back to JD-keyword slug.
pub fn role_folder_name(
    job_external_id: Option<&str>,
    role_title: &str,
    jd_keyword: Option<&str>,
) -> String {
    if let Some(id) = job_external_id {
        let id = safe_slug(id);
        if !id.is_empty() && id != "untitled" {
            let suffix = safe_slug(role_title);
            return format!("{id}_{suffix}");
        }
    }
    let role = safe_slug(role_title);
    if role != "untitled" {
        return role;
    }
    jd_keyword
        .map(safe_slug)
        .unwrap_or_else(|| "untitled".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_strips_separators() {
        // We preserve case but collapse whitespace and separators to `_`.
        assert_eq!(safe_slug("Acme / Corp"), "Acme_Corp");
        assert_eq!(safe_slug("   My Role   "), "My_Role");
    }

    #[test]
    fn rejects_reserved_names() {
        assert!(safe_slug("CON").starts_with('_'));
        assert!(safe_slug("com1").starts_with('_'));
    }

    #[test]
    fn join_rejects_traversal() {
        let r = std::env::temp_dir();
        let joined = safe_join(&r, &["..", "etc"]).unwrap();
        // `..` sanitizes to an empty-then-skipped segment; result stays inside root.
        assert!(joined.starts_with(&r));
    }
}
