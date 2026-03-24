export type AnnotationType = "highlight" | "text_note" | "drawing";

export interface Annotation {
  id: string;
  page_index: number;
  annotation_type: AnnotationType;
  payload: Record<string, unknown>;
}

export interface AnnotationStore {
  pdf_hash: string;
  annotations: Annotation[];
}

export interface PageInfo {
  index: number;
  page_number: number;
}

export interface OpenPdfResponse {
  document_id: string;
  page_count: number;
  page_infos: PageInfo[];
  file_hash: string;
  source_path: string;
}

/** Matches Rust `JobKind` (serde tag = `type`, snake_case). */
export type JobKind =
  | { type: "merge"; inputs: string[]; output: string }
  | { type: "split"; source: string; ranges: [number, number][]; outputs: string[] }
  | { type: "analyze"; path: string }
  | { type: "ocr"; path: string; output_txt: string; lang?: string };

export interface AegisErrorShape {
  code: string;
  message: string;
  details?: unknown;
}

export interface PageAnalysis {
  page_index: number;
  is_blank: boolean;
  content_hash: string;
  duplicate_of: number | null;
}

export interface JobEventPayload {
  job_id: string;
  phase: string;
  progress: number;
  result?: unknown;
  error?: AegisErrorShape;
}
