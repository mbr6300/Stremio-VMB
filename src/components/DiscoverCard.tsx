import { useNavigate } from "react-router-dom";
import type { DiscoverItem } from "../types";

interface DiscoverCardProps {
  item: DiscoverItem;
}

export default function DiscoverCard({ item }: DiscoverCardProps) {
  const navigate = useNavigate();

  function handleClick() {
    if (item.tmdb_id) {
      navigate(`/discover-detail/${item.tmdb_id}/${item.media_type}`);
    }
  }

  return (
    <div className="discover-card" onClick={handleClick}>
      <div className="discover-card-poster">
        {item.poster_url ? (
          <img src={item.poster_url} alt={item.title} loading="lazy" />
        ) : (
          <div className="discover-card-placeholder">
            {item.media_type === "movie" ? "🎬" : "📺"}
          </div>
        )}
      </div>
      <div className="discover-card-info">
        <span className="discover-card-title">{item.title}</span>
        <div className="discover-card-meta">
          {item.year && <span>{item.year}</span>}
          {item.rating != null && item.rating > 0 && (
            <span className="discover-card-rating">★ {item.rating.toFixed(1)}</span>
          )}
        </div>
      </div>
    </div>
  );
}
