//! Unified error type for the backend. All IPC commands return `AppResult<T>`
//! which serializes to a string error on the frontend. We intentionally keep
//! the serialized form opaque to avoid leaking implementation details, but
//! keep rich context in logs.

use serde::{Serialize, Serializer};
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("template error: {0}")]
    Template(#[from] handlebars::RenderError),

    #[error("tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("invalid input: {0}")]
    Invalid(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("other: {0}")]
    Other(String),
}

impl AppError {
    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::Invalid(msg.into())
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
}

// Serialize as a plain string so the frontend can surface messages without
// depending on backend enum shape.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
