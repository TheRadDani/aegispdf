use std::path::PathBuf;

use lopdf::Document;
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::core::annotations::AnnotationStore;
use crate::core::merge;
use crate::core::pdf::{PageInfo, PdfWorkspace};
use crate::core::split;
use crate::error::{AegisError, AegisErrorResponse, AegisResult};
use crate::jobs::{JobKind, JobQueue};
use crate::Workspaces;

#[derive(Debug, Serialize)]
pub struct OpenPdfResponse {
    pub document_id: String,
    pub page_count: usize,
    pub page_infos: Vec<PageInfo>,
    pub file_hash: String,
    pub source_path: String,
}

fn workspaces<'a>(state: &'a State<'_, Workspaces>) -> AegisResult<std::sync::MutexGuard<'a, std::collections::HashMap<String, PdfWorkspace>>> {
    state
        .documents
        .lock()
        .map_err(|_| AegisError::LockPoisoned)
}

#[tauri::command]
pub fn open_pdf(path: String, state: State<'_, Workspaces>) -> Result<OpenPdfResponse, AegisErrorResponse> {
    let path_buf = PathBuf::from(&path);
    let workspace = PdfWorkspace::open(&path_buf).map_err(|e| AegisError::pdf("open", e.to_string()))?;
    let page_count = workspace.page_count();
    let page_infos = workspace.page_infos();
    let file_hash = workspace.file_hash.clone();
    let source_path = workspace.source_path.to_string_lossy().to_string();
    let document_id = Uuid::new_v4().to_string();

    let mut lock = workspaces(&state).map_err(AegisErrorResponse::from)?;
    lock.insert(document_id.clone(), workspace);

    Ok(OpenPdfResponse {
        document_id,
        page_count,
        page_infos,
        file_hash,
        source_path,
    })
}

#[tauri::command]
pub fn get_page_thumbnail(
    document_id: String,
    page_index: usize,
    zoom: f32,
    state: State<'_, Workspaces>,
) -> Result<String, AegisErrorResponse> {
    let mut lock = workspaces(&state).map_err(AegisErrorResponse::from)?;
    let workspace = lock
        .get_mut(&document_id)
        .ok_or(AegisError::DocumentNotFound)?;
    crate::render::pdfium_renderer::render_page_thumbnail_base64(&workspace.document, page_index, zoom)
        .map_err(|e| AegisErrorResponse::from(AegisError::Render(e.to_string())))
}

#[tauri::command]
pub fn reorder_pages(
    document_id: String,
    new_order: Vec<u32>,
    state: State<'_, Workspaces>,
) -> Result<(), AegisErrorResponse> {
    let mut lock = workspaces(&state).map_err(AegisErrorResponse::from)?;
    let workspace = lock
        .get_mut(&document_id)
        .ok_or(AegisError::DocumentNotFound)?;
    workspace
        .reorder_pages_by_number(&new_order)
        .map_err(|e| AegisErrorResponse::from(AegisError::pdf("reorder", e.to_string())))?;
    Ok(())
}

#[tauri::command]
pub fn delete_pages(
    document_id: String,
    indices: Vec<usize>,
    state: State<'_, Workspaces>,
) -> Result<Vec<PageInfo>, AegisErrorResponse> {
    let mut lock = workspaces(&state).map_err(AegisErrorResponse::from)?;
    let workspace = lock
        .get_mut(&document_id)
        .ok_or(AegisError::DocumentNotFound)?;
    workspace
        .delete_pages_by_indices(&indices)
        .map_err(|e| AegisErrorResponse::from(AegisError::pdf("delete_pages", e.to_string())))?;
    Ok(workspace.page_infos())
}

#[tauri::command]
pub fn get_page_list(document_id: String, state: State<'_, Workspaces>) -> Result<Vec<PageInfo>, AegisErrorResponse> {
    let lock = workspaces(&state).map_err(AegisErrorResponse::from)?;
    let workspace = lock.get(&document_id).ok_or(AegisError::DocumentNotFound)?;
    Ok(workspace.page_infos())
}

#[tauri::command]
pub fn save_pdf(
    document_id: String,
    output_path: String,
    state: State<'_, Workspaces>,
) -> Result<(), AegisErrorResponse> {
    let mut lock = workspaces(&state).map_err(AegisErrorResponse::from)?;
    let workspace = lock
        .get_mut(&document_id)
        .ok_or(AegisError::DocumentNotFound)?;
    workspace
        .save_to(&PathBuf::from(output_path))
        .map_err(|e| AegisErrorResponse::from(AegisError::pdf("save", e.to_string())))?;
    Ok(())
}

