//! OCR via Tesseract CLI (offline). Requires `tesseract` on PATH.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use tempfile::NamedTempFile;

use crate::error::{AegisError, AegisResult};

/// Run Tesseract on a PNG byte slice; returns UTF-8 text.
pub fn ocr_png_bytes(png: &[u8], lang: &str) -> AegisResult<String> {
    let mut tmp = NamedTempFile::with_suffix(".png").map_err(AegisError::from)?;
    tmp.write_all(png).map_err(AegisError::from)?;
    tmp.flush().map_err(AegisError::from)?;

    let out = Command::new("tesseract")
        .arg(tmp.path())
        .arg("stdout")
        .arg("-l")
        .arg(lang)
        .output()
        .map_err(|e| AegisError::ExternalTool {
            tool: "tesseract".into(),
            message: e.to_string(),
        })?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(AegisError::ExternalTool {
            tool: "tesseract".into(),
            message: format!("exit {:?}: {stderr}", out.status.code()),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Append per-page OCR text to a UTF-8 file (simple text layer export).
pub fn append_page_text(path: &Path, page_index: usize, text: &str) -> AegisResult<()> {
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(AegisError::from)?;
    writeln!(f, "--- Page {} ---\n{text}\n", page_index.saturating_add(1))
        .map_err(AegisError::from)?;
    Ok(())
}
