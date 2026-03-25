//! Structured errors for IPC and job completion payloads.

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AegisError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("pdf: {message}")]
    Pdf { code: &'static str, message: String },

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("document not found")]
    DocumentNotFound,

    #[error("workspace lock poisoned")]
    LockPoisoned,

    #[error("external tool failed: {tool} — {message}")]
    ExternalTool { tool: String, message: String },

    #[error("job failed: {0}")]
    Job(String),

    #[error("render: {0}")]
    Render(String),

    #[error("merge: {0}")]
    Merge(String),

    #[error("split: {0}")]
    Split(String),

    #[error("not supported: {0}")]
    NotSupported(String),
}

impl AegisError {
    pub fn pdf(code: &'static str, message: impl Into<String>) -> Self {
        Self::Pdf {
            code,
            message: message.into(),
        }
    }
}

/// Serializable shape returned to the frontend and emitted on job events.
#[derive(Debug, Clone, Serialize)]
pub struct AegisErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl From<AegisError> for AegisErrorResponse {
    fn from(e: AegisError) -> Self {
        let (code, message, details) = match &e {
            AegisError::Io(err) => ("io".to_string(), err.to_string(), None),
            AegisError::Pdf { code, message } => (format!("pdf::{code}"), message.clone(), None),
            AegisError::InvalidArgument(m) => ("invalid_argument".to_string(), m.clone(), None),
            AegisError::DocumentNotFound => ("document_not_found".to_string(), e.to_string(), None),
            AegisError::LockPoisoned => ("lock_poisoned".to_string(), e.to_string(), None),
            AegisError::ExternalTool { tool, message } => {
                (format!("external::{tool}"), message.clone(), None)
            }
            AegisError::Job(m) => ("job".to_string(), m.clone(), None),
            AegisError::Render(m) => ("render".to_string(), m.clone(), None),
            AegisError::Merge(m) => ("merge".to_string(), m.clone(), None),
            AegisError::Split(m) => ("split".to_string(), m.clone(), None),
            AegisError::NotSupported(m) => ("not_supported".to_string(), m.clone(), None),
        };
        Self {
            code,
            message,
            details,
        }
    }
}

pub type AegisResult<T> = Result<T, AegisError>;

#[must_use]
pub fn to_invoke_err(e: AegisError) -> String {
    serde_json::to_string(&AegisErrorResponse::from(e))
        .unwrap_or_else(|_| r#"{"code":"serialize","message":"error"}"#.to_string())
}
