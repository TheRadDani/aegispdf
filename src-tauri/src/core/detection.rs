//! Blank and duplicate page detection using downscaled render fingerprints.

use std::collections::HashMap;

use lopdf::Document;

use crate::error::{AegisError, AegisResult};
use crate::render::pdfium_renderer;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PageAnalysis {
    pub page_index: usize,
    pub is_blank: bool,
    pub content_hash: String,
    pub duplicate_of: Option<usize>,
}

/// `threshold` — mean absolute deviation from white (0–255 scale) below this ⇒ blank.
///
/// # Errors
///
/// Returns [`AegisError`] if any page cannot be rendered.
pub fn analyze_pages(document: &Document, threshold: f32) -> AegisResult<Vec<PageAnalysis>> {
    let page_count = document.get_pages().len();
    let mut results = Vec::with_capacity(page_count);
    let mut hash_to_first: HashMap<String, usize> = HashMap::new();

    for idx in 0..page_count {
        let (hash, mad) = pdfium_renderer::page_render_fingerprint(document, idx, 64)
            .map_err(|e| AegisError::Render(e.to_string()))?;
        let is_blank = mad < threshold;
        let duplicate_of = hash_to_first
            .get(&hash)
            .copied()
            .filter(|&first| first != idx);
        if !hash_to_first.contains_key(&hash) {
            hash_to_first.insert(hash.clone(), idx);
        }
        results.push(PageAnalysis {
            page_index: idx,
            is_blank,
            content_hash: hash,
            duplicate_of,
        });
    }
    Ok(results)
}
