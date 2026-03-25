//! Integration tests for error types: AegisError, AegisErrorResponse, to_invoke_err.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use aegispdf_lib::error::{AegisError, AegisErrorResponse, to_invoke_err};

// ── AegisError constructors ───────────────────────────────────────────────────

#[test]
fn aegis_error_pdf_helper_sets_code_and_message() {
    let e = AegisError::pdf("load", "could not open file");
    let msg = e.to_string();
    assert!(msg.contains("could not open file"), "message should appear in Display: {msg}");
}

#[test]
fn aegis_error_invalid_argument_displays_message() {
    let e = AegisError::InvalidArgument("bad range".into());
    assert!(e.to_string().contains("bad range"));
}

#[test]
fn aegis_error_document_not_found_displays() {
    let e = AegisError::DocumentNotFound;
    assert!(!e.to_string().is_empty());
}

#[test]
fn aegis_error_lock_poisoned_displays() {
    let e = AegisError::LockPoisoned;
    assert!(!e.to_string().is_empty());
}

#[test]
fn aegis_error_merge_displays_message() {
    let e = AegisError::Merge("overlap detected".into());
    assert!(e.to_string().contains("overlap detected"));
}

#[test]
fn aegis_error_split_displays_message() {
    let e = AegisError::Split("empty range".into());
    assert!(e.to_string().contains("empty range"));
}

#[test]
fn aegis_error_render_displays_message() {
    let e = AegisError::Render("pdfium not found".into());
    assert!(e.to_string().contains("pdfium not found"));
}

#[test]
fn aegis_error_job_displays_message() {
    let e = AegisError::Job("timeout".into());
    assert!(e.to_string().contains("timeout"));
}

#[test]
fn aegis_error_not_supported_displays_message() {
    let e = AegisError::NotSupported("feature X".into());
    assert!(e.to_string().contains("feature X"));
}

#[test]
fn aegis_error_external_tool_displays_tool_and_message() {
    let e = AegisError::ExternalTool {
        tool: "tesseract".into(),
        message: "not found on PATH".into(),
    };
    assert!(e.to_string().contains("tesseract"));
    assert!(e.to_string().contains("not found on PATH"));
}

#[test]
fn aegis_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let e: AegisError = io_err.into();
    assert!(e.to_string().contains("file missing"));
}

// ── AegisErrorResponse From<AegisError> ──────────────────────────────────────

#[test]
fn error_response_from_invalid_argument_sets_correct_code() {
    let e = AegisError::InvalidArgument("nope".into());
    let resp = AegisErrorResponse::from(e);
    assert_eq!(resp.code, "invalid_argument");
    assert_eq!(resp.message, "nope");
    assert!(resp.details.is_none());
}

#[test]
fn error_response_from_pdf_error_sets_code_with_prefix() {
    let e = AegisError::pdf("save", "disk full");
    let resp = AegisErrorResponse::from(e);
    assert_eq!(resp.code, "pdf::save");
    assert_eq!(resp.message, "disk full");
}

#[test]
fn error_response_from_document_not_found() {
    let resp = AegisErrorResponse::from(AegisError::DocumentNotFound);
    assert_eq!(resp.code, "document_not_found");
    assert!(!resp.message.is_empty());
}

#[test]
fn error_response_from_lock_poisoned() {
    let resp = AegisErrorResponse::from(AegisError::LockPoisoned);
    assert_eq!(resp.code, "lock_poisoned");
}

#[test]
fn error_response_from_merge_error() {
    let resp = AegisErrorResponse::from(AegisError::Merge("bad merge".into()));
    assert_eq!(resp.code, "merge");
    assert_eq!(resp.message, "bad merge");
}

#[test]
fn error_response_from_split_error() {
    let resp = AegisErrorResponse::from(AegisError::Split("bad split".into()));
    assert_eq!(resp.code, "split");
    assert_eq!(resp.message, "bad split");
}

#[test]
fn error_response_from_render_error() {
    let resp = AegisErrorResponse::from(AegisError::Render("render fail".into()));
    assert_eq!(resp.code, "render");
}

#[test]
fn error_response_from_job_error() {
    let resp = AegisErrorResponse::from(AegisError::Job("timed out".into()));
    assert_eq!(resp.code, "job");
    assert_eq!(resp.message, "timed out");
}

#[test]
fn error_response_from_not_supported() {
    let resp = AegisErrorResponse::from(AegisError::NotSupported("fancy feature".into()));
    assert_eq!(resp.code, "not_supported");
}

#[test]
fn error_response_from_external_tool() {
    let resp = AegisErrorResponse::from(AegisError::ExternalTool {
        tool: "gs".into(),
        message: "exit 1".into(),
    });
    assert_eq!(resp.code, "external::gs");
    assert_eq!(resp.message, "exit 1");
}

// ── to_invoke_err ─────────────────────────────────────────────────────────────

#[test]
fn to_invoke_err_returns_valid_json_string() {
    let e = AegisError::InvalidArgument("bad arg".into());
    let s = to_invoke_err(e);
    let parsed: serde_json::Value = serde_json::from_str(&s)
        .expect("to_invoke_err must return valid JSON");
    assert_eq!(parsed["code"], "invalid_argument");
    assert_eq!(parsed["message"], "bad arg");
}

#[test]
fn to_invoke_err_contains_code_field() {
    let s = to_invoke_err(AegisError::Merge("fail".into()));
    assert!(s.contains("\"code\""));
    assert!(s.contains("\"merge\""));
}