#[tauri::command]
pub fn merge_pdfs_paths(inputs: Vec<String>, output: String) -> Result<(), AegisErrorResponse> {
    let paths: Vec<PathBuf> = inputs.iter().map(PathBuf::from).collect();
    merge::merge_pdfs(&paths, &PathBuf::from(output)).map_err(AegisErrorResponse::from)
}

#[tauri::command]
pub fn split_pdf_paths(
    source: String,
    ranges: Vec<(u32, u32)>,
    outputs: Vec<String>,
) -> Result<(), AegisErrorResponse> {
    if ranges.len() != outputs.len() {
        return Err(AegisErrorResponse::from(AegisError::InvalidArgument(
            "ranges and outputs length mismatch".into(),
        )));
    }
    let outs: Vec<PathBuf> = outputs.iter().map(PathBuf::from).collect();
    split::split_pdf_by_ranges(&PathBuf::from(source), &ranges, &outs).map_err(AegisErrorResponse::from)
}

/// One PDF per page in `output_dir` (`page_NNN.pdf`).
#[tauri::command]
pub fn split_pdf_each_page(source: String, output_dir: String) -> Result<Vec<String>, AegisErrorResponse> {
    let src = PathBuf::from(&source);
    let dir = PathBuf::from(&output_dir);
    std::fs::create_dir_all(&dir).map_err(AegisError::from)?;
    let doc = Document::load(&src).map_err(|e| AegisErrorResponse::from(AegisError::pdf("load", e.to_string())))?;
    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    let mut outs = Vec::new();
    for p in pages {
        let out = dir.join(format!("page_{p:03}.pdf"));
        split::split_pdf_by_ranges(&src, &[(p, p)], &[out.clone()]).map_err(AegisErrorResponse::from)?;
        outs.push(out.to_string_lossy().to_string());
    }
    Ok(outs)
}

#[tauri::command]
pub fn compress_workspace(
    document_id: String,
    roundtrip_streams: bool,
    state: State<'_, Workspaces>,
) -> Result<(), AegisErrorResponse> {
    let mut lock = workspaces(&state).map_err(AegisErrorResponse::from)?;
    let workspace = lock
        .get_mut(&document_id)
        .ok_or(AegisError::DocumentNotFound)?;
    workspace.apply_smart_compress(roundtrip_streams);
    Ok(())
}

#[tauri::command]
pub fn auto_clean_workspace(
    document_id: String,
    strip_annots: bool,
    state: State<'_, Workspaces>,
) -> Result<(), AegisErrorResponse> {
    let mut lock = workspaces(&state).map_err(AegisErrorResponse::from)?;
    let workspace = lock
        .get_mut(&document_id)
        .ok_or(AegisError::DocumentNotFound)?;
    workspace.apply_auto_clean(strip_annots);
    Ok(())
}

#[tauri::command]
pub fn load_aegis(pdf_path: String, file_hash: String) -> Result<AnnotationStore, AegisErrorResponse> {
    AnnotationStore::load_for_pdf(&PathBuf::from(pdf_path), &file_hash).map_err(AegisErrorResponse::from)
}

#[tauri::command]
pub fn save_aegis(pdf_path: String, store: AnnotationStore) -> Result<(), AegisErrorResponse> {
    store.save(&PathBuf::from(pdf_path)).map_err(AegisErrorResponse::from)
}

#[tauri::command]
pub fn submit_job(kind: JobKind, queue: State<'_, JobQueue>) -> Result<String, AegisErrorResponse> {
    queue.submit(kind).map_err(|e| AegisErrorResponse::from(AegisError::Job(e)))
}

/// Write current workspace PDF to a temp file for OCR / analyze jobs (releases lock before heavy work).
#[tauri::command]
pub fn export_pdf_temp(document_id: String, state: State<'_, Workspaces>) -> Result<String, AegisErrorResponse> {
    let mut lock = workspaces(&state).map_err(AegisErrorResponse::from)?;
    let workspace = lock
        .get_mut(&document_id)
        .ok_or(AegisError::DocumentNotFound)?;
    let path = std::env::temp_dir().join(format!(
        "aegispdf_{}_{}.pdf",
        document_id,
        uuid::Uuid::new_v4()
    ));
    workspace
        .document
        .save(&path)
        .map_err(|e| AegisErrorResponse::from(AegisError::pdf("save_temp", e.to_string())))?;
    Ok(path.to_string_lossy().to_string())
}
