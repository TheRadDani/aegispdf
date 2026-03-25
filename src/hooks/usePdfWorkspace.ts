import { useCallback, useMemo, useState } from "react";
import {
  autoCleanWorkspace as autoCleanWorkspaceApi,
  compressWorkspace as compressWorkspaceApi,
  deletePages as deletePagesApi,
  loadAegis as loadAegisApi,
  openPdf as openPdfApi,
  reorderPages as reorderPagesApi,
  saveAegis as saveAegisApi,
  savePdf as savePdfApi
} from "../services/api";
import type { Annotation, AnnotationStore, OpenPdfResponse, PageInfo } from "../types";

export function usePdfWorkspace() {
  const [doc, setDoc] = useState<OpenPdfResponse | null>(null);
  const [pages, setPages] = useState<PageInfo[]>([]);
  const [selected, setSelected] = useState(new Set<number>());
  const [annotations, setAnnotations] = useState<Annotation[]>([]);

  const openPdf = useCallback(async (path: string) => {
    const next = await openPdfApi(path);
    setDoc(next);
    setPages(next.page_infos);
    setSelected(new Set());
    const store = await loadAegisApi(next.source_path, next.file_hash);
    setAnnotations(store.annotations);
  }, []);

  const saveAnnotations = useCallback(async () => {
    if (!doc) {return;}
    const store: AnnotationStore = { pdf_hash: doc.file_hash, annotations };
    await saveAegisApi(doc.source_path, store);
  }, [doc, annotations]);

  const addAnnotation = useCallback((a: Annotation) => {
    setAnnotations((prev) => [...prev, a]);
  }, []);

  const reorder = useCallback(
    async (newPages: PageInfo[]) => {
      if (!doc) {return;}
      setPages(newPages);
      const newOrder = newPages.map((p) => p.page_number);
      await reorderPagesApi(doc.document_id, newOrder);
    },
    [doc]
  );

  const deleteSelected = useCallback(async () => {
    if (!doc || selected.size === 0) {return;}
    const indices = Array.from(selected.values()).sort((a, b) => b - a);
    const nextPages = await deletePagesApi(doc.document_id, indices);
    setPages(nextPages);
    const sortedDel = [...indices].sort((a, b) => a - b);
    setAnnotations((prev) =>
      prev
        .filter((a) => !sortedDel.includes(a.page_index))
        .map((a) => {
          const removedBefore = sortedDel.filter((i) => i < a.page_index).length;
          return { ...a, page_index: a.page_index - removedBefore };
        })
    );
    setSelected(new Set());
  }, [doc, selected]);

  const save = useCallback(
    async (outputPath: string) => {
      if (!doc) {return;}
      await savePdfApi(doc.document_id, outputPath);
    },
    [doc]
  );

  const compress = useCallback(
    async (roundtrip: boolean) => {
      if (!doc) {return;}
      await compressWorkspaceApi(doc.document_id, roundtrip);
    },
    [doc]
  );

  const autoClean = useCallback(
    async (stripAnnots: boolean) => {
      if (!doc) {return;}
      await autoCleanWorkspaceApi(doc.document_id, stripAnnots);
    },
    [doc]
  );

  const pageCount = useMemo(() => pages.length, [pages]);

  return {
    doc,
    pages,
    selected,
    setSelected,
    pageCount,
    annotations,
    setAnnotations,
    addAnnotation,
    saveAnnotations,
    openPdf,
    reorder,
    deleteSelected,
    save,
    compress,
    autoClean
  };
}
