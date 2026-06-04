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

/// Convert an HTML file to PDF using Chrome headless.
/// Returns `Ok(pdf_path)` on success, `Err` if Chrome is not found or fails.
pub fn html_to_pdf(html_path: &Path) -> AppResult<std::path::PathBuf> {
    let chrome = find_chrome().ok_or_else(|| {
        crate::error::AppError::invalid(
            "Google Chrome not found — install Chrome to generate PDFs",
        )
    })?;
    let pdf_path = html_path.with_extension("pdf");
    let file_url = format!("file://{}", html_path.display());
    let pdf_arg = format!("--print-to-pdf={}", pdf_path.display());
    let status = std::process::Command::new(&chrome)
        .args([
            "--headless",
            "--disable-gpu",
            "--no-sandbox",
            "--print-to-pdf-no-header",
            &pdf_arg,
            &file_url,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    if status.success() && pdf_path.exists() {
        Ok(pdf_path)
    } else {
        Err(crate::error::AppError::invalid("Chrome headless PDF generation failed"))
    }
}

fn find_chrome() -> Option<std::path::PathBuf> {
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium-browser",
        "/usr/bin/chromium",
    ];
    candidates
        .iter()
        .map(std::path::Path::new)
        .find(|p| p.exists())
        .map(|p| p.to_path_buf())
}
