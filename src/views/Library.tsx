import { useState, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import MediaCard from "../components/MediaCard";
import { useLibrary } from "../context/LibraryContext";
import { useScan } from "../context/ScanContext";
import { extractSeriesName, parseEpisodeInfo, slugify } from "../utils/series";
import { fetchMetadataBatch, improveClassificationsWithPerplexity } from "../api/commands";
import type { LibraryItemWithMeta, SeriesCluster } from "../types";

type FilterType = "all" | "movie" | "series";
type SortType = "title" | "year" | "rating" | "genre";

function buildClusters(items: LibraryItemWithMeta[]): {
  movies: LibraryItemWithMeta[];
  series: SeriesCluster[];
} {
  const movies: LibraryItemWithMeta[] = [];
  const seriesMap = new Map<string, LibraryItemWithMeta[]>();

  for (const entry of items) {
    const isSeries =
      entry.item.media_type === "series" || parseEpisodeInfo(entry.item.title) !== null;
    if (!isSeries) {
      movies.push(entry);
    } else {
      const seriesName =
        entry.item.series_name?.trim() ||
        extractSeriesName(entry.item.title);
      const key = slugify(seriesName);
      const list = seriesMap.get(key) ?? [];
      list.push(entry);
      seriesMap.set(key, list);
    }
  }

  const series: SeriesCluster[] = Array.from(seriesMap.entries()).map(
    ([key, episodes]) => {
      const first = episodes[0];
      const title =
        first.item.series_name?.trim() ||
        extractSeriesName(first.item.title);
      const metadata = episodes.find((e) => e.metadata)?.metadata ?? null;
      return {
        key,
        title,
        episodes,
        metadata,
      };
    }
  );

  return { movies, series };
}

export default function Library() {
  const navigate = useNavigate();
  const { items, loading, reload } = useLibrary();
  const { isFetchingMetadata, setFetchingMetadata } = useScan();
  const [isImproving, setIsImproving] = useState(false);
  const [filter, setFilter] = useState<FilterType>("all");
  const [search, setSearch] = useState("");
  const [sortBy, setSortBy] = useState<SortType>("title");
  const [genreFilter, setGenreFilter] = useState<string>("");

  const { movies, series } = useMemo(() => buildClusters(items), [items]);

  const genres = useMemo(() => {
    const set = new Set<string>();
    for (const { metadata } of items) {
      if (metadata?.genres) {
        try {
          const parsed = JSON.parse(metadata.genres) as string[];
          parsed.forEach((g) => set.add(g));
        } catch { /* ignore parse error */ }
      }
    }
    return Array.from(set).sort();
  }, [items]);

  const filteredMovies = useMemo(() => {
    const result = movies.filter((m) => {
      const title = m.metadata?.title || m.item.title;
      if (search && !title.toLowerCase().includes(search.toLowerCase()))
        return false;
      if (genreFilter) {
        const g = m.metadata?.genres;
        if (!g) return false;
        try {
          const arr = JSON.parse(g) as string[];
          return arr.includes(genreFilter);
        } catch {
          return false;
        }
      }
      return true;
    });
    const getYear = (d: LibraryItemWithMeta) =>
      d.metadata?.release_date ? parseInt(d.metadata.release_date.slice(0, 4), 10) : 0;
    const getTitle = (d: LibraryItemWithMeta) => d.metadata?.title || d.item.title;
    const getFirstGenre = (d: LibraryItemWithMeta) => {
      if (!d.metadata?.genres) return "";
      try {
        const arr = JSON.parse(d.metadata.genres) as string[];
        return arr[0] ?? "";
      } catch {
        return "";
      }
    };
    result.sort((a, b) => {
      switch (sortBy) {
        case "year":
          return getYear(b) - getYear(a);
        case "rating":
          return (b.metadata?.rating ?? 0) - (a.metadata?.rating ?? 0) > 0 ? 1 : -1;
        case "genre":
          return getFirstGenre(a).localeCompare(getFirstGenre(b)) || getTitle(a).localeCompare(getTitle(b));
        default:
          return getTitle(a).localeCompare(getTitle(b));
      }
    });
    return result;
  }, [movies, search, genreFilter, sortBy]);

  const filteredSeries = useMemo(() => {
    const result = series.filter((s) => {
      const title = s.metadata?.title || s.title;
      if (search && !title.toLowerCase().includes(search.toLowerCase()))
        return false;
      if (genreFilter) {
        const g = s.metadata?.genres;
        if (!g) return false;
        try {
          const arr = JSON.parse(g) as string[];
          return arr.includes(genreFilter);
        } catch {
          return false;
        }
      }
      return true;
    });
    const getYear = (s: SeriesCluster) =>
      s.metadata?.release_date ? parseInt(s.metadata.release_date.slice(0, 4), 10) : 0;
    const getTitle = (s: SeriesCluster) => s.metadata?.title || s.title;
    const getFirstGenre = (s: SeriesCluster) => {
      if (!s.metadata?.genres) return "";
      try {
        const arr = JSON.parse(s.metadata.genres) as string[];
        return arr[0] ?? "";
      } catch {
        return "";
      }
    };
    result.sort((a, b) => {
      switch (sortBy) {
        case "year":
          return getYear(b) - getYear(a);
        case "rating":
          return (b.metadata?.rating ?? 0) - (a.metadata?.rating ?? 0) > 0 ? 1 : -1;
        case "genre":
          return getFirstGenre(a).localeCompare(getFirstGenre(b)) || getTitle(a).localeCompare(getTitle(b));
        default:
          return getTitle(a).localeCompare(getTitle(b));
      }
    });
    return result;
  }, [series, search, genreFilter, sortBy]);

  return (
    <div className="view-library">
      <header className="view-header">
        <h1>Bibliothek</h1>
        <div className="view-controls">
          <input
            type="text"
            placeholder="Suchen..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="search-input"
          />
          <div className="filter-tabs">
            <button
              className={filter === "all" ? "active" : ""}
              onClick={() => setFilter("all")}
            >
              Alle
            </button>
            <button
              className={filter === "movie" ? "active" : ""}
              onClick={() => setFilter("movie")}
            >
              Filme
            </button>
            <button
              className={filter === "series" ? "active" : ""}
              onClick={() => setFilter("series")}
            >
              Serien
            </button>
          </div>
          <select
            className="sort-select"
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as SortType)}
          >
            <option value="title">Titel</option>
            <option value="year">Jahr</option>
            <option value="rating">Bewertung</option>
            <option value="genre">Genre</option>
          </select>
          {items.some((i) => !i.metadata) && items.length > 0 && (
            <button
              type="button"
              className="btn-secondary"
              disabled={isFetchingMetadata}
              onClick={async () => {
                try {
                  setFetchingMetadata(true);
                  const n = await fetchMetadataBatch();
                  await reload();
                  if (n > 0) {
                    // Erfolg – LibraryContext lädt bereits via metadata-batch-complete
                  }
                } catch {
                  // TMDb-Key fehlt oder Fehler – Nutzer in Einstellungen schicken
                } finally {
                  setFetchingMetadata(false);
                }
              }}
            >
              {isFetchingMetadata ? "Lade Metadaten…" : "Metadaten laden"}
            </button>
          )}
          {items.length > 0 && (
            <button
              type="button"
              className="btn-secondary"
              disabled={isImproving}
              onClick={async () => {
                try {
                  setIsImproving(true);
                  const n = await improveClassificationsWithPerplexity();
                  await reload();
                  if (n > 0) {
                    // Erfolg
                  }
                } catch {
                  alert("AI-Klassifizierung fehlgeschlagen. Perplexity-API-Key in Einstellungen setzen.");
                } finally {
                  setIsImproving(false);
                }
              }}
            >
              {isImproving ? "AI klassifiziert…" : "Mit AI klassifizieren"}
            </button>
          )}
          {genres.length > 0 && (
            <select
              className="genre-select"
              value={genreFilter}
              onChange={(e) => setGenreFilter(e.target.value)}
            >
              <option value="">Alle Genres</option>
              {genres.map((g) => (
                <option key={g} value={g}>
                  {g}
                </option>
              ))}
            </select>
          )}
        </div>
      </header>

      {loading ? (
        <div className="loading-state">Lade Bibliothek...</div>
      ) : filteredMovies.length === 0 && filteredSeries.length === 0 ? (
        <div className="empty-state">
          <p>Keine Medien gefunden.</p>
          <p>
            Füge Medienpfade in den <strong>Einstellungen</strong> hinzu und
            starte einen Scan.
          </p>
        </div>
      ) : (
        <div className="library-sections">
          {filter !== "series" && filteredMovies.length > 0 && (
            <section className="library-section">
              <h2 className="library-section-title">
                <span className="section-icon">🎬</span>
                Filme
                <span className="section-count">({filteredMovies.length})</span>
              </h2>
              <div className="media-grid">
                {filteredMovies.map((m) => (
                  <MediaCard
                    key={m.item.id}
                    item={m.item}
                    metadata={m.metadata}
                  />
                ))}
              </div>
            </section>
          )}
          {filter !== "movie" && filteredSeries.length > 0 && (
            <section className="library-section">
              <h2 className="library-section-title">
                <span className="section-icon">📺</span>
                Serien
                <span className="section-count">({filteredSeries.length})</span>
              </h2>
              <div className="media-grid">
                {filteredSeries.map((cluster) => (
                  <div
                    key={cluster.key}
                    className="series-card"
                    onClick={() => navigate(`/library/series/${cluster.key}`)}
                  >
                    <div className="media-card-poster">
                      {cluster.metadata?.poster_url ? (
                        <img
                          src={cluster.metadata.poster_url}
                          alt={cluster.title}
                        />
                      ) : (
                        <div className="media-card-placeholder">
                          <span>📺</span>
                        </div>
                      )}
                      <span className="media-badge series">Serie</span>
                      <span className="series-episode-badge">
                        {cluster.episodes.length} Folgen
                      </span>
                    </div>
                    <div className="media-card-info">
                      <h3>{cluster.metadata?.title || cluster.title}</h3>
                      {cluster.metadata?.release_date && (
                        <span className="media-year">
                          {cluster.metadata.release_date.slice(0, 4)}
                        </span>
                      )}
                      {cluster.metadata?.rating != null && (
                        <span className="media-rating">
                          ★ {cluster.metadata.rating.toFixed(1)}
                        </span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </section>
          )}
        </div>
      )}
    </div>
  );
}
