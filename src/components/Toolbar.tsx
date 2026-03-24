interface ToolbarProps {
  onOpen: () => void;
  onSave: () => void;
  onDelete: () => void;
  onMerge: () => void;
  onSplit: () => void;
  onCompress: () => void;
  onClean: () => void;
  onOcr: () => void;
  onAnalyze: () => void;
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
        {jobHint ? <span className="job-hint">{jobHint}</span> : null}
      </div>
      <div className="toolbar-actions">
        <button onClick={onOpen}>Open</button>
        <button onClick={onSave} disabled={!hasDocument}>
          Save
        </button>
        <button onClick={onMerge}>Merge</button>
        <button onClick={onSplit} disabled={!hasDocument}>
          Split
        </button>
        <button onClick={onDelete} disabled={!hasDocument}>
          Delete
        </button>
        <button onClick={onCompress} disabled={!hasDocument}>
          Compress
        </button>
        <button onClick={onClean} disabled={!hasDocument}>
          Clean
        </button>
        <button onClick={onOcr} disabled={!hasDocument}>
          OCR
        </button>
        <button onClick={onAnalyze} disabled={!hasDocument}>
          Analyze
        </button>
        <button onClick={onAnnotate} disabled={!hasDocument}>
          Annotate
        </button>
      </div>
    </header>
  );
}
