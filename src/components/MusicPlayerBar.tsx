import { useNavigate } from "react-router-dom";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useMusicPlayer } from "../context/MusicPlayerContext";

export default function MusicPlayerBar() {
  const navigate = useNavigate();
  const { currentTrack, currentAlbum, isPlaying, togglePlayPause } =
    useMusicPlayer();

  if (!currentTrack || !currentAlbum) return null;

  return (
    <div
      className="music-player-bar"
      onClick={() => navigate("/music/now-playing")}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => e.key === "Enter" && navigate("/music/now-playing")}
    >
      <div className="music-player-cover">
        {currentAlbum.cover_path ? (
          <img src={convertFileSrc(currentAlbum.cover_path)} alt="" />
        ) : (
          <div className="music-player-cover-placeholder">🎵</div>
        )}
      </div>
      <div className="music-player-info">
        <span className="music-player-title">{currentTrack.title}</span>
        <span className="music-player-artist">{currentAlbum.artist}</span>
      </div>
      <button
        type="button"
        className="music-player-play-btn"
        onClick={(e) => {
          e.stopPropagation();
          togglePlayPause();
        }}
      >
        {isPlaying ? "⏸" : "▶"}
      </button>
    </div>
  );
}
