import { invoke } from "@tauri-apps/api/core";
import type { AnnotationStore, OpenPdfResponse, PageInfo } from "../types";
import type { JobKind } from "../types";

export async function openPdf(path: string): Promise<OpenPdfResponse> {
  return invoke<OpenPdfResponse>("open_pdf", { path });
}

export async function getPageThumbnail(documentId: string, pageIndex: number, zoom: number): Promise<string> {
  return invoke<string>("get_page_thumbnail", { documentId, pageIndex, zoom });
}

export async function reorderPages(documentId: string, newOrder: number[]): Promise<void> {
  return invoke("reorder_pages", { documentId, newOrder });
}

export async function deletePages(documentId: string, indices: number[]): Promise<PageInfo[]> {
  return invoke<PageInfo[]>("delete_pages", { documentId, indices });
}

export async function getPageList(documentId: string): Promise<PageInfo[]> {
  return invoke<PageInfo[]>("get_page_list", { documentId });
}

export async function savePdf(documentId: string, outputPath: string): Promise<void> {
  return invoke("save_pdf", { documentId, outputPath });
}

export async function mergePdfPaths(inputs: string[], output: string): Promise<void> {
  return invoke("merge_pdfs_paths", { inputs, output });
}

export async function splitPdfEachPage(source: string, outputDir: string): Promise<string[]> {
  return invoke<string[]>("split_pdf_each_page", { source, outputDir });
}

export async function compressWorkspace(documentId: string, roundtripStreams: boolean): Promise<void> {
  return invoke("compress_workspace", { documentId, roundtripStreams });
}

export async function autoCleanWorkspace(documentId: string, stripAnnots: boolean): Promise<void> {
  return invoke("auto_clean_workspace", { documentId, stripAnnots });
}

export async function loadAegis(pdfPath: string, fileHash: string): Promise<AnnotationStore> {
  return invoke<AnnotationStore>("load_aegis", { pdfPath, fileHash });
}

export async function saveAegis(pdfPath: string, store: AnnotationStore): Promise<void> {
  return invoke("save_aegis", { pdfPath, store });
}

export async function submitJob(kind: JobKind): Promise<string> {
  return invoke<string>("submit_job", { kind });
}

export async function exportPdfTemp(documentId: string): Promise<string> {
  return invoke<string>("export_pdf_temp", { documentId });
}
