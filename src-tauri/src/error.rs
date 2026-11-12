//! Centralised error type for the whole crate.
//!
//! `AppError` serialises to a plain string so it can be returned directly from
//! `#[tauri::command]` functions and displayed in the UI.

use serde::{Serialize, Serializer};

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("no project is currently open")]
    NoProject,

    #[error("path is outside the workspace root: {0}")]
    PathEscape(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("provider error: {0}")]
    Provider(String),

    #[error("authentication error: missing or invalid API key")]
    Auth,

    #[error("configuration error: {0}")]
    Config(String),

    #[error("command was cancelled")]
    Cancelled,

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Other(e.to_string())
    }
}

impl From<String> for AppError {
    fn from(e: String) -> Self {
        AppError::Other(e)
    }
}

impl From<&str> for AppError {
    fn from(e: &str) -> Self {
        AppError::Other(e.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(e: tauri::Error) -> Self {
        AppError::Other(e.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
