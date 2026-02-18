import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getMusicAlbum } from "../api/commands";
import { useMusicPlayer } from "../context/MusicPlayerContext";
import type { MusicAlbum, MusicTrack } from "../types";

export default function MusicAlbumDetail() {
  const { albumId } = useParams<{ albumId: string }>();
  const navigate = useNavigate();
  const { play } = useMusicPlayer();
  const [album, setAlbum] = useState<MusicAlbum | null>(null);
  const [tracks, setTracks] = useState<MusicTrack[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (albumId) loadAlbum(albumId);
  }, [albumId]);

  async function loadAlbum(id: string) {
    try {
      setLoading(true);
      const result = await getMusicAlbum(id);
      if (result) {
        setAlbum(result.album);
        setTracks(result.tracks);
      } else {
        setAlbum(null);
        setTracks([]);
      }
    } catch (err) {
      console.error("Failed to load album:", err);
    } finally {
      setLoading(false);
    }
  }

  function formatDuration(secs: number | null): string {
    if (secs == null) return "--:--";
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  function handlePlayTrack(track: MusicTrack) {
    if (!album) return;
    const queue = tracks.map((t) => ({ track: t, album }));
    play(track, album, queue);
    navigate("/music/now-playing");
  }

  if (loading) return <div className="loading-state">Lade Album…</div>;
  if (!album) return <div className="empty-state">Album nicht gefunden.</div>;

  return (
    <div className="view-music-album">
      <button type="button" className="back-button" onClick={() => navigate(-1)}>
        ← Zurück
      </button>
      <div className="music-album-header">
        <div className="music-album-cover-large">
          {album.cover_path ? (
            <img src={convertFileSrc(album.cover_path)} alt={album.album_title} />
          ) : (
            <div className="music-album-placeholder">🎵</div>
          )}
        </div>
        <div className="music-album-meta">
          <h1>{album.album_title}</h1>
          <p className="music-album-artist">{album.artist}</p>
          {album.year && <p className="music-album-year">{album.year}</p>}
        </div>
      </div>
      <ul className="music-track-list">
        {tracks.map((track, i) => (
          <li key={track.id}>
            <button
              type="button"
              className="music-track-row"
              onClick={() => handlePlayTrack(track)}
            >
              <span className="music-track-num">{i + 1}</span>
              <span className="music-track-title">{track.title}</span>
              <span className="music-track-duration">
                {formatDuration(track.duration)}
              </span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
