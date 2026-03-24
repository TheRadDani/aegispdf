pub mod commands;
pub mod core;
pub mod error;
pub mod jobs;
pub mod render;

use std::collections::HashMap;
use std::sync::Mutex;

use core::pdf::PdfWorkspace;
use tauri::Emitter;

pub use error::{AegisError, AegisErrorResponse};

/// Open PDF workspaces keyed by session id.
pub struct Workspaces {
    pub documents: Mutex<HashMap<String, PdfWorkspace>>,
}

impl Default for Workspaces {
    fn default() -> Self {
        Self {
            documents: Mutex::new(HashMap::new()),
        }
    }
}

/// Extract the first file-path argument that looks like a PDF or .aegis file.
/// Used when the OS launches AegisPDF via a file association (e.g. double-click
/// a .pdf on Windows, or `xdg-open file.pdf` on Linux).
fn extract_file_arg() -> Option<String> {
    std::env::args()
        .skip(1)                           // skip argv[0] (binary name)
        .find(|arg| {
            if arg.starts_with('-') {
                return false;              // skip flag arguments
            }
            let path = std::path::Path::new(arg);
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            (ext == "pdf" || ext == "aegis") && path.exists()
        })
}

pub fn run() {
    let initial_file = extract_file_arg();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            let handle = app.handle().clone();
            app.manage(Workspaces::default());
            app.manage(jobs::JobQueue::spawn(handle.clone()));

            // If a file was passed on the command line (e.g. double-click PDF),
            // wait until the webview finishes loading, then emit the path so
            // the frontend auto-opens it.
            if let Some(path) = initial_file.clone() {
                std::thread::spawn(move || {
                    // 900 ms covers typical webview startup on slow machines.
                    std::thread::sleep(std::time::Duration::from_millis(900));
                    let _ = handle.emit("aegis://open-file", &path);
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::open_pdf,
            commands::get_page_thumbnail,
            commands::reorder_pages,
            commands::delete_pages,
            commands::get_page_list,
            commands::save_pdf,
            commands::merge_pdfs_paths,
            commands::split_pdf_paths,
            commands::split_pdf_each_page,
            commands::compress_workspace,
            commands::auto_clean_workspace,
            commands::load_aegis,
            commands::save_aegis,
            commands::submit_job,
            commands::export_pdf_temp,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
