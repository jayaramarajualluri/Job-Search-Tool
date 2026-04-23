//! Render a `TailoredResume` to HTML using Handlebars.  The frontend is
//! responsible for converting the HTML to PDF via the Tauri webview's
//! print-to-PDF capability (kept at the UI layer because PDF rendering
//! requires an on-screen window).

use super::tailor::TailoredResume;
use crate::error::AppResult;
use handlebars::Handlebars;
use std::fs;
use std::path::Path;

const TEMPLATE: &str = include_str!("../../../resources/resume_template.html");

pub fn render_html(resume: &TailoredResume) -> AppResult<String> {
    let mut hb = Handlebars::new();
    hb.set_strict_mode(false);
    hb.register_escape_fn(handlebars::html_escape);
    hb.register_template_string("resume", TEMPLATE)
        .map_err(|e| crate::error::AppError::Other(e.to_string()))?;
    Ok(hb.render("resume", resume)?)
}

pub fn render_html_to(resume: &TailoredResume, path: &Path) -> AppResult<()> {
    let html = render_html(resume)?;
    fs::write(path, html)?;
    Ok(())
}
