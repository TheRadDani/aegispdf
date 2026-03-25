use std::io::Cursor;

use base64::Engine;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
use lopdf::Document;
#[allow(clippy::wildcard_imports)]
use pdfium_render::prelude::*;
use sha2::{Digest, Sha256};

/// Serialize the document to bytes so PDFium can load it.
fn doc_to_bytes(document: &Document) -> anyhow::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    document.clone().save_to(&mut bytes)?;
    Ok(bytes)
}

/// Raw PNG bytes for OCR pipelines.
pub fn render_page_png(
    document: &Document,
    page_index: usize,
    target_width: i32,
) -> anyhow::Result<Vec<u8>> {
    let bindings = Pdfium::bind_to_system_library().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let pdfium = Pdfium::new(bindings);
    let bytes = doc_to_bytes(document)?;
    let loaded = pdfium.load_pdf_from_byte_vec(bytes, None)?;
    let page = loaded.pages().get(page_index as u16)?;
    let bitmap = page.render_with_config(
        &PdfRenderConfig::new()
            .set_target_width(target_width)
            .render_form_data(true)
            .rotate_if_landscape(PdfPageRenderRotation::None, false),
    )?;
    let raw = bitmap.as_rgba_bytes();
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(
        bitmap.width() as u32,
        bitmap.height() as u32,
        raw.to_vec(),
    )
    .ok_or_else(|| anyhow::anyhow!("invalid bitmap buffer"))?;
    let mut out = Cursor::new(Vec::<u8>::new());
    DynamicImage::ImageRgba8(buffer).write_to(&mut out, ImageFormat::Png)?;
    Ok(out.into_inner())
}

pub fn render_page_thumbnail_base64(
    document: &Document,
    page_index: usize,
    zoom: f32,
) -> anyhow::Result<String> {
    let target_width = (220.0 * zoom).clamp(100.0, 800.0) as i32;
    let png = render_page_png(document, page_index, target_width)?;
    let data_url = format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&png)
    );
    Ok(data_url)
}

/// Downscale render for fingerprinting; returns `(sha256_hex, mean_abs_deviation_from_white)`.
pub fn page_render_fingerprint(
    document: &Document,
    page_index: usize,
    target_width: i32,
) -> anyhow::Result<(String, f32)> {
    let bindings = Pdfium::bind_to_system_library().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let pdfium = Pdfium::new(bindings);
    let bytes = doc_to_bytes(document)?;
    let loaded = pdfium.load_pdf_from_byte_vec(bytes, None)?;
    let page = loaded.pages().get(page_index as u16)?;
    let bitmap = page.render_with_config(
        &PdfRenderConfig::new()
            .set_target_width(target_width)
            .render_form_data(true)
            .rotate_if_landscape(PdfPageRenderRotation::None, false),
    )?;
    let raw = bitmap.as_rgba_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&raw); // borrow, not move — raw is used again below
    let hex = format!("{:x}", hasher.finalize());

    let mut mad_sum = 0_f64;
    let mut n = 0_u64;
    for chunk in raw.chunks(4) {
        if let [r, g, b, _] = chunk {
            let r = f64::from(*r);
            let g = f64::from(*g);
            let b = f64::from(*b);
            mad_sum += (255.0 - r).abs() + (255.0 - g).abs() + (255.0 - b).abs();
            n = n.saturating_add(3);
        }
    }
    let mad = if n == 0 {
        0.0
    } else {
        (mad_sum / n as f64) as f32
    };
    Ok((hex, mad))
}
