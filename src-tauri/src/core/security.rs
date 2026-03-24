use lopdf::{Document, Object};

/// Remove sensitive metadata keys from trailer/info dictionary.
pub fn strip_metadata(document: &mut Document) {
    if let Ok(info_ref) = document.trailer.get(b"Info").and_then(Object::as_reference) {
        if let Ok(info) = document.get_object_mut(info_ref).and_then(Object::as_dict_mut) {
            info.remove(b"Author");
            info.remove(b"Creator");
            info.remove(b"Producer");
            info.remove(b"Subject");
            info.remove(b"Title");
            info.remove(b"Keywords");
        }
    }
}

/// Remove `/Annots` from each page dictionary (embedded PDF annotations only).
pub fn strip_page_annots(document: &mut Document) {
    let page_ids: Vec<lopdf::ObjectId> = document.get_pages().into_values().collect();
    for id in page_ids {
        if let Ok(dict) = document.get_dictionary_mut(id) {
            dict.remove(b"Annots");
        }
    }
}

/// Metadata cleanup, optional embedded annotation strip, then prune.
pub fn auto_clean(document: &mut Document, strip_annots: bool) {
    strip_metadata(document);
    if strip_annots {
        strip_page_annots(document);
    }
    document.prune_objects();
}
