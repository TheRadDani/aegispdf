//! Integration tests for core::pdf::PdfWorkspace.

mod common;

use std::path::PathBuf;

use aegispdf_lib::core::pdf::PdfWorkspace;

fn setup(name: &str) -> PathBuf {
    let dir = common::unique_dir(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ── open / basic accessors ────────────────────────────────────────────────────

#[test]
fn workspace_open_single_page_pdf() {
    let dir = setup("ws_open");
    let path = common::save_one_page_pdf(&dir, "doc", "Hello");
    let ws = PdfWorkspace::open(&path).unwrap();
    assert_eq!(ws.page_count(), 1);
}

#[test]
fn workspace_file_hash_is_64_hex_chars() {
    let dir = setup("ws_hash");
    let path = common::save_one_page_pdf(&dir, "doc", "Hash");
    let ws = PdfWorkspace::open(&path).unwrap();
    assert_eq!(ws.file_hash.len(), 64, "SHA-256 hex string must be 64 chars");
    assert!(
        ws.file_hash.chars().all(|c| c.is_ascii_hexdigit()),
        "file_hash must be hex"
    );
}

#[test]
fn workspace_source_path_matches_opened_file() {
    let dir = setup("ws_path");
    let path = common::save_one_page_pdf(&dir, "doc", "Path");
    let ws = PdfWorkspace::open(&path).unwrap();
    assert_eq!(ws.source_path, path);
}

#[test]
fn workspace_open_missing_file_returns_error() {
    let result = PdfWorkspace::open(std::path::Path::new("/nonexistent/absolutely/fake.pdf"));
    assert!(result.is_err());
}

// ── page_infos ────────────────────────────────────────────────────────────────

#[test]
fn page_infos_single_page_has_correct_index() {
    let dir = setup("ws_infos1");
    let path = common::save_one_page_pdf(&dir, "doc", "Infos");
    let ws = PdfWorkspace::open(&path).unwrap();
    let infos = ws.page_infos();
    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].index, 0);
}

#[test]
fn page_infos_two_page_doc_has_sequential_indices() {
    let dir = setup("ws_infos2");
    let merged = common::merged_two_page_pdf(&dir);
    let ws = PdfWorkspace::open(&merged).unwrap();
    let infos = ws.page_infos();
    assert_eq!(infos.len(), 2);
    assert_eq!(infos[0].index, 0);
    assert_eq!(infos[1].index, 1);
}

// ── object_id_for_index ───────────────────────────────────────────────────────

#[test]
fn object_id_for_valid_index_returns_some() {
    let dir = setup("ws_objid_valid");
    let path = common::save_one_page_pdf(&dir, "doc", "ObjId");
    let ws = PdfWorkspace::open(&path).unwrap();
    assert!(ws.object_id_for_index(0).is_some());
}

#[test]
fn object_id_for_out_of_range_index_returns_none() {
    let dir = setup("ws_objid_oob");
    let path = common::save_one_page_pdf(&dir, "doc", "ObjIdOob");
    let ws = PdfWorkspace::open(&path).unwrap();
    assert!(ws.object_id_for_index(99).is_none());
}

// ── reorder_pages_by_number ───────────────────────────────────────────────────

#[test]
fn reorder_pages_by_number_reverses_two_page_doc() {
    let dir = setup("ws_reorder");
    let merged = common::merged_two_page_pdf(&dir);
    let mut ws = PdfWorkspace::open(&merged).unwrap();
    assert_eq!(ws.page_count(), 2);
    let original = ws.ordered_page_numbers.clone();
    // Reverse the order
    let reversed: Vec<u32> = original.iter().copied().rev().collect();
    ws.reorder_pages_by_number(&reversed).unwrap();
    assert_eq!(ws.ordered_page_numbers, reversed);
}

