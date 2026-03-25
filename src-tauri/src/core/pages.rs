use lopdf::{Document, Object};

/// Reorder pages by assigning root `/Pages` `/Kids` in the given order (flat page trees).
/// PDF page labels are 1-based keys from [`Document::get_pages`].
///
/// # Errors
/// Returns an error if a page number in `new_order` is not present in the document,
/// or if the pages root or catalog cannot be found.
pub fn reorder_pages_by_page_number(
    document: &mut Document,
    new_order: &[u32],
) -> anyhow::Result<()> {
    let pages_map = document.get_pages();
    let kids = new_order
        .iter()
        .map(|pn| {
            pages_map
                .get(pn)
                .copied()
                .map(Object::Reference)
                .ok_or_else(|| anyhow::anyhow!("missing page {pn}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Extract pages_id in a limited scope so the shared borrow on `document`
    // from `catalog()` is dropped before the mutable borrow below.
    let pages_id = {
        let catalog = document.catalog()?;
        catalog
            .get(b"Pages")
            .and_then(Object::as_reference)
            .map_err(|_| anyhow::anyhow!("catalog missing Pages entry"))?
    };

    let dict = document.get_dictionary_mut(pages_id)?;
    dict.set(b"Kids", Object::Array(kids));
    let count = i64::try_from(new_order.len())
        .map_err(|e| anyhow::anyhow!("page count overflow: {e}"))?;
    dict.set(b"Count", Object::Integer(count));
    Ok(())
}

/// # Errors
/// Returns an error if any index is out of range or all pages would be deleted.
pub fn delete_pages_by_indices(
    document: &mut Document,
    indices: &[usize],
    ordered_page_numbers: &[u32],
) -> anyhow::Result<()> {
    let mut to_delete = Vec::new();
    for index in indices {
        let page_number = ordered_page_numbers
            .get(*index)
            .ok_or_else(|| anyhow::anyhow!("invalid page index {index}"))?;
        to_delete.push(*page_number);
    }
    document.delete_pages(&to_delete);
    let pages = document.get_pages();
    if pages.is_empty() {
        return Err(anyhow::anyhow!("cannot delete all pages"));
    }

    document.adjust_zero_pages();
    document
        .objects
        .retain(|_, object| !matches!(object, Object::Null));
    Ok(())
}
