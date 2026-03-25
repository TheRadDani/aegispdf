use std::path::{Path, PathBuf};

use lopdf::{Document, ObjectId};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::pages::{delete_pages_by_indices, reorder_pages_by_page_number};

#[derive(Debug)]
pub struct PdfWorkspace {
    pub source_path: PathBuf,
    pub document: Document,
    pub ordered_page_numbers: Vec<u32>,
    pub file_hash: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PageInfo {
    pub index: usize,
    pub page_number: u32,
}

impl PdfWorkspace {
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed as a PDF.
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let bytes = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let file_hash = format!("{:x}", hasher.finalize());
        let document = Document::load_mem(&bytes)?;
        let ordered_page_numbers = document.get_pages().keys().copied().collect::<Vec<_>>();
        Ok(Self {
            source_path: path.to_path_buf(),
            document,
            ordered_page_numbers,
            file_hash,
        })
    }

    #[must_use]
    pub fn page_count(&self) -> usize {
        self.ordered_page_numbers.len()
    }

    #[must_use]
    pub fn page_infos(&self) -> Vec<PageInfo> {
        self.ordered_page_numbers
            .iter()
            .enumerate()
            .map(|(index, page_number)| PageInfo {
                index,
                page_number: *page_number,
            })
            .collect()
    }

    #[must_use]
    pub fn object_id_for_index(&self, index: usize) -> Option<ObjectId> {
        let page_number = self.ordered_page_numbers.get(index)?;
        let pages = self.document.get_pages();
        pages.get(page_number).copied()
    }

    /// # Errors
    ///
    /// Returns an error if the page reordering fails.
    pub fn reorder_pages_by_number(&mut self, new_order: &[u32]) -> anyhow::Result<()> {
        reorder_pages_by_page_number(&mut self.document, new_order)?;
        self.ordered_page_numbers = new_order.to_vec();
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if the page deletion fails.
    pub fn delete_pages_by_indices(&mut self, indices: &[usize]) -> anyhow::Result<()> {
        delete_pages_by_indices(&mut self.document, indices, &self.ordered_page_numbers)?;
        self.ordered_page_numbers = self.document.get_pages().keys().copied().collect();
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if the document cannot be saved to disk.
    pub fn save_to(&mut self, output_path: &Path) -> anyhow::Result<()> {
        self.document.prune_objects();
        self.document.compress();
        self.document.save(output_path)?;
        Ok(())
    }

    pub fn apply_smart_compress(&mut self, roundtrip_streams: bool) {
        if roundtrip_streams {
            super::compress::recompress_streams_roundtrip(&mut self.document);
        }
        super::compress::smart_compress(&mut self.document);
    }

    pub fn apply_auto_clean(&mut self, strip_annots: bool) {
        super::security::auto_clean(&mut self.document, strip_annots);
    }
}
