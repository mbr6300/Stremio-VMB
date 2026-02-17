import { createContext, useContext, useState, useCallback, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { fetchMetadataBatch } from "../api/commands";

type ScanContextType = {
  isScanning: boolean;
  isFetchingMetadata: boolean;
  setScanning: (v: boolean) => void;
  setFetchingMetadata: (v: boolean) => void;
};

const ScanContext = createContext<ScanContextType | null>(null);

export function ScanProvider({ children }: { children: React.ReactNode }) {
  const [isScanning, setScanning] = useState(false);
  const [isFetchingMetadata, setFetchingMetadata] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("library-scan-complete", async () => {
      setScanning(false);
      try {
        setFetchingMetadata(true);
        await fetchMetadataBatch();
      } catch (err) {
        console.warn("Metadaten-Batch abgebrochen (z.B. kein TMDb-API-Key):", err);
      } finally {
        setFetchingMetadata(false);
      }
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  const setScanningStable = useCallback((v: boolean) => setScanning(v), []);
  const setFetchingMetadataStable = useCallback((v: boolean) => setFetchingMetadata(v), []);

  return (
    <ScanContext.Provider value={{ isScanning, isFetchingMetadata, setScanning: setScanningStable, setFetchingMetadata: setFetchingMetadataStable }}>
      {children}
    </ScanContext.Provider>
  );
}

export function useScan() {
  const ctx = useContext(ScanContext);
  if (!ctx) throw new Error("useScan must be used within ScanProvider");
  return ctx;
}
