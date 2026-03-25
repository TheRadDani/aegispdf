//! Extract contiguous page ranges into new PDF files (1-based inclusive page indices).

use std::path::Path;

use lopdf::Document;

use crate::error::{AegisError, AegisResult};

/// Each range is `(start_page, end_page)` inclusive, using PDF page labels from [`Document::get_pages`].
pub fn split_pdf_by_ranges(
    source: &Path,
    ranges: &[(u32, u32)],
    outputs: &[std::path::PathBuf],
) -> AegisResult<()> {
    if ranges.len() != outputs.len() {
        return Err(AegisError::InvalidArgument(
            "ranges and outputs length mismatch".into(),
        ));
    }
    let bytes = std::fs::read(source).map_err(AegisError::from)?;
    let all_pages: Vec<u32> = {
        let doc = Document::load_mem(&bytes).map_err(|e| AegisError::pdf("load", e.to_string()))?;
        doc.get_pages().keys().copied().collect()
    };

    for ((start, end), out) in ranges.iter().zip(outputs.iter()) {
        if start > end {
            return Err(AegisError::Split(format!("invalid range {start}-{end}")));
        }
        let mut doc =
            Document::load_mem(&bytes).map_err(|e| AegisError::pdf("load", e.to_string()))?;
        let to_delete: Vec<u32> = all_pages
            .iter()
            .copied()
            .filter(|p| *p < *start || *p > *end)
            .collect();
        if to_delete.len() == all_pages.len() {
            return Err(AegisError::Split("range excludes all pages".into()));
        }
        doc.delete_pages(&to_delete);
        doc.prune_objects();
        doc.compress();
        doc.save(out)
            .map_err(|e| AegisError::pdf("save", e.to_string()))?;
    }
    Ok(())
}
