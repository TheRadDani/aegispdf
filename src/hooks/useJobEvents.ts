import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import type { JobEventPayload } from "../types";

export function useJobEvents() {
  const [lastEvent, setLastEvent] = useState<JobEventPayload | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    void listen<JobEventPayload>("aegis-job-event", (e) => {
      setLastEvent(e.payload);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      void unlisten?.();
    };
  }, []);

  return { lastEvent, clearLast: () => setLastEvent(null) };
}
