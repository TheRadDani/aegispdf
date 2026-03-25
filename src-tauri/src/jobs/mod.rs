//! Background job queue for long-running PDF work (OCR, analysis, merge, split).

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use lopdf::Document;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::core::detection;
use crate::core::merge;
use crate::core::ocr;
use crate::core::split;
use crate::error::{AegisError, AegisErrorResponse};
use crate::render::pdfium_renderer;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobKind {
    Merge {
        inputs: Vec<String>,
        output: String,
    },
    Split {
        source: String,
        ranges: Vec<(u32, u32)>,
        outputs: Vec<String>,
    },
    Analyze {
        path: String,
    },
    Ocr {
        path: String,
        output_txt: String,
        #[serde(default = "default_lang")]
        lang: String,
    },
}

fn default_lang() -> String {
    "eng".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub struct JobEvent {
    pub job_id: String,
    pub phase: String,
    pub progress: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AegisErrorResponse>,
}

struct QueuedJob {
    id: String,
    kind: JobKind,
}

pub struct JobQueue {
    tx: mpsc::Sender<QueuedJob>,
    #[allow(dead_code)]
    worker: thread::JoinHandle<()>,
}

impl JobQueue {
    pub fn spawn(app: AppHandle) -> Self {
        let (tx, rx) = mpsc::channel::<QueuedJob>();
        let worker = thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                let emit = |evt: JobEvent| {
                    let _ = app.emit("aegis-job-event", &evt);
                };
                emit(JobEvent {
                    job_id: job.id.clone(),
                    phase: "started".into(),
                    progress: 0.0,
                    result: None,
                    error: None,
                });
                let outcome = run_job(&job.id, &job.kind, &emit);
                match outcome {
                    Ok(value) => emit(JobEvent {
                        job_id: job.id,
                        phase: "complete".into(),
                        progress: 1.0,
                        result: Some(value),
                        error: None,
                    }),
                    Err(e) => emit(JobEvent {
                        job_id: job.id,
                        phase: "error".into(),
                        progress: 1.0,
                        result: None,
                        error: Some(AegisErrorResponse::from(e)),
                    }),
                }
            }
        });
        Self { tx, worker }
    }

    pub fn submit(&self, kind: JobKind) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        self.tx
            .send(QueuedJob {
                id: id.clone(),
                kind,
            })
            .map_err(|_| "job queue unavailable".to_string())?;
        Ok(id)
    }
}

fn run_job(
    job_id: &str,
    kind: &JobKind,
    emit: &impl Fn(JobEvent),
) -> Result<serde_json::Value, AegisError> {
    match kind {
        JobKind::Merge { inputs, output } => {
            let paths: Vec<PathBuf> = inputs.iter().map(PathBuf::from).collect();
            let out = PathBuf::from(output);
            merge::merge_pdfs(&paths, &out)?;
            Ok(serde_json::json!({ "output": output, "job_id": job_id }))
        }
        JobKind::Split {
            source,
            ranges,
            outputs,
        } => {
            if ranges.len() != outputs.len() {
                return Err(AegisError::InvalidArgument(
                    "ranges and outputs length mismatch".into(),
                ));
            }
            let src = PathBuf::from(source);
            let outs: Vec<PathBuf> = outputs.iter().map(PathBuf::from).collect();
            split::split_pdf_by_ranges(&src, ranges, &outs)?;
            Ok(serde_json::json!({ "outputs": outputs }))
        }
        JobKind::Analyze { path } => {
            emit(JobEvent {
                job_id: job_id.to_string(),
                phase: "running".into(),
                progress: 0.2,
                result: None,
                error: None,
            });
            let doc = Document::load(path).map_err(|e| AegisError::pdf("load", e.to_string()))?;
            let analysis = detection::analyze_pages(&doc, 8.0)?;
            Ok(serde_json::to_value(&analysis).map_err(|e| AegisError::Job(e.to_string()))?)
        }
        JobKind::Ocr {
            path,
            output_txt,
            lang,
        } => {
            let doc = Document::load(path).map_err(|e| AegisError::pdf("load", e.to_string()))?;
            let n = doc.get_pages().len().max(1);
            let out_path = PathBuf::from(output_txt);
            if out_path.exists() {
                std::fs::remove_file(&out_path).map_err(AegisError::from)?;
            }
            for idx in 0..doc.get_pages().len() {
                #[allow(clippy::cast_precision_loss)]
                let progress = 0.1_f32 + 0.85_f32 * (idx as f32) / (n as f32);
                emit(JobEvent {
                    job_id: job_id.to_string(),
                    phase: "running".into(),
                    progress,
                    result: None,
                    error: None,
                });
                let png = pdfium_renderer::render_page_png(&doc, idx, 2000)
                    .map_err(|e| AegisError::Render(e.to_string()))?;
                let text = ocr::ocr_png_bytes(&png, lang)?;
                ocr::append_page_text(&out_path, idx, &text)?;
            }
            Ok(serde_json::json!({ "output_txt": output_txt, "pages": n }))
        }
    }
}
