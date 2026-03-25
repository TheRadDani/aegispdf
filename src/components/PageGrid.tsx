import { closestCenter, DndContext, type DragEndEvent } from "@dnd-kit/core";
import { arrayMove, SortableContext, rectSortingStrategy } from "@dnd-kit/sortable";
import type { MouseEvent } from "react";
import type { Annotation, PageInfo } from "../types";
import PageCard from "./PageCard";

interface PageGridProps {
  documentId: string;
  pages: PageInfo[];
  selected: Set<number>;
  zoom: number;
  annotations: Annotation[];
  onReorder: (pages: PageInfo[]) => Promise<void>;
  onSelect: (index: number, event: MouseEvent) => void;
}

export default function PageGrid({
  documentId,
  pages,
  selected,
  zoom,
  annotations,
  onReorder,
  onSelect
}: PageGridProps) {
  const onDragEnd = async (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over || active.id === over.id) {return;}
    const oldIndex = pages.findIndex((p) => p.index === active.id);
    const newIndex = pages.findIndex((p) => p.index === over.id);
    const moved = arrayMove(pages, oldIndex, newIndex).map((p, idx) => ({ ...p, index: idx }));
    await onReorder(moved);
  };

  return (
    <DndContext collisionDetection={closestCenter} onDragEnd={(event) => { void onDragEnd(event); }}>
      <SortableContext items={pages.map((p) => p.index)} strategy={rectSortingStrategy}>
        <section className="page-grid">
          {pages.map((page) => (
            <PageCard
              key={`${page.page_number}-${page.index}`}
              documentId={documentId}
              page={page}
              zoom={zoom}
              isSelected={selected.has(page.index)}
              annotations={annotations}
              onSelect={onSelect}
            />
          ))}
        </section>
      </SortableContext>
    </DndContext>
  );
}
