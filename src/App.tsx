import { type MouseEvent, useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import PageGrid from "./components/PageGrid";
import RightPanel from "./components/RightPanel";
import Toolbar from "./components/Toolbar";
import { useJobEvents } from "./hooks/useJobEvents";
import { usePdfWorkspace } from "./hooks/usePdfWorkspace";
import {
  exportPdfTemp,
  mergePdfPaths,
  splitPdfEachPage,
  submitJob
} from "./services/api";
import type { PageAnalysis } from "./types";

export default function App() {
  const [zoom, setZoom] = useState(1);
  const [analysis, setAnalysis] = useState<PageAnalysis[] | null>(null);
  const [statusMsg, setStatusMsg] = useState("");
  const { lastEvent } = useJobEvents();
  const {
    doc,
    pages,
    selected,
    setSelected,
    pageCount,
    annotations,
    addAnnotation,
    saveAnnotations,
    openPdf,
    reorder,
    deleteSelected,
    save: savePdf,
    compress,
    autoClean
  } = usePdfWorkspace();

  const selectedCount = useMemo(() => selected.size, [selected]);

  // ── File-association handler ──────────────────────────────────────────────
  // When AegisPDF is launched from the OS (double-click .pdf / .aegis, or
  // "Open with AegisPDF"), the Rust backend emits this event with the path.
  const handleFileOpen = useCallback(
    (path: string) => {
      void openPdf(path).then(() => setAnalysis(null));
    },
    [openPdf]
  );

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<string>("aegis://open-file", (e) => {
      handleFileOpen(e.payload);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [handleFileOpen]);

  const jobHint = useMemo(() => {
    if (!lastEvent) {return "";}
    if (lastEvent.phase === "error" && lastEvent.error) {return `Job error: ${lastEvent.error.message}`;}
    if (lastEvent.phase === "complete" && lastEvent.result !== null && lastEvent.result !== undefined && typeof lastEvent.result === "object") {
      const r = lastEvent.result as Record<string, unknown>;
      if (typeof r.output_txt === "string") {return "OCR job finished";}
      if (Array.isArray(lastEvent.result)) {return "Analyze job finished";}
      if (typeof r.output === "string") {return "Merge job finished";}
      if (Array.isArray(r.outputs)) {return "Split job finished";}
    }
    if (lastEvent.phase === "running") {return `Job ${Math.round(lastEvent.progress * 100)}%`;}
    if (lastEvent.phase === "started") {return "Job started…";}
    return "";
  }, [lastEvent]);

  useEffect(() => {
    if (lastEvent?.phase !== "complete" || lastEvent.result === null || lastEvent.result === undefined) {return;}
    if (Array.isArray(lastEvent.result)) {
      const rows = lastEvent.result as PageAnalysis[];
      if (rows.length && typeof rows[0]?.page_index === "number") {
        setAnalysis(rows);
      }
    }
  }, [lastEvent]);

  const onOpen = async () => {
    const selectedPath = await open({
      multiple: false,
      filters: [{ name: "PDF", extensions: ["pdf"] }]
    });
    if (typeof selectedPath === "string") {
      await openPdf(selectedPath);
      setAnalysis(null);
    }
  };

  const onSave = async () => {
    if (!doc) {return;}
    const outputPath = await save({
      filters: [{ name: "PDF", extensions: ["pdf"] }],
      defaultPath: "aegispdf-output.pdf"
    });
    if (typeof outputPath === "string") {
      try {
        setStatusMsg("Saving…");
        await savePdf(outputPath);
        setStatusMsg("Saved successfully");
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        setStatusMsg(`Save failed: ${msg}`);
      }
    }
  };

  const onMerge = async () => {
    // If a document is already open, the user only needs to pick the PDFs
    // to merge INTO it. Otherwise, they select all files at once.
    const dialogResult = await open({
      multiple: true,
      title: doc
        ? "Select PDF(s) to merge into the current document"
        : "Select PDFs to merge (2 or more)",
      filters: [{ name: "PDF", extensions: ["pdf"] }]
    });

    // Normalize the dialog result to a string array.
    // Tauri's open() returns: null (cancelled) | string (one file) | string[] (multiple).
    let picked: string[];
    if (dialogResult === null) {
      return;
    } else if (typeof dialogResult === "string") {
      picked = [dialogResult];
    } else if (Array.isArray(dialogResult)) {
      picked = dialogResult;
    } else {
      return;
    }

    // Build the full list of inputs: current document first (if open), then picked files.
    let inputs: string[];
    if (doc) {
      // Export the current workspace state to a temp file so it includes
      // any reorders/deletes the user already made in this session.
      const currentPdf = await exportPdfTemp(doc.document_id);
      inputs = [currentPdf, ...picked];
    } else {
      inputs = picked;
    }

    if (inputs.length < 2) {
      setStatusMsg("Merge requires at least 2 PDFs.");
      return;
    }

    const outputPath = await save({
      filters: [{ name: "PDF", extensions: ["pdf"] }],
      defaultPath: "merged.pdf"
    });
    if (typeof outputPath !== "string") {return;}

    try {
      setStatusMsg(`Merging ${String(inputs.length)} PDFs…`);
      await mergePdfPaths(inputs, outputPath);
      setStatusMsg("Merge complete — opening merged file…");
      // Automatically open the merged result so the user sees it immediately.
      await openPdf(outputPath);
      setAnalysis(null);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setStatusMsg(`Merge failed: ${msg}`);
    }
  };

  const onSplit = async () => {
    if (!doc) {return;}
    const dir = await open({ directory: true, title: "Output folder for split pages" });
    if (typeof dir !== "string") {return;}
    try {
      setStatusMsg("Splitting…");
      const tmp = await exportPdfTemp(doc.document_id);
      await splitPdfEachPage(tmp, dir);
      setStatusMsg("Split complete");
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setStatusMsg(`Split failed: ${msg}`);
    }
  };

  const onCompress = async () => {
    try {
      setStatusMsg("Compressing…");
      await compress(true);
      setStatusMsg("Compression complete");
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setStatusMsg(`Compress failed: ${msg}`);
    }
  };

  const onClean = async () => {
    const strip = window.confirm("Also strip embedded PDF annotations from the working copy?");
    try {
      setStatusMsg("Cleaning…");
      await autoClean(strip);
      setStatusMsg("Clean complete");
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setStatusMsg(`Clean failed: ${msg}`);
    }
  };

  const onOcr = async () => {
    if (!doc) {return;}
    const tmp = await exportPdfTemp(doc.document_id);
    const txtPath = await save({
      filters: [{ name: "Text", extensions: ["txt"] }],
      defaultPath: "aegispdf-ocr.txt"
    });
    if (typeof txtPath !== "string") {return;}
    await submitJob({ type: "ocr", path: tmp, output_txt: txtPath, lang: "eng" });
  };

  const onAnalyze = async () => {
    if (!doc) {return;}
    const tmp = await exportPdfTemp(doc.document_id);
    await submitJob({ type: "analyze", path: tmp });
  };

  const onAnnotateToolbar = () => {
    document.querySelector(".right-panel")?.scrollIntoView({ behavior: "smooth" });
  };

  const onAddHighlight = () => {
    if (selected.size === 0) {
      window.alert("Select at least one page thumbnail first.");
      return;
    }
    const idx = Math.min(...Array.from(selected));
    addAnnotation({
      id: crypto.randomUUID(),
      page_index: idx,
      annotation_type: "highlight",
      payload: { x: 0.05, y: 0.06, w: 0.88, h: 0.12, color: "rgba(255, 220, 0, 0.38)" }
    });
  };

  const onAddTextNote = () => {
    if (selected.size === 0) {
      window.alert("Select a page first.");
      return;
    }
    const text = window.prompt("Note text");
    if (text === null || text.trim().length === 0) {return;}
    const idx = Math.min(...Array.from(selected));
    addAnnotation({
      id: crypto.randomUUID(),
      page_index: idx,
      annotation_type: "text_note",
      payload: { x: 0.05, y: 0.2, text }
    });
  };

  const onSelect = (index: number, event: MouseEvent) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (event.ctrlKey || event.metaKey) {
        if (next.has(index)) {next.delete(index);}
        else {next.add(index);}
        return next;
      }
      if (event.shiftKey) {
        const sorted = Array.from(next.values()).sort((a, b) => a - b);
        const anchor = sorted.length ? sorted[sorted.length - 1] : index;
        const [min, max] = anchor < index ? [anchor, index] : [index, anchor];
        for (let i = min; i <= max; i += 1) {next.add(i);}
        return next;
      }
      return new Set([index]);
    });
  };

  const displayHint = statusMsg || jobHint;

  return (
    <main className="app-shell">
      <Toolbar
        onOpen={onOpen}
        onSave={onSave}
        onDelete={deleteSelected}
        onMerge={onMerge}
        onSplit={onSplit}
        onCompress={onCompress}
        onClean={onClean}
        onOcr={onOcr}
        onAnalyze={onAnalyze}
        onAnnotate={onAnnotateToolbar}
        hasDocument={!!doc}
        jobHint={displayHint}
      />
      <div className="content">
        <section className="workspace">
          <div className="zoom-controls">
            <button type="button" onClick={() => setZoom((z) => Math.max(0.5, z - 0.1))}>
              −
            </button>
            <span>{Math.round(zoom * 100)}%</span>
            <button type="button" onClick={() => setZoom((z) => Math.min(2, z + 0.1))}>
              +
            </button>
          </div>
          {doc ? (
            <PageGrid
              documentId={doc.document_id}
              pages={pages}
              selected={selected}
              zoom={zoom}
              annotations={annotations}
              onReorder={reorder}
              onSelect={onSelect}
            />
          ) : (
            <div className="empty-state">Open a PDF to start your workspace.</div>
          )}
        </section>
        <RightPanel
          selectedCount={selectedCount}
          pageCount={pageCount}
          analysis={analysis}
          onAddHighlight={onAddHighlight}
          onAddTextNote={onAddTextNote}
          onSaveAegis={saveAnnotations}
        />
      </div>
    </main>
  );
}
