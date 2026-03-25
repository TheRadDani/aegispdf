//! Integration tests for core::compress (smart_compress, recompress_streams_roundtrip, zlib_compress_best).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod common;

use lopdf::Document;

use aegispdf_lib::core::compress;

fn make_pdf_file(dir: &std::path::PathBuf, name: &str) -> std::path::PathBuf {
    common::save_one_page_pdf(dir, name, name)
}

// ── smart_compress ────────────────────────────────────────────────────────────

#[test]
fn smart_compress_produces_loadable_single_page_document() {
    let dir = common::unique_dir("compress_smart");
    std::fs::create_dir_all(&dir).unwrap();
    let path = make_pdf_file(&dir, "smart");
    let mut doc = Document::load(&path).unwrap();

    compress::smart_compress(&mut doc);
    doc.save(&path).unwrap();

    let reloaded = Document::load(&path).unwrap();
    assert_eq!(reloaded.get_pages().len(), 1);
}

#[test]
fn smart_compress_on_two_page_doc_preserves_page_count() {
    let dir = common::unique_dir("compress_smart2");
    std::fs::create_dir_all(&dir).unwrap();
    let merged = common::merged_two_page_pdf(&dir);
    let mut doc = Document::load(&merged).unwrap();

    compress::smart_compress(&mut doc);
    doc.save(&merged).unwrap();

    let reloaded = Document::load(&merged).unwrap();
    assert_eq!(reloaded.get_pages().len(), 2);
}

// ── recompress_streams_roundtrip ──────────────────────────────────────────────

#[test]
fn recompress_streams_roundtrip_preserves_page_count() {
    let dir = common::unique_dir("compress_roundtrip");
    std::fs::create_dir_all(&dir).unwrap();
    let path = make_pdf_file(&dir, "roundtrip");
    let mut doc = Document::load(&path).unwrap();

    compress::recompress_streams_roundtrip(&mut doc);
    doc.save(&path).unwrap();

    let reloaded = Document::load(&path).unwrap();
    assert_eq!(reloaded.get_pages().len(), 1);
}

// ── zlib_compress_best ────────────────────────────────────────────────────────

#[test]
fn zlib_compress_best_produces_non_empty_output_for_non_empty_input() {
    let data = b"Some text to compress for AegisPDF tests.";
    let compressed = compress::zlib_compress_best(data).unwrap();
    assert!(!compressed.is_empty());
}

#[test]
fn zlib_compress_best_produces_empty_output_for_empty_input() {
    // flate2 produces a valid but minimal zlib stream for empty data.
    let compressed = compress::zlib_compress_best(b"").unwrap();
    // A valid empty zlib stream is still non-empty (header + checksum).
    assert!(!compressed.is_empty());
}

#[test]
fn zlib_compress_output_is_smaller_than_repetitive_input() {
    let data = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let compressed = compress::zlib_compress_best(data).unwrap();
    assert!(compressed.len() < data.len(), "compress should reduce highly repetitive data");
}
