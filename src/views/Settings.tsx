import { useCallback, useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import {
  loadSettings,
  saveSettings,
  checkMediaPath,
  checkMusicPath,
  checkQuickConnect,
  checkApiConfigurationStatus,
  diagnosePath,
  scanMediaDirsProgressive,
  scanMusicDirsProgressive,
  fetchMetadataBatch,
  rdGetDeviceCode,
  rdPollCredentials,
  rdSaveApiKey,
  rdGetStatus,
} from "../api/commands";
import { useScan } from "../context/ScanContext";
import type {
  Settings,
  PathCheckResult,
  RdStatusInfo,
  ApiConfigurationStatus,
  ApiServiceStatus,
} from "../types";

function ApiStatusLine({ label, status }: { label: string; status: ApiServiceStatus }) {
  const badgeClass = status.connected
    ? "rd-status-badge connected"
    : status.configured
      ? "rd-status-badge expired"
      : "rd-status-badge";

  const badgeText = status.connected
    ? "Verbunden"
    : status.configured
      ? "Nicht erreichbar"
      : "Nicht konfiguriert";

  return (
    <div className="api-status-item">
      <span className={badgeClass}>{label}: {badgeText}</span>
      <p className="api-status-message">{status.message}</p>
    </div>
  );
}

function buildUncheckedApiStatus(s: Settings): ApiConfigurationStatus {
  const tmdbConfigured = !!(s["tmdb_api_key"] ?? "").trim();
  const perplexityConfigured = !!(s["perplexity_api_key"] ?? "").trim();

  return {
    tmdb: {
      configured: tmdbConfigured,
      connected: false,
      message: tmdbConfigured
        ? "Noch nicht geprüft. Auf \"TMDb-Verbindung prüfen\" klicken."
        : "Kein API-Key gesetzt.",
    },
    perplexity: {
      configured: perplexityConfigured,
      connected: false,
      message: perplexityConfigured
        ? "Noch nicht geprüft. Auf \"Perplexity-Verbindung prüfen\" klicken."
        : "Kein API-Key gesetzt.",
    },
  };
}

export default function SettingsView() {
  const { isScanning, setScanning, isFetchingMetadata, setFetchingMetadata } = useScan();
  const [settings, setSettings] = useState<Settings>({});
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [pathInput, setPathInput] = useState("");
  const [pathCheck, setPathCheck] = useState<PathCheckResult | null>(null);
  const [checkingPath, setCheckingPath] = useState(false);
  const [musicPathInput, setMusicPathInput] = useState("");
  const [musicPathCheck, setMusicPathCheck] = useState<PathCheckResult | null>(null);
  const [checkingMusicPath, setCheckingMusicPath] = useState(false);
  const [musicScanning, setMusicScanning] = useState(false);
  const [rdStatus, setRdStatus] = useState<RdStatusInfo | null>(null);
  const [rdCode, setRdCode] = useState<string | null>(null);
  const [rdUrl, setRdUrl] = useState<string>("");
  const [rdPolling, setRdPolling] = useState(false);
  const [rdApiKeyInput, setRdApiKeyInput] = useState("");
  const [apiStatus, setApiStatus] = useState<ApiConfigurationStatus | null>(null);
  const [checkingTmdbStatus, setCheckingTmdbStatus] = useState(false);
  const [checkingPerplexityStatus, setCheckingPerplexityStatus] = useState(false);
  const [qcStatus, setQcStatus] = useState<{
    connected: boolean;
    message: string;
    server_url: string | null;
  } | null>(null);
  const [checkingQc, setCheckingQc] = useState(false);
  const [diagnoseResult, setDiagnoseResult] = useState<{
    volumes: string[];
    path_checked: string;
    path_exists: boolean;
    path_is_dir: boolean;
  } | null>(null);

  const settingsRef = useRef(settings);
  settingsRef.current = settings;

  useEffect(() => {
    return () => {
      saveSettings(settingsRef.current).catch((err) =>
        console.error("Auto-save on leave failed:", err)
      );
    };
  }, []);

  const load = useCallback(async () => {
    try {
      setLoading(true);
      const [s, rdStatusRes] = await Promise.all([
        loadSettings(),
        rdGetStatus().catch(() => null),
      ]);
      setSettings(s);
      setRdStatus(rdStatusRes);
      setApiStatus(buildUncheckedApiStatus(s));
      const paths = parsePaths(s["media_paths"]);
      if (paths.length > 0 && !pathInput) setPathInput(paths[0]);
      const musicPaths = parsePaths(s["music_paths"]);
      if (musicPaths.length > 0 && !musicPathInput) setMusicPathInput(musicPaths[0]);
    } catch (err) {
      console.error("Failed to load settings:", err);
    } finally {
      setLoading(false);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps -- pathInput/musicPathInput excluded: only set on initial load when empty
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("music-scan-complete", () => setMusicScanning(false)).then(
      (fn) => { unlisten = fn; }
    );
    return () => { unlisten?.(); };
  }, []);

  function parsePaths(json: string | undefined): string[] {
    if (!json) return [];
    try {
      const arr = JSON.parse(json);
      return Array.isArray(arr) ? arr : [];
    } catch {
      return [];
    }
  }

  function getPaths(): string[] {
    return parsePaths(settings["media_paths"]);
  }

  function getMusicPaths(): string[] {
    return parsePaths(settings["music_paths"]);
  }

  async function handleSave() {
    try {
      setSaving(true);
      await saveSettings(settings);
    } catch (err) {
      console.error("Failed to save:", err);
    } finally {
      setSaving(false);
    }
  }

  async function handleAddPath() {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: "/Volumes",
    });
    if (selected && typeof selected === "string") {
      const paths = getPaths();
      if (!paths.includes(selected)) {
        paths.push(selected);
        setSettings({ ...settings, media_paths: JSON.stringify(paths) });
      }
    }
  }

  function handleRemovePath(p: string) {
    const paths = getPaths().filter((x) => x !== p);
    setSettings({ ...settings, media_paths: JSON.stringify(paths) });
  }

  function handleAddPathFromInput() {
    const p = pathInput.trim();
    if (!p) return;
    const paths = getPaths();
    if (!paths.includes(p)) {
      paths.push(p);
      setSettings({ ...settings, media_paths: JSON.stringify(paths) });
    }
  }

  async function handleAddMusicPath() {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: "/Volumes",
    });
    if (selected && typeof selected === "string") {
      const paths = getMusicPaths();
      if (!paths.includes(selected)) {
        paths.push(selected);
        setSettings({ ...settings, music_paths: JSON.stringify(paths) });
      }
    }
  }

  function handleRemoveMusicPath(p: string) {
    const paths = getMusicPaths().filter((x) => x !== p);
    setSettings({ ...settings, music_paths: JSON.stringify(paths) });
  }

  function handleAddMusicPathFromInput() {
    const p = musicPathInput.trim();
    if (!p) return;
    const paths = getMusicPaths();
    if (!paths.includes(p)) {
      paths.push(p);
      setSettings({ ...settings, music_paths: JSON.stringify(paths) });
    }
  }

  async function handleDiagnosePath(section: "media" | "music") {
    const path = section === "media"
      ? (pathInput.trim() || getPaths()[0])
      : (musicPathInput.trim() || getMusicPaths()[0]);
    if (!path) return;
    try {
      const result = await diagnosePath(path);
      setDiagnoseResult(result);
    } catch {
      setDiagnoseResult({
        volumes: [],
        path_checked: path,
        path_exists: false,
        path_is_dir: false,
      });
    }
  }

  async function handleCheckPath() {
    const path = pathInput.trim() || getPaths()[0];
    if (!path) return;
    setCheckingPath(true);
    setPathCheck(null);
    try {
      const timeout = new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error("Zeitüberschreitung (30 s). Pfad prüfen oder NAS-Verbindung prüfen.")), 30000)
      );
      const result = await Promise.race([checkMediaPath(path), timeout]);
      setPathCheck(result);
    } catch (err) {
      setPathCheck({
        path,
        exists: false,
        is_directory: false,
        files_found: 0,
        sample_files: [],
        sample_all: [],
        subdirs: [],
        error: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setCheckingPath(false);
    }
  }

  async function handleCheckMusicPath() {
    const path = musicPathInput.trim() || getMusicPaths()[0];
    if (!path) return;
    setCheckingMusicPath(true);
    setMusicPathCheck(null);
    try {
      const timeout = new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error("Zeitüberschreitung (30 s). Pfad prüfen oder NAS-Verbindung prüfen.")), 30000)
      );
      const result = await Promise.race([checkMusicPath(path), timeout]);
      setMusicPathCheck(result);
    } catch (err) {
      setMusicPathCheck({
        path,
        exists: false,
        is_directory: false,
        files_found: 0,
        sample_files: [],
        sample_all: [],
        subdirs: [],
        error: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setCheckingMusicPath(false);
    }
  }

  async function handleCheckQuickConnect() {
    const id = (settings["quickconnect_id"] ?? "").trim();
    if (!id) {
      setQcStatus({
        connected: false,
        message: "QuickConnect-ID eingeben.",
        server_url: null,
      });
      return;
    }
    try {
      setCheckingQc(true);
      setQcStatus(null);
      const result = await checkQuickConnect(id);
      setQcStatus(result);
    } catch (err) {
      setQcStatus({
        connected: false,
        message: `Prüfung fehlgeschlagen: ${err}`,
        server_url: null,
      });
    } finally {
      setCheckingQc(false);
    }
  }

  async function handleCheckTmdbStatus() {
    try {
      setCheckingTmdbStatus(true);
      const status = await checkApiConfigurationStatus(
        settings["tmdb_api_key"] ?? "",
        null
      );
      setApiStatus((prev) => ({
        ...(prev ?? buildUncheckedApiStatus(settings)),
        tmdb: status.tmdb,
      }));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setApiStatus((prev) => ({
        ...(prev ?? buildUncheckedApiStatus(settings)),
        tmdb: {
          configured: !!(settings["tmdb_api_key"] ?? "").trim(),
          connected: false,
          message: `Statusprüfung fehlgeschlagen: ${message}`,
        },
      }));
    } finally {
      setCheckingTmdbStatus(false);
    }
  }

  async function handleCheckPerplexityStatus() {
    try {
      setCheckingPerplexityStatus(true);
      const status = await checkApiConfigurationStatus(
        null,
        settings["perplexity_api_key"] ?? ""
      );
      setApiStatus((prev) => ({
        ...(prev ?? buildUncheckedApiStatus(settings)),
        perplexity: status.perplexity,
      }));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setApiStatus((prev) => ({
        ...(prev ?? buildUncheckedApiStatus(settings)),
        perplexity: {
          configured: !!(settings["perplexity_api_key"] ?? "").trim(),
          connected: false,
          message: `Statusprüfung fehlgeschlagen: ${message}`,
        },
      }));
    } finally {
      setCheckingPerplexityStatus(false);
    }
  }

  async function handleScanMusic() {
    const paths = getMusicPaths();
    if (paths.length === 0) return;
    try {
      setMusicScanning(true);
      setMusicPathCheck(null);
      await scanMusicDirsProgressive(paths);
    } catch (err) {
      console.error("Music scan failed:", err);
      setMusicScanning(false);
    }
  }

  async function handleScan() {
    const paths = getPaths();
    if (paths.length === 0) return;
    try {
      setScanning(true);
      setPathCheck(null);
      await scanMediaDirsProgressive(paths);
    } catch (err) {
      console.error("Scan failed:", err);
      setScanning(false);
    }
  }

  async function startRdAuth() {
    try {
      const { user_code, verification_url, device_code } =
        await rdGetDeviceCode();
      setRdCode(user_code);
      setRdUrl(verification_url);
      setRdPolling(true);
      const ok = await rdPollCredentials(device_code);
      setRdPolling(false);
      if (ok) {
        setRdCode(null);
        setRdUrl("");
        const status = await rdGetStatus();
        setRdStatus(status);
      }
    } catch (err) {
      console.error("RD auth failed:", err);
      setRdPolling(false);
    }
  }

  async function saveRdApiKey() {
    const key = rdApiKeyInput.trim();
    if (!key) return;
    try {
      await rdSaveApiKey(key);
      setRdApiKeyInput("");
      const status = await rdGetStatus();
      setRdStatus(status);
    } catch (err) {
      console.error("RD save failed:", err);
    }
  }

  if (loading) return <div className="loading-state">Lade Einstellungen...</div>;

  const paths = getPaths();

  return (
    <div className="view-settings">
      <h1>Einstellungen</h1>

      <section className="settings-section">
        <h2>Medienpfade</h2>
        <p className="settings-hint">
          Ordner mit Filmen und Serien. NAS z.B. unter{" "}
          <code>/Volumes/Diskstation/video</code>. Lokale Pfade werden bevorzugt;
          für Remote-Zugriff QuickConnect konfigurieren und Share einbinden.
        </p>
        {typeof navigator !== "undefined" && /Mac|Darwin/.test(navigator.platform) && (
          <p className="settings-hint settings-hint-macos">
            macOS: Wenn gemountete Pfade nicht gefunden werden, App unter{" "}
            <strong>Systemeinstellungen → Datenschutz &amp; Sicherheit → Vollständiger Festplattenzugriff</strong>{" "}
            hinzufügen und App neu starten.
          </p>
        )}
        <div className="path-list">
          {paths.map((p) => (
            <div key={p} className="path-item">
              <span className="path-text">{p}</span>
              <button
                type="button"
                className="btn-secondary btn-sm"
                onClick={() => handleRemovePath(p)}
              >
                Entfernen
              </button>
            </div>
          ))}
        </div>
        <div className="path-input-row">
          <input
            type="text"
            placeholder="/Pfad/zum/Ordner oder /Volumes/NAS/video"
            value={pathInput}
            onChange={(e) => setPathInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAddPathFromInput()}
          />
          <button
            type="button"
            className="btn-secondary"
            onClick={handleAddPathFromInput}
            disabled={!pathInput.trim()}
          >
            Hinzufügen
          </button>
          <button
            type="button"
            className="btn-secondary"
            onClick={handleAddPath}
          >
            Ordner wählen
          </button>
        </div>
        <div className="action-row">
          <button
            type="button"
            className="btn-secondary"
            onClick={handleCheckPath}
            disabled={checkingPath || (!pathInput.trim() && paths.length === 0)}
          >
            {checkingPath ? "Prüfe…" : "Prüfen"}
          </button>
          <button
            type="button"
            className="btn-secondary"
            onClick={() => handleDiagnosePath("media")}
            title="Zeigt, welche Volumes die App sieht"
          >
            Diagnose
          </button>
          <button
            type="button"
            className="btn-primary"
            onClick={handleScan}
            disabled={isScanning || paths.length === 0}
          >
            {isScanning ? "Scanne…" : "Bibliothek scannen"}
          </button>
        </div>
        {(checkingPath || pathCheck || diagnoseResult) && (
          <div className="scan-result">
            {checkingPath ? (
              <p>Prüfe Pfad…</p>
            ) : diagnoseResult ? (
              <>
                <p><strong>Diagnose</strong></p>
                <p>Volumes unter /Volumes: {diagnoseResult.volumes.join(", ") || "(keine)"}</p>
                <p>Geprüfter Pfad: {diagnoseResult.path_checked}</p>
                <p>Existiert: {diagnoseResult.path_exists ? "✓ Ja" : "✗ Nein"}</p>
                <p>Ist Ordner: {diagnoseResult.path_is_dir ? "✓ Ja" : "✗ Nein"}</p>
                <button type="button" className="btn-secondary btn-sm" onClick={() => setDiagnoseResult(null)}>
                  Schließen
                </button>
              </>
            ) : pathCheck?.error ? (
              <p className="status-warn">{pathCheck.error}</p>
            ) : pathCheck ? (
              <>
                <p>
                  {pathCheck.exists
                    ? `✓ Verzeichnis gefunden, ${pathCheck.files_found} Dateien`
                    : "✗ Pfad nicht gefunden"}
                </p>
                {pathCheck.subdirs.length > 0 && (
                  <p>Unterordner: {pathCheck.subdirs.slice(0, 5).join(", ")}…</p>
                )}
              </>
            ) : null}
          </div>
        )}
      </section>

      <section className="settings-section">
        <h2>Synology QuickConnect</h2>
        <p className="settings-hint">
          Für Remote-Zugriff auf Synology NAS. Lokale Pfade (z.B. /Volumes/Diskstation) werden
          bevorzugt. Ohne lokalen Zugriff wird QuickConnect verwendet.
        </p>
        <p className="settings-hint">
          Für Dateizugriff: NAS-Share im System mounten (SMB, Finder). Beim Mounten werden
          Benutzername und Passwort abgefragt – die App nutzt dann den gemounteten Pfad.
        </p>
        <div className="setting-row">
          <label>QuickConnect-ID</label>
          <input
            type="text"
            value={settings["quickconnect_id"] ?? ""}
            onChange={(e) =>
              setSettings({ ...settings, quickconnect_id: e.target.value })
            }
            placeholder="z.B. mynas"
          />
        </div>
        <div className="setting-row">
          <label>Lokaler Hostname (optional)</label>
          <input
            type="text"
            value={settings["quickconnect_local_host"] ?? ""}
            onChange={(e) =>
              setSettings({ ...settings, quickconnect_local_host: e.target.value })
            }
            placeholder="z.B. diskstation.local oder 192.168.1.100"
          />
        </div>
        <div className="action-row">
          <button
            type="button"
            className="btn-secondary"
            onClick={handleCheckQuickConnect}
            disabled={checkingQc}
          >
            {checkingQc ? "Prüfe…" : "Verbindung prüfen"}
          </button>
        </div>
        {qcStatus && (
          <div className={`qc-status ${qcStatus.connected ? "qc-connected" : "qc-disconnected"}`}>
            <span className={`qc-status-badge ${qcStatus.connected ? "connected" : ""}`}>
              {qcStatus.connected ? "Verbunden" : "Nicht erreichbar"}
            </span>
            <p className="qc-status-message">{qcStatus.message}</p>
            {qcStatus.server_url && (
              <p className="qc-status-url">
                <a
                  href={qcStatus.server_url}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  {qcStatus.server_url}
                </a>
              </p>
            )}
          </div>
        )}
      </section>

      <section className="settings-section">
        <h2>Musikpfade</h2>
        <p className="settings-hint">
          Ordner mit Musikdateien (MP3, FLAC, M4A, etc.). NAS z.B. unter{" "}
          <code>/Volumes/Diskstation/music</code> (auf macOS: großes <strong>V</strong> in Volumes).
        </p>
        {typeof navigator !== "undefined" && /Mac|Darwin/.test(navigator.platform) && (
          <p className="settings-hint settings-hint-macos">
            macOS: Wenn gemountete Pfade nicht gefunden werden, App unter{" "}
            <strong>Systemeinstellungen → Datenschutz &amp; Sicherheit → Vollständiger Festplattenzugriff</strong>{" "}
            hinzufügen und App neu starten.
          </p>
        )}
        <div className="path-list">
          {getMusicPaths().map((p) => (
            <div key={p} className="path-item">
              <span className="path-text">{p}</span>
              <button
                type="button"
                className="btn-secondary btn-sm"
                onClick={() => handleRemoveMusicPath(p)}
              >
                Entfernen
              </button>
            </div>
          ))}
        </div>
        <div className="path-input-row">
          <input
            type="text"
            placeholder="/Pfad/zum/Musik-Ordner oder /Volumes/NAS/music"
            value={musicPathInput}
            onChange={(e) => setMusicPathInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAddMusicPathFromInput()}
          />
          <button
            type="button"
            className="btn-secondary"
            onClick={handleAddMusicPathFromInput}
            disabled={!musicPathInput.trim()}
          >
            Hinzufügen
          </button>
          <button
            type="button"
            className="btn-secondary"
            onClick={handleAddMusicPath}
          >
            Ordner wählen
          </button>
        </div>
        <div className="action-row">
          <button
            type="button"
            className="btn-secondary"
            onClick={handleCheckMusicPath}
            disabled={checkingMusicPath || (!musicPathInput.trim() && getMusicPaths().length === 0)}
          >
            {checkingMusicPath ? "Prüfe…" : "Prüfen"}
          </button>
          <button
            type="button"
            className="btn-secondary"
            onClick={() => handleDiagnosePath("music")}
            title="Zeigt, welche Volumes die App sieht"
          >
            Diagnose
          </button>
          <button
            type="button"
            className="btn-primary"
            onClick={handleScanMusic}
            disabled={musicScanning || getMusicPaths().length === 0}
          >
            {musicScanning ? "Scanne…" : "Musikbibliothek scannen"}
          </button>
        </div>
        {(checkingMusicPath || musicPathCheck || diagnoseResult) && (
          <div className="scan-result">
            {checkingMusicPath ? (
              <p>Prüfe Pfad…</p>
            ) : diagnoseResult ? (
              <>
                <p><strong>Diagnose</strong></p>
                <p>Volumes unter /Volumes: {diagnoseResult.volumes.join(", ") || "(keine)"}</p>
                <p>Geprüfter Pfad: {diagnoseResult.path_checked}</p>
                <p>Existiert: {diagnoseResult.path_exists ? "✓ Ja" : "✗ Nein"}</p>
                <p>Ist Ordner: {diagnoseResult.path_is_dir ? "✓ Ja" : "✗ Nein"}</p>
                <button type="button" className="btn-secondary btn-sm" onClick={() => setDiagnoseResult(null)}>
                  Schließen
                </button>
              </>
            ) : musicPathCheck?.error ? (
              <p className="status-warn">{musicPathCheck.error}</p>
            ) : musicPathCheck ? (
              <>
                <p>
                  {musicPathCheck.exists
                    ? `✓ Verzeichnis gefunden, ${musicPathCheck.files_found} Audiodateien`
                    : "✗ Pfad nicht gefunden"}
                </p>
                {musicPathCheck.subdirs.length > 0 && (
                  <p>Unterordner: {musicPathCheck.subdirs.slice(0, 5).join(", ")}…</p>
                )}
              </>
            ) : null}
          </div>
        )}
      </section>

      <section className="settings-section">
        <h2>TMDb API</h2>
        <p className="settings-hint">
          Kostenloser Key von{" "}
          <a
            href="https://www.themoviedb.org/settings/api"
            target="_blank"
            rel="noopener noreferrer"
          >
            themoviedb.org
          </a>{" "}
          – für Discover und Metadaten.
        </p>
        <div className="setting-row">
          <label>TMDb API-Key</label>
          <input
            type="password"
            value={settings["tmdb_api_key"] ?? ""}
            onChange={(e) =>
              setSettings({ ...settings, tmdb_api_key: e.target.value })
            }
            placeholder="API-Key"
          />
        </div>
        <div className="action-row" style={{ marginTop: 12 }}>
          <button
            type="button"
            className="btn-secondary"
            onClick={handleCheckTmdbStatus}
            disabled={checkingTmdbStatus}
          >
            {checkingTmdbStatus ? "Prüfe…" : "TMDb-Verbindung prüfen"}
          </button>
          <button
            type="button"
            className="btn-secondary"
            onClick={async () => {
              try {
                setFetchingMetadata(true);
                const n = await fetchMetadataBatch();
                alert(`${n} Metadaten geladen.`);
              } catch {
                alert("Metadaten laden fehlgeschlagen. TMDb-API-Key gesetzt?");
              } finally {
                setFetchingMetadata(false);
              }
            }}
            disabled={isFetchingMetadata || !(settings["tmdb_api_key"] ?? "").trim()}
          >
            {isFetchingMetadata ? "Lade…" : "Metadaten für Bibliothek laden"}
          </button>
        </div>
        {apiStatus && (
          <div className="api-status-panel">
            <ApiStatusLine label="TMDb" status={apiStatus.tmdb} />
          </div>
        )}
      </section>

      <section className="settings-section">
        <h2>Lieblingsfilme (für „My taste“)</h2>
        <p className="settings-hint">
          Kommagetrennte Liste deiner Lieblingsfilme – für personalisierte
          Empfehlungen in „Empfehlungen für dich“.
        </p>
        <div className="setting-row">
          <label>Lieblingsfilme</label>
          <input
            type="text"
            value={settings["favorite_films"] ?? ""}
            onChange={(e) =>
              setSettings({ ...settings, favorite_films: e.target.value })
            }
            placeholder="z.B. Inception, Matrix, Pulp Fiction"
          />
        </div>
      </section>

      <section className="settings-section">
        <h2>Perplexity AI (Serien vs. Film)</h2>
        <p className="settings-hint">
          API-Key von{" "}
          <a
            href="https://www.perplexity.ai/settings/api"
            target="_blank"
            rel="noopener noreferrer"
          >
            perplexity.ai
          </a>{" "}
          – verbessert die Trennung von Serien/Filmen und das Clustern von Folgen.
        </p>
        <div className="setting-row">
          <label>Perplexity API-Key</label>
          <input
            type="password"
            value={settings["perplexity_api_key"] ?? ""}
            onChange={(e) =>
              setSettings({ ...settings, perplexity_api_key: e.target.value })
            }
            placeholder="API-Key"
          />
        </div>
        <div className="action-row" style={{ marginTop: 12 }}>
          <button
            type="button"
            className="btn-secondary"
            onClick={handleCheckPerplexityStatus}
            disabled={checkingPerplexityStatus}
          >
            {checkingPerplexityStatus ? "Prüfe…" : "Perplexity-Verbindung prüfen"}
          </button>
        </div>
        {apiStatus && (
          <div className="api-status-panel">
            <ApiStatusLine label="Perplexity" status={apiStatus.perplexity} />
          </div>
        )}
      </section>

      <section className="settings-section">
        <h2>Discover-Region</h2>
        <div className="setting-row">
          <label>Ländercode (z.B. DE, AT)</label>
          <input
            type="text"
            value={settings["discover_country"] ?? "DE"}
            onChange={(e) =>
              setSettings({ ...settings, discover_country: e.target.value })
            }
            placeholder="DE"
            maxLength={2}
          />
        </div>
      </section>

      <section className="settings-section">
        <h2>RealDebrid</h2>
        <p className="settings-hint">
          Für Streams aus Torrentio/YTS. API-Key oder Geräte-Authentifizierung.
        </p>
        {rdStatus?.user ? (
          <div className="rd-connected-info">
            <span className={`rd-status-badge connected`}>Verbunden</span>
            <p className="rd-user-details">
              {rdStatus.user.username} · {rdStatus.user.email}
            </p>
          </div>
        ) : rdCode ? (
          <div className="rd-auth-flow">
            <p>Öffne die URL und gib den Code ein:</p>
            <p>
              <a href={rdUrl} target="_blank" rel="noopener noreferrer">
                {rdUrl}
              </a>
            </p>
            <p className="user-code">{rdCode}</p>
            {rdPolling && <p className="auth-waiting">Warte auf Bestätigung…</p>}
          </div>
        ) : (
          <>
            <div className="setting-row">
              <label>RealDebrid API-Key (alternativ)</label>
              <input
                type="password"
                value={rdApiKeyInput}
                onChange={(e) => setRdApiKeyInput(e.target.value)}
                placeholder="API-Key von real-debrid.com"
              />
            </div>
            <div className="action-row">
              <button
                type="button"
                className="btn-primary"
                onClick={saveRdApiKey}
                disabled={!rdApiKeyInput.trim()}
              >
                API-Key verbinden
              </button>
              <button
                type="button"
                className="btn-secondary"
                onClick={startRdAuth}
                disabled={rdPolling}
              >
                Mit Gerät verbinden
              </button>
            </div>
          </>
        )}
      </section>

      <section className="settings-section">
        <h2>Debridio</h2>
        <p className="settings-hint">
          Addon-URL von{" "}
          <a
            href="https://debridio.com/addons"
            target="_blank"
            rel="noopener noreferrer"
          >
            debridio.com
          </a>
          – nach Konfiguration „Install URL kopieren“.
        </p>
        <div className="setting-row">
          <label>Debridio Addon-URL</label>
          <input
            type="url"
            value={settings["debridio_url"] ?? ""}
            onChange={(e) =>
              setSettings({ ...settings, debridio_url: e.target.value })
            }
            placeholder="https://..."
          />
        </div>
      </section>

      <section className="settings-section">
        <h2>Wiedergabe</h2>
        <div className="setting-row">
          <label>Bevorzugter Player</label>
          <select
            value={settings["preferred_player"] ?? ""}
            onChange={(e) =>
              setSettings({ ...settings, preferred_player: e.target.value })
            }
          >
            <option value="">System-Standard</option>
            <option value="vlc">VLC</option>
            <option value="iina">IINA</option>
            <option value="html5">HTML5 (eingebettet)</option>
          </select>
        </div>
      </section>

      <div className="settings-save-row">
        <button
          type="button"
          className="btn-primary"
          onClick={handleSave}
          disabled={saving}
        >
          {saving ? "Speichere…" : "Speichern"}
        </button>
      </div>
    </div>
  );
}
