//! Integration tests for core::annotations (AnnotationStore sidecar lifecycle).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod common;

use std::path::PathBuf;

use aegispdf_lib::core::annotations::{Annotation, AnnotationStore, AnnotationType};

fn unique_dir(name: &str) -> PathBuf {
    common::unique_dir(name)
}

// ── sidecar_path ─────────────────────────────────────────────────────────────

#[test]
fn sidecar_path_replaces_pdf_extension_with_aegis() {
    let pdf = PathBuf::from("/some/path/document.pdf");
    let sidecar = AnnotationStore::sidecar_path(&pdf);
    assert_eq!(sidecar.extension().unwrap(), "aegis");
    assert_eq!(sidecar.file_stem().unwrap(), "document");
}

#[test]
fn sidecar_path_works_for_path_without_extension() {
    let path = PathBuf::from("/tmp/myfile");
    let sidecar = AnnotationStore::sidecar_path(&path);
    assert_eq!(sidecar.extension().unwrap(), "aegis");
}

// ── load_for_pdf ─────────────────────────────────────────────────────────────

#[test]
fn load_for_pdf_returns_empty_store_when_no_sidecar_exists() {
    let dir = unique_dir("annot_nosidecar");
    std::fs::create_dir_all(&dir).unwrap();
    let pdf = dir.join("test.pdf");
    std::fs::write(&pdf, b"%PDF-1.4").unwrap();

    let store = AnnotationStore::load_for_pdf(&pdf, "hashvalue").unwrap();
    assert!(store.annotations.is_empty(), "expected empty annotations");
    assert_eq!(store.pdf_hash, "hashvalue");
}

#[test]
fn load_for_pdf_returns_error_on_hash_mismatch() {
    let dir = unique_dir("annot_hashmismatch");
    std::fs::create_dir_all(&dir).unwrap();
    let pdf = dir.join("test.pdf");
    std::fs::write(&pdf, b"%PDF-1.4").unwrap();

    let original = AnnotationStore {
        pdf_hash: "original-hash-abc".into(),
        annotations: vec![],
    };
    original.save(&pdf).unwrap();

    let result = AnnotationStore::load_for_pdf(&pdf, "different-hash-xyz");
    assert!(result.is_err(), "expected hash mismatch error");
}

// ── save + reload roundtrip ───────────────────────────────────────────────────

#[test]
fn save_and_reload_annotations_roundtrip() {
    let dir = unique_dir("annot_roundtrip");
    std::fs::create_dir_all(&dir).unwrap();
    let pdf = dir.join("doc.pdf");
    std::fs::write(&pdf, b"%PDF-1.4").unwrap();

    let store = AnnotationStore {
        pdf_hash: "abc123def456".into(),
        annotations: vec![
            Annotation {
                id: "ann-1".into(),
                page_index: 0,
                annotation_type: AnnotationType::Highlight,
                payload: serde_json::json!({"x": 0.05, "y": 0.1, "w": 0.9, "h": 0.2}),
            },
            Annotation {
                id: "ann-2".into(),
                page_index: 1,
                annotation_type: AnnotationType::TextNote,
                payload: serde_json::json!({"text": "hello world"}),
            },
        ],
    };
    store.save(&pdf).unwrap();

    let loaded = AnnotationStore::load_for_pdf(&pdf, "abc123def456").unwrap();
    assert_eq!(loaded.annotations.len(), 2);
    assert_eq!(loaded.annotations[0].id, "ann-1");
    assert_eq!(loaded.annotations[1].id, "ann-2");
    assert_eq!(loaded.pdf_hash, "abc123def456");
}

#[test]
fn save_empty_annotations_and_reload() {
    let dir = unique_dir("annot_empty");
    std::fs::create_dir_all(&dir).unwrap();
    let pdf = dir.join("empty.pdf");
    std::fs::write(&pdf, b"%PDF-1.4").unwrap();

    let store = AnnotationStore {
        pdf_hash: "emptyhash".into(),
        annotations: vec![],
    };
    store.save(&pdf).unwrap();

    let loaded = AnnotationStore::load_for_pdf(&pdf, "emptyhash").unwrap();
    assert!(loaded.annotations.is_empty());
}

#[test]
fn drawing_annotation_type_roundtrips() {
    let dir = unique_dir("annot_drawing");
    std::fs::create_dir_all(&dir).unwrap();
    let pdf = dir.join("draw.pdf");
    std::fs::write(&pdf, b"%PDF-1.4").unwrap();

    let store = AnnotationStore {
        pdf_hash: "drawhash".into(),
        annotations: vec![Annotation {
            id: "draw-1".into(),
            page_index: 2,
            annotation_type: AnnotationType::Drawing,
            payload: serde_json::json!({"points": [[0,0],[1,1]]}),
        }],
    };
    store.save(&pdf).unwrap();

    let loaded = AnnotationStore::load_for_pdf(&pdf, "drawhash").unwrap();
    assert_eq!(loaded.annotations.len(), 1);
    assert_eq!(loaded.annotations[0].page_index, 2);
}
