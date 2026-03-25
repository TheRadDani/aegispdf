use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AegisError, AegisResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationType {
    Highlight,
    TextNote,
    Drawing,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Annotation {
    pub id: String,
    pub page_index: usize,
    pub annotation_type: AnnotationType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnotationStore {
    pub pdf_hash: String,
    pub annotations: Vec<Annotation>,
}

impl AnnotationStore {
    /// Sidecar path: `document.pdf` → `document.aegis`
    #[must_use]
    pub fn sidecar_path(pdf_path: &Path) -> PathBuf {
        let mut p = pdf_path.to_path_buf();
        p.set_extension("aegis");
        p
    }

    /// # Errors
    ///
    /// Returns [`AegisError`] if the sidecar file cannot be read, parsed as JSON,
    /// or if the PDF hash does not match.
    pub fn load_for_pdf(pdf_path: &Path, expected_hash: &str) -> AegisResult<Self> {
        let path = Self::sidecar_path(pdf_path);
        if !path.exists() {
            return Ok(Self {
                pdf_hash: expected_hash.to_string(),
                annotations: vec![],
            });
        }
        let text = fs::read_to_string(&path).map_err(AegisError::from)?;
        let store: Self =
            serde_json::from_str(&text).map_err(|e| AegisError::InvalidArgument(e.to_string()))?;
        if store.pdf_hash != expected_hash {
            return Err(AegisError::InvalidArgument(
                "annotation file hash does not match PDF (file changed)".into(),
            ));
        }
        Ok(store)
    }

    /// # Errors
    ///
    /// Returns [`AegisError`] if the annotation store cannot be serialized or written to disk.
    pub fn save(&self, pdf_path: &Path) -> AegisResult<()> {
        let path = Self::sidecar_path(pdf_path);
        let text = serde_json::to_string_pretty(self)
            .map_err(|e| AegisError::InvalidArgument(e.to_string()))?;
        fs::write(&path, text).map_err(AegisError::from)?;
        Ok(())
    }
}
