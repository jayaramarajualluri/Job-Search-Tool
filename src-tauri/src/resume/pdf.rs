//! HTML → PDF via a headless Chromium-family subprocess.
//!
//! Best-effort and zero-runtime-dependency: we shell out to whichever
//! Chrome/Edge/Brave/Chromium binary is already installed on the user's
//! machine.  If none are found we fall back to HTML and surface the
//! error so the UI can tell the user what to install.
//!
//! Caveats:
//!   - The Tauri webview itself does NOT have a print-to-PDF API, so we
//!     rely on the system browser binary.
//!   - The HTML template emits `@page { margin: 0.2in 0.35in }` and
//!     handles its own header layout — we pass `--no-pdf-header-footer`
//!     to suppress Chrome's default URL/date stripe.

use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "macos")]
const CHROME_CANDIDATES: &[&str] = &[
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/Applications/Arc.app/Contents/MacOS/Arc",
];

#[cfg(target_os = "windows")]
const CHROME_CANDIDATES: &[&str] = &[
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe",
];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const CHROME_CANDIDATES: &[&str] = &[
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/usr/bin/microsoft-edge",
    "/usr/bin/brave-browser",
];

fn find_chrome() -> Option<PathBuf> {
    for p in CHROME_CANDIDATES {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Render `html_path` to `pdf_path` via a headless Chrome subprocess.
/// Returns an error if no Chrome-family browser is installed.
pub fn html_to_pdf(html_path: &Path, pdf_path: &Path) -> AppResult<()> {
    let chrome = find_chrome().ok_or_else(|| {
        AppError::invalid(
            "Couldn't find Chrome / Edge / Brave / Chromium. Install any one of them to enable PDF export.",
        )
    })?;

    let abs_html = html_path
        .canonicalize()
        .map_err(|e| AppError::Other(format!("canonicalize html path: {e}")))?;
    let file_url = url::Url::from_file_path(&abs_html)
        .map_err(|_| AppError::Other("could not build file:// URL".into()))?;

    tracing::debug!(chrome = %chrome.display(), html = %abs_html.display(), "headless print-to-pdf");

    let out = Command::new(&chrome)
        .args([
            "--headless=new",
            "--disable-gpu",
            "--no-pdf-header-footer",
            &format!("--print-to-pdf={}", pdf_path.display()),
            file_url.as_str(),
        ])
        .output();

    let out = match out {
        Ok(o) => o,
        Err(_) => {
            // Older Chrome doesn't accept --headless=new — retry with classic.
            Command::new(&chrome)
                .args([
                    "--headless",
                    "--disable-gpu",
                    "--no-pdf-header-footer",
                    &format!("--print-to-pdf={}", pdf_path.display()),
                    file_url.as_str(),
                ])
                .output()?
        }
    };

    if !out.status.success() {
        return Err(AppError::Other(format!(
            "headless chrome failed (exit {}): {}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    if !pdf_path.exists() {
        return Err(AppError::Other(
            "headless chrome reported success but no PDF was written".into(),
        ));
    }
    Ok(())
}
