//! Integration tests for `core::security` (metadata strip, annots strip, `auto_clean`).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod common;

use lopdf::{dictionary, Document, Object};

use aegispdf_lib::core::security;

// ── strip_metadata ────────────────────────────────────────────────────────────

#[test]
fn strip_metadata_removes_standard_info_fields() {
    let mut doc = Document::with_version("1.4");
    let info_id = doc.add_object(dictionary! {
        "Author"   => Object::string_literal("Alice"),
        "Creator"  => Object::string_literal("Word"),
        "Producer" => Object::string_literal("Acrobat"),
        "Subject"  => Object::string_literal("Sensitive"),
        "Title"    => Object::string_literal("Secret Doc"),
        "Keywords" => Object::string_literal("top secret"),
    });
    doc.trailer.set(b"Info", Object::Reference(info_id));

    security::strip_metadata(&mut doc);

    let info_ref = doc
        .trailer
        .get(b"Info")
        .and_then(Object::as_reference)
        .unwrap();
    let info = doc.get_object(info_ref).unwrap().as_dict().unwrap();
    assert!(info.get(b"Author").is_err(), "Author should be removed");
    assert!(info.get(b"Creator").is_err(), "Creator should be removed");
    assert!(info.get(b"Producer").is_err(), "Producer should be removed");
    assert!(info.get(b"Subject").is_err(), "Subject should be removed");
    assert!(info.get(b"Title").is_err(), "Title should be removed");
    assert!(info.get(b"Keywords").is_err(), "Keywords should be removed");
}

#[test]
fn strip_metadata_noop_when_no_info_dict() {
    // Document without a /Info entry in trailer — must not panic.
    let mut doc = common::one_page_doc("no-meta");
    security::strip_metadata(&mut doc); // should be a no-op
    assert_eq!(doc.get_pages().len(), 1);
}

// ── strip_page_annots ─────────────────────────────────────────────────────────

#[test]
fn strip_page_annots_removes_annots_entry_from_page() {
    let mut doc = common::one_page_doc("annotated");

    // Add an /Annots array to the first page.
    let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().into_values().collect();
    doc.get_dictionary_mut(page_ids[0])
        .unwrap()
        .set(b"Annots", Object::Array(vec![]));

    // Confirm the entry exists before stripping.
    let has_before = doc
        .get_object(page_ids[0])
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"Annots")
        .is_ok();
    assert!(has_before, "Annots should exist before strip");

    security::strip_page_annots(&mut doc);

    let has_after = doc
        .get_object(page_ids[0])
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"Annots")
        .is_ok();
    assert!(!has_after, "Annots should be removed after strip");
}

#[test]
fn strip_page_annots_noop_when_no_annots_present() {
    let mut doc = common::one_page_doc("clean");
    // Pages have no /Annots — must not panic.
    security::strip_page_annots(&mut doc);
    assert_eq!(doc.get_pages().len(), 1);
}

// ── auto_clean ────────────────────────────────────────────────────────────────

#[test]
fn auto_clean_without_strip_annots_preserves_page_count() {
    let mut doc = common::one_page_doc("autoclean");
    let info_id = doc.add_object(dictionary! {
        "Author" => Object::string_literal("Bob"),
    });
    doc.trailer.set(b"Info", Object::Reference(info_id));

    security::auto_clean(&mut doc, false);

    assert_eq!(doc.get_pages().len(), 1);
    // Author should be stripped even without strip_annots
    let info_ref = doc
        .trailer
        .get(b"Info")
        .and_then(Object::as_reference)
        .unwrap();
    let info = doc.get_object(info_ref).unwrap().as_dict().unwrap();
    assert!(info.get(b"Author").is_err());
}

#[test]
fn auto_clean_with_strip_annots_removes_annots_and_metadata() {
    let mut doc = common::one_page_doc("autoclean-full");

    // Add metadata
    let info_id = doc.add_object(dictionary! {
        "Creator" => Object::string_literal("Word"),
    });
    doc.trailer.set(b"Info", Object::Reference(info_id));

    // Add /Annots to first page
    let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().into_values().collect();
    doc.get_dictionary_mut(page_ids[0])
        .unwrap()
        .set(b"Annots", Object::Array(vec![]));

    security::auto_clean(&mut doc, true);

    // Metadata stripped
    let info_ref = doc
        .trailer
        .get(b"Info")
        .and_then(Object::as_reference)
        .unwrap();
    let info = doc.get_object(info_ref).unwrap().as_dict().unwrap();
    assert!(info.get(b"Creator").is_err());

    // Annots stripped
    let has_annots = doc
        .get_object(page_ids[0])
        .unwrap()
        .as_dict()
        .unwrap()
        .get(b"Annots")
        .is_ok();
    assert!(!has_annots);
}