#[test]
fn reorder_pages_by_number_with_invalid_page_number_returns_error() {
    let dir = setup("ws_reorder_err");
    let path = common::save_one_page_pdf(&dir, "doc", "Reorder");
    let mut ws = PdfWorkspace::open(&path).unwrap();
    let result = ws.reorder_pages_by_number(&[999]);
    assert!(result.is_err());
}

// ── delete_pages_by_indices ───────────────────────────────────────────────────

#[test]
fn delete_pages_by_indices_reduces_page_count() {
    let dir = setup("ws_delete");
    let merged = common::merged_two_page_pdf(&dir);
    let mut ws = PdfWorkspace::open(&merged).unwrap();
    assert_eq!(ws.page_count(), 2);
    ws.delete_pages_by_indices(&[0]).unwrap();
    assert_eq!(ws.page_count(), 1);
}

#[test]
fn delete_pages_by_indices_invalid_index_returns_error() {
    let dir = setup("ws_delete_err");
    let path = common::save_one_page_pdf(&dir, "doc", "DelErr");
    let mut ws = PdfWorkspace::open(&path).unwrap();
    let result = ws.delete_pages_by_indices(&[99]);
    assert!(result.is_err());
}

// ── save_to ───────────────────────────────────────────────────────────────────

#[test]
fn save_to_writes_loadable_pdf() {
    let dir = setup("ws_save");
    let src = common::save_one_page_pdf(&dir, "src", "SaveSrc");
    let mut ws = PdfWorkspace::open(&src).unwrap();
    let out = dir.join("out.pdf");
    ws.save_to(&out).unwrap();
    // File must exist and be loadable
    assert!(out.exists());
    let reloaded = lopdf::Document::load(&out).unwrap();
    assert_eq!(reloaded.get_pages().len(), 1);
}

// ── apply_smart_compress ──────────────────────────────────────────────────────

#[test]
fn apply_smart_compress_with_roundtrip_does_not_corrupt() {
    let dir = setup("ws_compress_rt");
    let merged = common::merged_two_page_pdf(&dir);
    let mut ws = PdfWorkspace::open(&merged).unwrap();
    ws.apply_smart_compress(true); // roundtrip = true
    let out = dir.join("compressed.pdf");
    ws.save_to(&out).unwrap();
    let reloaded = lopdf::Document::load(&out).unwrap();
    assert_eq!(reloaded.get_pages().len(), 2);
}

#[test]
fn apply_smart_compress_without_roundtrip_does_not_corrupt() {
    let dir = setup("ws_compress_nort");
    let path = common::save_one_page_pdf(&dir, "doc", "Compress");
    let mut ws = PdfWorkspace::open(&path).unwrap();
    ws.apply_smart_compress(false); // roundtrip = false
    let out = dir.join("compressed.pdf");
    ws.save_to(&out).unwrap();
    let reloaded = lopdf::Document::load(&out).unwrap();
    assert_eq!(reloaded.get_pages().len(), 1);
}

// ── apply_auto_clean ──────────────────────────────────────────────────────────

#[test]
fn apply_auto_clean_without_strip_annots_does_not_corrupt() {
    let dir = setup("ws_clean_no_strip");
    let path = common::save_one_page_pdf(&dir, "doc", "Clean");
    let mut ws = PdfWorkspace::open(&path).unwrap();
    ws.apply_auto_clean(false);
    let out = dir.join("cleaned.pdf");
    ws.save_to(&out).unwrap();
    let reloaded = lopdf::Document::load(&out).unwrap();
    assert_eq!(reloaded.get_pages().len(), 1);
}

#[test]
fn apply_auto_clean_with_strip_annots_does_not_corrupt() {
    let dir = setup("ws_clean_strip");
    let path = common::save_one_page_pdf(&dir, "doc", "CleanStrip");
    let mut ws = PdfWorkspace::open(&path).unwrap();
    ws.apply_auto_clean(true);
    let out = dir.join("cleaned_full.pdf");
    ws.save_to(&out).unwrap();
    let reloaded = lopdf::Document::load(&out).unwrap();
    assert_eq!(reloaded.get_pages().len(), 1);
}
