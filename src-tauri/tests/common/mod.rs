//! Shared test helpers for all integration test crates.
#![allow(dead_code)]

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Document, Object, Stream};

/// Return a unique temp directory path (not yet created).
pub fn unique_dir(name: &str) -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("aegispdf_{name}_{n}"))
}

/// Build a minimal one-page lopdf Document with the given text label.
pub fn one_page_doc(label: &str) -> Document {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! { "F1" => font_id },
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
    let content_id =
        doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
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

/// Save a one-page PDF to `path` and return the path.
pub fn save_one_page_pdf(dir: &PathBuf, name: &str, label: &str) -> PathBuf {
    let path = dir.join(format!("{name}.pdf"));
    one_page_doc(label).save(&path).unwrap();
    path
}

/// Create a two-page merged PDF in `dir` and return the path to it.
pub fn merged_two_page_pdf(dir: &PathBuf) -> PathBuf {
    let a = save_one_page_pdf(dir, "a", "Page A");
    let b = save_one_page_pdf(dir, "b", "Page B");
    let merged = dir.join("merged.pdf");
    aegispdf_lib::core::merge::merge_pdfs(&[a, b], &merged).unwrap();
    merged
}
