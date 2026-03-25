import { CSS } from "@dnd-kit/utilities";
import { type MouseEvent, useEffect, useMemo, useState } from "react";
import { useSortable } from "@dnd-kit/sortable";
import { getPageThumbnail } from "../services/api";
import type { Annotation, PageInfo } from "../types";

interface PageCardProps {
  documentId: string;
  page: PageInfo;
  zoom: number;
  isSelected: boolean;
  annotations: Annotation[];
  onSelect: (index: number, event: MouseEvent) => void;
}

function HighlightOverlay({ payload }: { payload: Record<string, unknown> }) {
  const x = typeof payload.x === "number" ? payload.x : 0;
  const y = typeof payload.y === "number" ? payload.y : 0;
  const w = typeof payload.w === "number" ? payload.w : 0.5;
  const h = typeof payload.h === "number" ? payload.h : 0.1;
  const color = typeof payload.color === "string" ? payload.color : "rgba(255, 220, 0, 0.35)";
  return (
    <div
      className="ann-highlight"
      style={{
        left: `${x * 100}%`,
        top: `${y * 100}%`,
        width: `${w * 100}%`,
        height: `${h * 100}%`,
        background: color
      }}
    />
  );
}

export default function PageCard({ documentId, page, zoom, isSelected, annotations, onSelect }: PageCardProps) {
  const [thumb, setThumb] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const hasThumbnail = thumb.length > 0;
  const hasError = error !== null && error.length > 0;
  const { attributes, listeners, setNodeRef, transform, transition } = useSortable({
    id: page.index
  });

  const pageAnnotations = useMemo(
    () => annotations.filter((a) => a.page_index === page.index),
    [annotations, page.index]
  );

  useEffect(() => {
    let mounted = true;
    setLoading(true);
    setThumb("");
    setError(null);
    void getPageThumbnail(documentId, page.index, zoom)
      .then((data) => {
        if (mounted) {setThumb(data);}
      })
      .catch((err: unknown) => {
        if (mounted) {
          const msg = err instanceof Error ? err.message : String(err);
          setError(msg);
          console.error(`Thumbnail failed for page ${page.page_number}:`, msg);
        }
      })
      .finally(() => {
        if (mounted) {setLoading(false);}
      });
    return () => {
      mounted = false;
    };
  }, [documentId, page.index, page.page_number, zoom]);

  return (
    <article
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className={`page-card ${isSelected ? "selected" : ""}`}
      onClick={(e) => onSelect(page.index, e)}
      {...attributes}
      {...listeners}
    >
      <div className="page-thumb">
        {loading ? (
          <span>Loading...</span>
        ) : hasError ? (
          <span className="page-error" title={error}>Preview error</span>
        ) : hasThumbnail ? (
          <>
            <img src={thumb} alt={`Page ${page.page_number}`} draggable={false} />
            <div className="ann-overlay" aria-hidden>
              {pageAnnotations.map((a) => {
                if (a.annotation_type === "highlight") {
                  return <HighlightOverlay key={a.id} payload={a.payload} />;
                }
                if (a.annotation_type === "text_note" && typeof a.payload.text === "string") {
                  const x = typeof a.payload.x === "number" ? a.payload.x : 0.05;
                  return (
                    <div key={a.id} className="ann-text-note" style={{ left: `${x * 100}%` }}>
                      {a.payload.text}
                    </div>
                  );
                }
                return null;
              })}
            </div>
          </>
        ) : (
          <span>No preview</span>
        )}
      </div>
      <div className="page-meta">Page {page.page_number}</div>
    </article>
  );
}
