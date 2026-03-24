# AegisPDF

Cross-platform, offline-first PDF workspace built with Tauri + Rust + React.

## Features

### Workspace (Phase 1–2)

- Open / save PDF
- PDFium thumbnails, zoom, drag-and-drop reorder (updates `/Pages` `/Kids`)
- Multi-select (Ctrl/Cmd, Shift) and delete pages
- Structured errors returned as JSON objects to the UI

### Phase 3 — pipeline

- **Merge** multiple PDFs with object-id remapping (`core/merge.rs`)
- **Split** by ranges or **one file per page** (`core/split.rs`, `split_pdf_each_page`)
- **Compress** workspace: prune, zero-length stream removal, lopdf compress, optional decompress→recompress round-trip; **flate2** helper for tooling (`core/compress.rs`)
- **Auto-clean**: strip metadata, optional embedded `/Annots` removal (`core/security.rs`)

### Phase 4 — intelligence & OCR

- **Blank / duplicate** page hints via downscaled render fingerprints (`core/detection.rs`)
- **OCR** text export via **Tesseract** CLI (`core/ocr.rs`); requires `tesseract` on `PATH`
- **Annotations**: non-destructive sidecar `document.aegis` JSON, hash-bound to the PDF (`core/annotations.rs`); highlight + text note overlays in the UI

### Production-oriented backend

- **Error taxonomy** (`error.rs`): `AegisError` → serializable `AegisErrorResponse` for IPC
- **Job queue** (`jobs/mod.rs`): background worker thread emits `aegis-job-event` for merge/split/analyze/OCR jobs (`submit_job`)
- **Integration tests** (no PDFium): `cargo test --manifest-path src-tauri/Cargo.toml` runs merge/split/reorder checks in `tests/merge_split.rs`

## Project structure

- `src-tauri/src/core`: `pdf`, `pages`, `merge`, `split`, `compress`, `ocr`, `detection`, `annotations`, `security`
- `src-tauri/src/render`: `pdfium_renderer` (thumbnails, fingerprints, OCR renders)
- `src-tauri/src/commands`: Tauri command handlers
- `src-tauri/src/jobs`: job types + worker
- `src-tauri/src/error.rs`: shared errors
- `src`: React UI (`components`, `hooks`, `services`)

## Requirements

- Node.js 20+
- Rust toolchain
- Tauri prerequisites for your OS
- PDFium (system library) for rendering
- **Tesseract** for OCR jobs (optional for other features)

## Run

```bash
npm install
npm run tauri dev
```

## Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

## Notes

- **Page reorder** assumes a flat `/Pages` tree with `/Kids` pointing at page objects (typical for many files; deeply nested page trees may need a future flattener).
- **llama.cpp** is not integrated; keep job payload enums extensible for future AI tasks.
- Frontend listens for **`aegis-job-event`**; if events are blocked by your Tauri capability setup, add the appropriate `core:event` listen permission for that channel.
