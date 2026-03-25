use std::io::Cursor;
use std::path::{Path, PathBuf};

use base64::Engine;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
use lopdf::Document;
#[allow(clippy::wildcard_imports)]
use pdfium_render::prelude::*;
use sha2::{Digest, Sha256};

/// Serialize the document to bytes so `PDFium` can load it.
fn doc_to_bytes(document: &Document) -> anyhow::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    document.clone().save_to(&mut bytes)?;
    Ok(bytes)
}

/// Try to locate the `PDFium` shared library by probing multiple paths in order.
/// Returns the first existing path, or `None` (which means fall back to system search).
fn find_pdfium_library(tauri_resource_hint: Option<&Path>) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. Tauri resource directory (set by lib.rs at startup via BaseDirectory::Resource)
    if let Some(p) = tauri_resource_hint {
        candidates.push(p.to_path_buf());
    }

    // 2. Next to the running executable (works for AppImage, Windows, dev builds)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            #[cfg(target_os = "linux")]
            {
                candidates.push(exe_dir.join("libs/libpdfium.so"));
                candidates.push(exe_dir.join("libpdfium.so"));
            }
            #[cfg(target_os = "windows")]
            {
                candidates.push(exe_dir.join("libs/pdfium.dll"));
                candidates.push(exe_dir.join("pdfium.dll"));
            }
            #[cfg(target_os = "macos")]
            {
                candidates.push(exe_dir.join("libs/libpdfium.dylib"));
                candidates.push(exe_dir.join("libpdfium.dylib"));
            }
        }
    }

    // 3. Well-known system paths
    #[cfg(target_os = "linux")]
    {
        candidates.push(PathBuf::from("/usr/lib/aegispdf/libpdfium.so"));
        candidates.push(PathBuf::from("/usr/local/lib/libpdfium.so"));
        candidates.push(PathBuf::from("/usr/lib/libpdfium.so"));
        candidates.push(PathBuf::from("/usr/lib/x86_64-linux-gnu/libpdfium.so"));
    }

    candidates.into_iter().find(|p| p.exists())
}

/// Load `PDFium`, trying bundled/well-known paths first, then falling back to system library search.
fn load_pdfium(tauri_resource_hint: Option<&Path>) -> anyhow::Result<Pdfium> {
    if let Some(lib_path) = find_pdfium_library(tauri_resource_hint) {
        let path_str = lib_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("PDFium library path is not valid UTF-8"))?;
        let bindings =
            Pdfium::bind_to_library(path_str).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        return Ok(Pdfium::new(bindings));
    }
    let bindings = Pdfium::bind_to_system_library().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(Pdfium::new(bindings))
}

/// Raw PNG bytes for OCR pipelines.
///
/// # Errors
///
/// Returns an error if `PDFium` cannot be loaded, the PDF parsed, the page rendered,
/// or the PNG encoded.
pub fn render_page_png(
    document: &Document,
    page_index: usize,
    target_width: i32,
    pdfium_path: Option<&Path>,
) -> anyhow::Result<Vec<u8>> {
    let pdfium = load_pdfium(pdfium_path)?;
    let bytes = doc_to_bytes(document)?;
    let loaded = pdfium.load_pdf_from_byte_vec(bytes, None)?;
    let page = loaded.pages().get(u16::try_from(page_index)?)?;
    let bitmap = page.render_with_config(
        &PdfRenderConfig::new()
            .set_target_width(target_width)
            .render_form_data(true)
            .rotate_if_landscape(PdfPageRenderRotation::None, false),
    )?;
    let raw = bitmap.as_rgba_bytes();
    let bmp_w = u32::try_from(bitmap.width()).map_err(|e| anyhow::anyhow!("bitmap width: {e}"))?;
    let bmp_h =
        u32::try_from(bitmap.height()).map_err(|e| anyhow::anyhow!("bitmap height: {e}"))?;
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(bmp_w, bmp_h, raw)
        .ok_or_else(|| anyhow::anyhow!("invalid bitmap buffer"))?;
    let mut out = Cursor::new(Vec::<u8>::new());
    DynamicImage::ImageRgba8(buffer).write_to(&mut out, ImageFormat::Png)?;
    Ok(out.into_inner())
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
/// # Errors
///
/// Returns an error if the page cannot be rendered or base64-encoded.
pub fn render_page_thumbnail_base64(
    document: &Document,
    page_index: usize,
    zoom: f32,
    pdfium_path: Option<&Path>,
) -> anyhow::Result<String> {
    let target_width = (220.0 * zoom).clamp(100.0, 800.0) as i32;
    let png = render_page_png(document, page_index, target_width, pdfium_path)?;
    let data_url = format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&png)
    );
    Ok(data_url)
}

/// Downscale render for fingerprinting; returns `(sha256_hex, mean_abs_deviation_from_white)`.
///
/// # Errors
///
/// Returns an error if `PDFium` cannot be loaded, the PDF parsed, or the page rendered.
pub fn page_render_fingerprint(
    document: &Document,
    page_index: usize,
    target_width: i32,
    pdfium_path: Option<&Path>,
) -> anyhow::Result<(String, f32)> {
    let pdfium = load_pdfium(pdfium_path)?;
    let bytes = doc_to_bytes(document)?;
    let loaded = pdfium.load_pdf_from_byte_vec(bytes, None)?;
    let page = loaded.pages().get(u16::try_from(page_index)?)?;
    let bitmap = page.render_with_config(
        &PdfRenderConfig::new()
            .set_target_width(target_width)
            .render_form_data(true)
            .rotate_if_landscape(PdfPageRenderRotation::None, false),
    )?;
    let raw = bitmap.as_rgba_bytes();
    let mut hasher = Sha256::new();
    hasher.update(&raw);
    let hex = format!("{:x}", hasher.finalize());

    let mut mad_sum = 0_f64;
    let mut mad_count = 0_u32;
    for chunk in raw.chunks(4) {
        if let [r, g, b, _] = chunk {
            let r = f64::from(*r);
            let g = f64::from(*g);
            let b = f64::from(*b);
            mad_sum += (255.0 - r).abs() + (255.0 - g).abs() + (255.0 - b).abs();
            mad_count = mad_count.saturating_add(3);
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    let mad = if mad_count == 0 {
        0.0
    } else {
        (mad_sum / f64::from(mad_count)) as f32
    };
    Ok((hex, mad))
}
