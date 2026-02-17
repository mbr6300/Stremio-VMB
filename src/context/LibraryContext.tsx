import { createContext, useContext, useState, useCallback, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getLibraryWithMetadata } from "../api/commands";
import type { LibraryItemWithMeta, MediaItem, MediaMetadata } from "../types";

type MetadataFetchedPayload = [MediaItem, MediaMetadata];

type LibraryContextType = {
  items: LibraryItemWithMeta[];
  loading: boolean;
  reload: () => Promise<void>;
};

const LibraryContext = createContext<LibraryContextType | null>(null);

export function LibraryProvider({ children }: { children: React.ReactNode }) {
  const [items, setItems] = useState<LibraryItemWithMeta[]>([]);
  const [loading, setLoading] = useState(true);

  const loadLibrary = useCallback(async () => {
    try {
      setLoading(true);
      const data = await getLibraryWithMetadata();
      setItems(data);
    } catch (err) {
      console.error("Failed to load library:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadLibrary();
  }, [loadLibrary]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<MediaItem>("library-item-added", (ev) => {
      const item = ev.payload;
      setItems((prev) => {
        const idx = prev.findIndex((x) => x.item.file_path === item.file_path);
        const newEntry: LibraryItemWithMeta = { item, metadata: null };
        if (idx >= 0) {
          const next = [...prev];
          next[idx] = newEntry;
          return next;
        }
        return [...prev, newEntry];
      });
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<MetadataFetchedPayload>("metadata-fetched", (ev) => {
      const [item, metadata] = ev.payload;
      setItems((prev) => {
        const idx = prev.findIndex((x) => x.item.id === item.id);
        if (idx >= 0) {
          const next = [...prev];
          next[idx] = { item: next[idx].item, metadata };
          return next;
        }
        return prev;
      });
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("metadata-batch-complete", () => {
      loadLibrary();
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, [loadLibrary]);

  return (
    <LibraryContext.Provider value={{ items, loading, reload: loadLibrary }}>
      {children}
    </LibraryContext.Provider>
  );
}

export function useLibrary() {
  const ctx = useContext(LibraryContext);
  if (!ctx) throw new Error("useLibrary must be used within LibraryProvider");
  return ctx;
}
