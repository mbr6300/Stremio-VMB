import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  loadSettings,
  saveSettings,
  checkMediaPath,
  scanMediaDirsProgressive,
  fetchMetadataBatch,
  rdGetDeviceCode,
  rdPollCredentials,
  rdSaveApiKey,
  rdGetStatus,
} from "../api/commands";
import { useScan } from "../context/ScanContext";
import type { Settings, PathCheckResult, RdStatusInfo } from "../types";

export default function SettingsView() {
  const { isScanning, setScanning, isFetchingMetadata, setFetchingMetadata } = useScan();
  const [settings, setSettings] = useState<Settings>({});
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [pathInput, setPathInput] = useState("");
  const [pathCheck, setPathCheck] = useState<PathCheckResult | null>(null);
  const [checkingPath, setCheckingPath] = useState(false);
  const [rdStatus, setRdStatus] = useState<RdStatusInfo | null>(null);
  const [rdCode, setRdCode] = useState<string | null>(null);
  const [rdUrl, setRdUrl] = useState<string>("");
  const [rdPolling, setRdPolling] = useState(false);
  const [rdApiKeyInput, setRdApiKeyInput] = useState("");

  useEffect(() => {
    load();
  }, []);

  async function load() {
    try {
      setLoading(true);
      const [s, rdStatusRes] = await Promise.all([
        loadSettings(),
        rdGetStatus().catch(() => null),
      ]);
      setSettings(s);
      setRdStatus(rdStatusRes);
      const paths = parsePaths(s["media_paths"]);
      if (paths.length > 0 && !pathInput) setPathInput(paths[0]);
    } catch (err) {
      console.error("Failed to load settings:", err);
    } finally {
      setLoading(false);
    }
  }

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

  async function handleCheckPath() {
    const path = pathInput.trim() || getPaths()[0];
    if (!path) return;
    try {
      setCheckingPath(true);
      setPathCheck(null);
      const result = await checkMediaPath(path);
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
        error: String(err),
      });
    } finally {
      setCheckingPath(false);
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
          <code>/Volumes/Diskstation/video</code>
        </p>
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
            placeholder="/Pfad/zum/Ordner"
            value={pathInput}
            onChange={(e) => setPathInput(e.target.value)}
          />
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
            disabled={checkingPath || paths.length === 0}
          >
            {checkingPath ? "Prüfe…" : "Prüfen"}
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
        {pathCheck && (
          <div className="scan-result">
            {pathCheck.error ? (
              <p className="status-warn">{pathCheck.error}</p>
            ) : (
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
            )}
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
            onClick={async () => {
              try {
                setFetchingMetadata(true);
                const n = await fetchMetadataBatch();
                alert(`${n} Metadaten geladen.`);
              } catch (err) {
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
