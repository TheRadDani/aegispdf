use lopdf::{Document, Object};

/// Reorder pages by assigning root `/Pages` `/Kids` in the given order (flat page trees).
/// PDF page labels are 1-based keys from [`Document::get_pages`].
pub fn reorder_pages_by_page_number(
    document: &mut Document,
    new_order: &[u32],
) -> anyhow::Result<()> {
    let pages_map = document.get_pages();
    for pn in new_order {
        if !pages_map.contains_key(pn) {
            anyhow::bail!("missing page {pn}");
        }
    }
    let kids: Vec<Object> = new_order
        .iter()
        .map(|pn| Object::Reference(*pages_map.get(pn).expect("validated")))
        .collect();

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
    dict.set(b"Count", Object::Integer(new_order.len() as i64));
    Ok(())
}

pub fn delete_pages_by_indices(
    document: &mut Document,
    indices: &[usize],
    ordered_page_numbers: &[u32],
) -> anyhow::Result<()> {
    let mut to_delete = Vec::new();
    for index in indices {
        let page_number = ordered_page_numbers
            .get(*index)
            .ok_or_else(|| anyhow::anyhow!("invalid page index {}", index))?;
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
