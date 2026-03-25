//! Merge multiple PDFs with object-id remapping (lopdf merge pattern, bookmarks omitted).

use std::path::PathBuf;

use lopdf::{Document, Object, ObjectId};

use crate::error::{AegisError, AegisResult};

fn pdf_type(object: &Object) -> Option<&[u8]> {
    match object {
        Object::Dictionary(d) => d.get(b"Type").ok()?.as_name().ok(),
        Object::Stream(s) => s.dict.get(b"Type").ok()?.as_name().ok(),
        _ => None,
    }
}

/// Merge PDFs in path order into `output`. Rewrites object IDs to avoid collisions.
///
/// # Errors
/// Returns `AegisError::InvalidArgument` if `inputs` is empty, or `AegisError::Merge`
/// if any PDF cannot be loaded or the merged file cannot be saved.
pub fn merge_pdfs(inputs: &[PathBuf], output: &PathBuf) -> AegisResult<()> {
    if inputs.is_empty() {
        return Err(AegisError::InvalidArgument("no input PDFs".into()));
    }
    let mut documents: Vec<Document> = Vec::with_capacity(inputs.len());
    for p in inputs {
        let d =
            Document::load(p).map_err(|e| AegisError::Merge(format!("{}: {e}", p.display())))?;
        documents.push(d);
    }
    let merged = merge_documents(documents).map_err(|e| AegisError::Merge(e.to_string()))?;
    let mut out = merged;
    out.prune_objects();
    out.compress();
    out.save(output)
        .map_err(|e| AegisError::pdf("save", e.to_string()))?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn merge_documents(documents: Vec<Document>) -> anyhow::Result<Document> {
    let mut max_id = 1_u32;
    // Preserve page order (not ObjectId sort order).
    let mut documents_pages: Vec<(ObjectId, Object)> = Vec::new();
    let mut documents_objects: std::collections::BTreeMap<ObjectId, Object> =
        std::collections::BTreeMap::new();
    let mut document = Document::with_version("1.7");

    for mut doc in documents {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id.saturating_add(1);

        let pages = doc.get_pages();
        for object_id in pages.into_values() {
            let value = doc.get_object(object_id)?.clone();
            documents_pages.push((object_id, value));
        }
        documents_objects.extend(doc.objects);
    }

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in documents_objects {
        match pdf_type(&object).unwrap_or(b"") {
            b"Catalog" => {
                catalog_object = Some((
                    if let Some((id, _)) = catalog_object {
                        id
                    } else {
                        object_id
                    },
                    object,
                ));
            }
            b"Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref prev_obj)) = pages_object {
                        if let Ok(old_dictionary) = prev_obj.as_dict() {
                            for (k, v) in old_dictionary.as_hashmap() {
                                dictionary.as_hashmap_mut().insert(k.clone(), v.clone());
                            }
                        }
                    }
                    pages_object = Some((
                        if let Some((id, _)) = pages_object {
                            id
                        } else {
                            object_id
                        },
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            b"Page" | b"Outlines" | b"Outline" => {}
            _ => {
                document.objects.insert(object_id, object);
            }
        }
    }

    let (page_id, pages_root) =
        pages_object.ok_or_else(|| anyhow::anyhow!("pages root not found"))?;
    let (catalog_id, catalog_object) =
        catalog_object.ok_or_else(|| anyhow::anyhow!("catalog root not found"))?;

    for (object_id, object) in &documents_pages {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set(b"Parent", Object::Reference(page_id));
            document
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    if let Ok(dictionary) = pages_root.as_dict() {
        let mut dictionary = dictionary.clone();
        let page_count = i64::try_from(documents_pages.len())
            .map_err(|e| anyhow::anyhow!("page count overflow: {e}"))?;
        dictionary.set(b"Count", Object::Integer(page_count));
        dictionary.set(
            b"Kids",
            Object::Array(
                documents_pages
                    .iter()
                    .map(|(id, _)| Object::Reference(*id))
                    .collect::<Vec<_>>(),
            ),
        );
        document
            .objects
            .insert(page_id, Object::Dictionary(dictionary));
    }

    if let Ok(dictionary) = catalog_object.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set(b"Pages", Object::Reference(page_id));
        dictionary.remove(b"Outlines");
        document
            .objects
            .insert(catalog_id, Object::Dictionary(dictionary));
    }

    document.trailer.set(b"Root", Object::Reference(catalog_id));
    document.max_id = u32::try_from(document.objects.len())
        .map_err(|_| anyhow::anyhow!("object count overflow"))?;
    document.renumber_objects();
    document.adjust_zero_pages();
    Ok(document)
}
