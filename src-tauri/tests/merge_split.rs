//! Integration tests for merge/split pipeline (no PDFium required).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};

fn unique_dir(name: &str) -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("aegispdf_{name}_{n}"))
}

fn one_page_doc(label: &str) -> Document {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    });
    let content = Content {
        operations: vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 12.into()]),
            Operation::new("Td", vec![72.into(), 720.into()]),
            Operation::new("Tj", vec![Object::string_literal(label)]),
            Operation::new("ET", vec![]),
        ],
    };
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "Contents" => content_id,
        "Resources" => resources_id,
        "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
    });
    let pages = dictionary! {
        "Type" => "Pages",
        "Kids" => vec![page_id.into()],
        "Count" => 1,
    };
    doc.objects.insert(pages_id, Object::Dictionary(pages));
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.trailer.set(b"Root", Object::Reference(catalog_id));
    doc
}

#[test]
fn merge_two_one_page_pdfs() {
    let dir = unique_dir("merge");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.pdf");
    let b = dir.join("b.pdf");
    let out = dir.join("out.pdf");
    one_page_doc("A").save(&a).unwrap();
    one_page_doc("B").save(&b).unwrap();
    aegispdf_lib::core::merge::merge_pdfs(&[a, b], &out).unwrap();
    let merged = Document::load(&out).unwrap();
    assert_eq!(merged.get_pages().len(), 2);
}

#[test]
fn split_two_page_merged_pdf() {
    let dir = unique_dir("split");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.pdf");
    let b = dir.join("b.pdf");
    let merged_path = dir.join("m.pdf");
    let p1 = dir.join("p1.pdf");
    let p2 = dir.join("p2.pdf");
    one_page_doc("A").save(&a).unwrap();
    one_page_doc("B").save(&b).unwrap();
    aegispdf_lib::core::merge::merge_pdfs(&[a, b], &merged_path).unwrap();
    aegispdf_lib::core::split::split_pdf_by_ranges(
        &merged_path,
        &[(1, 1), (2, 2)],
        &[p1.clone(), p2.clone()],
    )
    .unwrap();
    assert_eq!(Document::load(&p1).unwrap().get_pages().len(), 1);
    assert_eq!(Document::load(&p2).unwrap().get_pages().len(), 1);
}

#[test]
fn reorder_pages_does_not_panic_on_merged_doc() {
    let dir = unique_dir("reorder");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.pdf");
    let b = dir.join("b.pdf");
    let merged_path = dir.join("m.pdf");
    one_page_doc("A").save(&a).unwrap();
    one_page_doc("B").save(&b).unwrap();
    aegispdf_lib::core::merge::merge_pdfs(&[a, b], &merged_path).unwrap();
    let mut doc = Document::load(&merged_path).unwrap();
    aegispdf_lib::core::pages::reorder_pages_by_page_number(&mut doc, &[2, 1]).unwrap();
}

#[test]
fn merge_with_empty_inputs_returns_error() {
    let dir = unique_dir("merge_empty");
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join("out.pdf");
    let result = aegispdf_lib::core::merge::merge_pdfs(&[], &out);
    assert!(result.is_err());
}

#[test]
fn split_range_output_count_mismatch_returns_error() {
    let dir = unique_dir("split_mismatch");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.pdf");
    one_page_doc("A").save(&a).unwrap();
    // 2 ranges but only 1 output path — must fail
    let result = aegispdf_lib::core::split::split_pdf_by_ranges(
        &a,
        &[(1, 1), (1, 1)],
        &[dir.join("out.pdf")],
    );
    assert!(result.is_err());
}

#[test]
fn split_invalid_range_start_gt_end_returns_error() {
    let dir = unique_dir("split_invalid");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.pdf");
    one_page_doc("A").save(&a).unwrap();
    let result = aegispdf_lib::core::split::split_pdf_by_ranges(
        &a,
        &[(3, 1)], // start > end
        &[dir.join("out.pdf")],
    );
    assert!(result.is_err());
}

#[test]
fn split_range_excludes_all_pages_returns_error() {
    let dir = unique_dir("split_nopage");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.pdf");
    one_page_doc("A").save(&a).unwrap();
    // Page range (5, 10) does not include the single page (page 1)
    let result =
        aegispdf_lib::core::split::split_pdf_by_ranges(&a, &[(5, 10)], &[dir.join("out.pdf")]);
    assert!(result.is_err());
}

#[test]
fn delete_pages_by_indices_removes_correct_page() {
    let dir = unique_dir("delete_pages");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.pdf");
    let b = dir.join("b.pdf");
    let merged_path = dir.join("m.pdf");
    one_page_doc("A").save(&a).unwrap();
    one_page_doc("B").save(&b).unwrap();
    aegispdf_lib::core::merge::merge_pdfs(&[a, b], &merged_path).unwrap();
    let mut doc = Document::load(&merged_path).unwrap();
    // ordered_page_numbers mirrors get_pages() order
    let page_nums: Vec<u32> = doc.get_pages().keys().copied().collect();
    aegispdf_lib::core::pages::delete_pages_by_indices(&mut doc, &[0], &page_nums).unwrap();
    assert_eq!(doc.get_pages().len(), 1);
}

#[test]
fn delete_pages_invalid_index_returns_error() {
    let dir = unique_dir("delete_invalid");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.pdf");
    one_page_doc("A").save(&a).unwrap();
    let mut doc = Document::load(&a).unwrap();
    let page_nums: Vec<u32> = doc.get_pages().keys().copied().collect();
    // index 99 does not exist
    let result = aegispdf_lib::core::pages::delete_pages_by_indices(&mut doc, &[99], &page_nums);
    assert!(result.is_err());
}

#[test]
fn reorder_pages_missing_page_number_returns_error() {
    let dir = unique_dir("reorder_missing");
    std::fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.pdf");
    one_page_doc("A").save(&a).unwrap();
    let mut doc = Document::load(&a).unwrap();
    // page number 99 does not exist
    let result = aegispdf_lib::core::pages::reorder_pages_by_page_number(&mut doc, &[99]);
    assert!(result.is_err());
}
