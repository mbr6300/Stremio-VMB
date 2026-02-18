import type { MediaItem, MediaMetadata } from "../types";
import { useNavigate } from "react-router-dom";

interface MediaCardProps {
  item: MediaItem;
  metadata?: MediaMetadata | null;
  versionCount?: number;
}

export default function MediaCard({ item, metadata, versionCount = 1 }: MediaCardProps) {
  const navigate = useNavigate();
  const posterUrl = metadata?.poster_url;
  const displayTitle = metadata?.title || item.title;
  const year = metadata?.release_date?.split("-")[0];

  return (
    <div className="media-card" onClick={() => navigate(`/details/${item.id}`)}>
      <div className="media-card-poster">
        {posterUrl ? (
          <img src={posterUrl} alt={displayTitle} />
        ) : (
          <div className="media-card-placeholder">
            <span>{item.media_type === "movie" ? "🎬" : "📺"}</span>
          </div>
        )}
        <span className={`media-badge ${item.media_type}`}>
          {item.media_type === "movie" ? "Film" : "Serie"}
        </span>
      </div>
      <div className="media-card-info">
        <h3>{displayTitle}</h3>
        {year && <span className="media-year">{year}</span>}
        {metadata?.rating != null && (
          <span className="media-rating">★ {metadata.rating.toFixed(1)}</span>
        )}
        {versionCount > 1 && (
          <span className="media-versions">{versionCount} Versionen</span>
        )}
      </div>
    </div>
  );
}
