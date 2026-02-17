import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { getLibraryWithMetadata } from "../api/commands";
import {
  extractSeriesName,
  parseEpisodeInfo,
  slugify,
} from "../utils/series";
import type { LibraryItemWithMeta } from "../types";

export default function LibrarySeries() {
  const { seriesId } = useParams<{ seriesId: string }>();
  const navigate = useNavigate();
  const [cluster, setCluster] = useState<{
    title: string;
    episodes: LibraryItemWithMeta[];
  } | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (seriesId) loadSeries(seriesId);
  }, [seriesId]);

  async function loadSeries(id: string) {
    try {
      setLoading(true);
      const data = await getLibraryWithMetadata();
      const seriesItems = data.filter(
        (x) => x.item.media_type === "series" && slugify(extractSeriesName(x.item.title)) === id
      );
      if (seriesItems.length > 0) {
        const title = extractSeriesName(seriesItems[0].item.title);
        const sorted = [...seriesItems].sort((a, b) => {
          const ai = parseEpisodeInfo(a.item.title);
          const bi = parseEpisodeInfo(b.item.title);
          if (!ai) return 1;
          if (!bi) return -1;
          if (ai.season !== bi.season) return ai.season - bi.season;
          return ai.episode - bi.episode;
        });
        setCluster({ title, episodes: sorted });
      } else {
        setCluster(null);
      }
    } catch (err) {
      console.error("Failed to load series:", err);
      setCluster(null);
    } finally {
      setLoading(false);
    }
  }

  if (loading) return <div className="loading-state">Lade Serie…</div>;
  if (!cluster) return <div className="empty-state">Serie nicht gefunden.</div>;

  const bySeason = cluster.episodes.reduce<Record<number, LibraryItemWithMeta[]>>((acc, ep) => {
    const info = parseEpisodeInfo(ep.item.title);
    const s = info?.season ?? 0;
    if (!acc[s]) acc[s] = [];
    acc[s].push(ep);
    return acc;
  }, {});

  const seasons = Object.keys(bySeason)
    .map(Number)
    .sort((a, b) => a - b);

  return (
    <div className="view-library-series">
      <button className="back-button" onClick={() => navigate("/library")}>
        ← Zurück zur Bibliothek
      </button>
      <h1>{cluster.title}</h1>
      <p className="series-episode-count">
        {cluster.episodes.length} Folge{cluster.episodes.length !== 1 ? "n" : ""}
      </p>
      <div className="series-seasons">
        {seasons.map((seasonNum) => (
          <section key={seasonNum} className="series-season">
            <h2>Staffel {seasonNum}</h2>
            <div className="episode-grid">
              {bySeason[seasonNum].map(({ item, metadata }) => {
                const info = parseEpisodeInfo(item.title);
                return (
                  <div
                    key={item.id}
                    className="episode-card"
                    onClick={() => navigate(`/details/${item.id}`)}
                  >
                    <div className="episode-poster">
                      {metadata?.poster_url ? (
                        <img src={metadata.poster_url} alt={item.title} />
                      ) : (
                        <div className="episode-placeholder">
                          {info?.display ?? "?"}
                        </div>
                      )}
                    </div>
                    <div className="episode-info">
                      <span className="episode-label">
                        {info?.display ?? item.title}
                      </span>
                      <span className="episode-title">
                        {metadata?.title || item.title}
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          </section>
        ))}
      </div>
    </div>
  );
}
