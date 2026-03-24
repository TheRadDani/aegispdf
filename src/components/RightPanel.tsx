import type { PageAnalysis } from "../types";

interface RightPanelProps {
  selectedCount: number;
  pageCount: number;
  analysis: PageAnalysis[] | null;
  onAddHighlight: () => void;
  onAddTextNote: () => void;
  onSaveAegis: () => void;
}

export default function RightPanel({
  selectedCount,
  pageCount,
  analysis,
  onAddHighlight,
  onAddTextNote,
  onSaveAegis
}: RightPanelProps) {
  const blanks = analysis?.filter((a) => a.is_blank) ?? [];
  const dups = analysis?.filter((a) => a.duplicate_of != null) ?? [];

  return (
    <aside className="right-panel">
      <h3>Page Details</h3>
      <p>Total Pages: {pageCount}</p>
      <p>Selected: {selectedCount}</p>

      <h3>Intelligence</h3>
      {!analysis ? (
        <p className="muted">Run Analyze to detect blank / duplicate pages.</p>
      ) : (
        <>
          <p className="muted">Blank-like: {blanks.length}</p>
          <ul className="analysis-list">
            {blanks.slice(0, 8).map((b) => (
              <li key={b.page_index}>Page {b.page_index + 1}</li>
            ))}
          </ul>
          <p className="muted">Duplicate groups: {dups.length}</p>
          <ul className="analysis-list">
            {dups.slice(0, 8).map((d) => (
              <li key={d.page_index}>
                Page {d.page_index + 1} → same as {((d.duplicate_of ?? 0) as number) + 1}
              </li>
            ))}
          </ul>
        </>
      )}

      <h3>Annotations (.aegis)</h3>
      <p className="muted">Non-destructive overlay; original PDF unchanged until export.</p>
      <button type="button" onClick={onAddHighlight}>
        Highlight selection
      </button>
      <button type="button" onClick={onAddTextNote}>
        Text note
      </button>
      <button type="button" onClick={onSaveAegis}>
        Save .aegis
      </button>
    </aside>
  );
}
