import { useEffect, useState, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
import {
  getMediaItem,
  loadSettings,
  detectPlayers,
  openInPlayer,
  openInDefaultPlayer,
} from "../api/commands";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { MediaItem, ExternalPlayer } from "../types";

export default function Player() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const videoRef = useRef<HTMLVideoElement>(null);
  const [item, setItem] = useState<MediaItem | null>(null);
  const [videoSrc, setVideoSrc] = useState<string>("");
  const [error, setError] = useState<string>("");
  const [players, setPlayers] = useState<ExternalPlayer[]>([]);
  const [preferredPlayer, setPreferredPlayer] = useState<string>("");
  const [useHtml5, setUseHtml5] = useState(false);
  const [launched, setLaunched] = useState(false);

  useEffect(() => {
    loadPlayerConfig();
  }, []);

  useEffect(() => {
    if (id && preferredPlayer !== undefined) loadMedia(id);
  }, [id, preferredPlayer]);

  async function loadPlayerConfig() {
    try {
      const [detectedPlayers, settings] = await Promise.all([
        detectPlayers(),
        loadSettings(),
      ]);
      setPlayers(detectedPlayers);
      const pref = settings["preferred_player"] || "";
      setPreferredPlayer(pref);

      if (pref === "html5") {
        setUseHtml5(true);
      }
    } catch (err) {
      console.error("Failed to load player config:", err);
    }
  }

  async function loadMedia(mediaId: string) {
    try {
      const mediaItem = await getMediaItem(mediaId);
      if (!mediaItem) {
        setError("Medium nicht gefunden.");
        return;
      }
      setItem(mediaItem);

      const isStreamingUrl = mediaItem.file_path.startsWith("http");

      if (useHtml5 || (isStreamingUrl && !hasExternalPlayer())) {
        if (isStreamingUrl) {
          setVideoSrc(mediaItem.file_path);
        } else {
          setVideoSrc(convertFileSrc(mediaItem.file_path));
        }
        return;
      }

      await launchExternalPlayer(mediaItem.file_path);
    } catch (err) {
      setError(`Fehler beim Laden: ${err}`);
    }
  }

  function hasExternalPlayer(): boolean {
    if (preferredPlayer && preferredPlayer !== "html5" && preferredPlayer !== "system") {
      return players.some((p) => p.id === preferredPlayer);
    }
    return players.length > 0;
  }

  async function launchExternalPlayer(filePath: string) {
    try {
      if (preferredPlayer && preferredPlayer !== "html5" && preferredPlayer !== "system") {
        await openInPlayer(preferredPlayer, filePath);
      } else if (preferredPlayer === "system" || players.length === 0) {
        await openInDefaultPlayer(filePath);
      } else {
        await openInPlayer(players[0].id, filePath);
      }
      setLaunched(true);
    } catch (err) {
      setError(`Fehler beim Starten des Players: ${err}`);
    }
  }

  function switchToHtml5() {
    if (!item) return;
    setUseHtml5(true);
    setLaunched(false);
    if (item.file_path.startsWith("http")) {
      setVideoSrc(item.file_path);
    } else {
      setVideoSrc(convertFileSrc(item.file_path));
    }
  }

  async function relaunchExternal() {
    if (!item) return;
    setUseHtml5(false);
    setVideoSrc("");
    await launchExternalPlayer(item.file_path);
  }

  const activePlayerName =
    preferredPlayer && preferredPlayer !== "html5" && preferredPlayer !== "system"
      ? players.find((p) => p.id === preferredPlayer)?.name
      : players[0]?.name;

  return (
    <div className="view-player">
      <div className="player-header">
        <button className="back-button" onClick={() => navigate(-1)}>
          ← Zurück
        </button>
        {item && <span className="player-title">{item.title}</span>}
      </div>

      <div className="player-container">
        {error ? (
          <div className="player-error">
            <p>{error}</p>
            {item && (
              <button className="btn-secondary player-retry" onClick={switchToHtml5}>
                Als HTML5-Video versuchen
              </button>
            )}
          </div>
        ) : launched && !useHtml5 ? (
          <div className="player-external-state">
            <div className="external-icon">
              {activePlayerName === "VLC" ? "🎬" : "▶"}
            </div>
            <h2>Wiedergabe in {activePlayerName || "externem Player"}</h2>
            <p>Das Video wird extern abgespielt.</p>
            <div className="player-switch-row">
              <button className="btn-secondary" onClick={relaunchExternal}>
                Erneut starten
              </button>
              <button className="btn-secondary" onClick={switchToHtml5}>
                Im Browser abspielen
              </button>
            </div>
          </div>
        ) : videoSrc ? (
          <video
            ref={videoRef}
            src={videoSrc}
            controls
            autoPlay
            className="video-element"
          >
            Dein Browser unterstützt dieses Format nicht.
          </video>
        ) : (
          <div className="loading-state">Lade Video...</div>
        )}
      </div>

      {useHtml5 && item && hasExternalPlayer() && (
        <div className="player-footer">
          <button className="btn-secondary" onClick={relaunchExternal}>
            In {activePlayerName || "externem Player"} öffnen
          </button>
        </div>
      )}
    </div>
  );
}
