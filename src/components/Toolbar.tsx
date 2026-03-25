interface ToolbarProps {
  onOpen: () => Promise<void>;
  onSave: () => Promise<void>;
  onDelete: () => Promise<void>;
  onMerge: () => Promise<void>;
  onSplit: () => Promise<void>;
  onCompress: () => Promise<void>;
  onClean: () => Promise<void>;
  onOcr: () => Promise<void>;
  onAnalyze: () => Promise<void>;
  onAnnotate: () => void;
  hasDocument: boolean;
  jobHint?: string;
}

export default function Toolbar({
  onOpen,
  onSave,
  onDelete,
  onMerge,
  onSplit,
  onCompress,
  onClean,
  onOcr,
  onAnalyze,
  onAnnotate,
  hasDocument,
  jobHint
}: ToolbarProps) {
  return (
    <header className="toolbar">
      <div className="toolbar-brand">
        <h1>AegisPDF</h1>
        {jobHint !== undefined && jobHint.length > 0 ? <span className="job-hint">{jobHint}</span> : null}
      </div>
      <div className="toolbar-actions">
        <button onClick={() => { void onOpen(); }}>Open</button>
        <button onClick={() => { void onSave(); }} disabled={!hasDocument}>
          Save
        </button>
        <button onClick={() => { void onMerge(); }}>Merge</button>
        <button onClick={() => { void onSplit(); }} disabled={!hasDocument}>
          Split
        </button>
        <button onClick={() => { void onDelete(); }} disabled={!hasDocument}>
          Delete
        </button>
        <button onClick={() => { void onCompress(); }} disabled={!hasDocument}>
          Compress
        </button>
        <button onClick={() => { void onClean(); }} disabled={!hasDocument}>
          Clean
        </button>
        <button onClick={() => { void onOcr(); }} disabled={!hasDocument}>
          OCR
        </button>
        <button onClick={() => { void onAnalyze(); }} disabled={!hasDocument}>
          Analyze
        </button>
        <button onClick={onAnnotate} disabled={!hasDocument}>
          Annotate
        </button>
      </div>
    </header>
  );
}
