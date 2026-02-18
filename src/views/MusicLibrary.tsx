import { useEffect, useState, useCallback } from "react";
import { useNavigate, Link } from "react-router-dom";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getMusicLibrary } from "../api/commands";
import type { MusicAlbum } from "../types";

export default function MusicLibrary() {
  const navigate = useNavigate();
  const [albums, setAlbums] = useState<MusicAlbum[]>([]);
  const [loading, setLoading] = useState(true);

  const loadLibrary = useCallback(async () => {
    try {
      setLoading(true);
      const data = await getMusicLibrary();
      setAlbums(data);
    } catch (err) {
      console.error("Failed to load music library:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadLibrary();
  }, [loadLibrary]);

  useEffect(() => {
    let unlisten1: (() => void) | undefined;
    let unlisten2: (() => void) | undefined;
    listen("music-album-added", loadLibrary).then((fn) => { unlisten1 = fn; });
    listen("music-scan-complete", loadLibrary).then((fn) => { unlisten2 = fn; });
    return () => {
      unlisten1?.();
      unlisten2?.();
    };
  }, [loadLibrary]);

  if (loading) return <div className="loading-state">Lade Musikbibliothek…</div>;

  return (
    <div className="view-music">
      <h1>Musik</h1>
      <div className="music-grid">
        {albums.map((album) => (
          <div
            key={album.id}
            className="music-album-card"
            onClick={() => navigate(`/music/album/${album.id}`)}
          >
            <div className="music-album-cover">
              {album.cover_path ? (
                <img
                  src={convertFileSrc(album.cover_path)}
                  alt={album.album_title}
                />
              ) : (
                <div className="music-album-placeholder">🎵</div>
              )}
            </div>
            <div className="music-album-info">
              <span className="music-album-title">{album.album_title}</span>
              <span className="music-album-artist">{album.artist}</span>
              {album.year && (
                <span className="music-album-year">{album.year}</span>
              )}
            </div>
          </div>
        ))}
      </div>
      {albums.length === 0 && !loading && (
        <p className="empty-state">
          Keine Alben. Musikpfade in den{" "}
          <Link to="/settings">Einstellungen</Link> hinzufügen und scannen.
        </p>
      )}
    </div>
  );
}
