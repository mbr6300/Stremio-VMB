import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useMusicPlayer } from "../context/MusicPlayerContext";
import StarRating from "../components/StarRating";

export default function MusicNowPlaying() {
  const navigate = useNavigate();
  const { currentTrack, currentAlbum, isPlaying, togglePlayPause, setRating } =
    useMusicPlayer();
  const [rating, setRatingState] = useState(0);

  if (!currentTrack || !currentAlbum) {
    return (
      <div className="empty-state">
        Kein Titel ausgewählt.{" "}
        <button type="button" onClick={() => navigate("/music")}>
          Zur Musikbibliothek
        </button>
      </div>
    );
  }

  return (
    <div className="view-music-now-playing">
      <button type="button" className="back-button" onClick={() => navigate(-1)}>
        ← Zurück
      </button>
      <div className="now-playing-cover">
        {currentAlbum.cover_path ? (
          <img
            src={convertFileSrc(currentAlbum.cover_path)}
            alt={currentAlbum.album_title}
          />
        ) : (
          <div className="music-album-placeholder">🎵</div>
        )}
      </div>
      <div className="now-playing-info">
        <h1>{currentTrack.title}</h1>
        <p>{currentAlbum.artist} – {currentAlbum.album_title}</p>
        <StarRating
          value={rating}
          size="large"
          onChange={(r) => {
            setRatingState(r);
            setRating(currentTrack.id, r);
          }}
        />
      </div>
      <div className="now-playing-controls">
        <button
          type="button"
          className="btn-primary btn-play-pause"
          onClick={togglePlayPause}
        >
          {isPlaying ? "⏸ Pause" : "▶ Abspielen"}
        </button>
      </div>
    </div>
  );
}
