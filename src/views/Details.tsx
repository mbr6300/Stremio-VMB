import { useEffect, useState, useMemo, useRef } from "react";
import { createPortal } from "react-dom";
import { useParams, useNavigate } from "react-router-dom";
import {
  getMediaItem,
  getMetadata,
  fetchMetadata,
  detectPlayers,
  openInPlayer,
  getPersonDetails,
  getMediaExtendedInfo,
  getActorMovieSuggestions,
} from "../api/commands";
import FormatBoldText from "../components/FormatBoldText";
import { useLibrary } from "../context/LibraryContext";
import { buildMovieGroups } from "../utils/libraryGrouping";
import type {
  MediaItem,
  MediaMetadata,
  CastCrew,
  CastMember,
  ExternalPlayer,
  PersonDetails,
  MediaExtendedInfo,
  ActorMovieSuggestion,
} from "../types";

export default function Details() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { items: libraryItems } = useLibrary();
  const [item, setItem] = useState<MediaItem | null>(null);
  const [metadata, setMetadata] = useState<MediaMetadata | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshingMeta, setRefreshingMeta] = useState(false);
  const [players, setPlayers] = useState<ExternalPlayer[]>([]);
  const [personOverlay, setPersonOverlay] = useState<PersonDetails | null>(null);
  const [personLoading, setPersonLoading] = useState(false);
  const [extendedOverlay, setExtendedOverlay] = useState<MediaExtendedInfo | null>(null);
  const [extendedLoading, setExtendedLoading] = useState(false);
  const [actorSuggestions, setActorSuggestions] = useState<ActorMovieSuggestion[]>([]);
  const [selectedVariantId, setSelectedVariantId] = useState<string | null>(null);
  const personCacheRef = useRef<Map<number, PersonDetails>>(new Map());
  const extendedInfoRef = useRef<MediaExtendedInfo | null>(null);
  const preloadTmdbIdRef = useRef<number | null>(null);

  useEffect(() => {
    if (id) loadDetails(id);
    detectPlayers().then(setPlayers).catch(() => {});
  }, [id]);

  async function loadDetails(mediaId: string) {
    try {
      setLoading(true);
      const mediaItem = await getMediaItem(mediaId);
      setItem(mediaItem);

      let meta = await getMetadata(mediaId);
      if (!meta) {
        meta = await fetchMetadata(mediaId);
      }
      setMetadata(meta ?? null);
    } catch (err) {
      console.error("Failed to load details:", err);
    } finally {
      setLoading(false);
    }
  }

  const genres: string[] = useMemo(() => {
    if (!metadata?.genres) return [];
    try {
      return JSON.parse(metadata.genres);
    } catch {
      return [];
    }
  }, [metadata?.genres]);

  const castCrew: CastCrew | null = useMemo(() => {
    if (!metadata?.cast_crew) return null;
    try {
      return JSON.parse(metadata.cast_crew);
    } catch {
      return null;
    }
  }, [metadata?.cast_crew]);

  const directors = useMemo(
    () =>
      castCrew?.crew.filter(
        (c) => c.job === "Director" || c.job === "Creator"
      ) ?? [],
    [castCrew]
  );

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setPersonOverlay(null);
        setExtendedOverlay(null);
      }
    };
    if (personOverlay || extendedOverlay) {
      document.addEventListener("keydown", handler);
      return () => document.removeEventListener("keydown", handler);
    }
  }, [personOverlay, extendedOverlay]);

  useEffect(() => {
    if (!metadata?.tmdb_id || !castCrew?.cast?.length || !item) return;
    const ids = castCrew.cast
      .filter((m): m is CastMember & { id: number } => typeof m.id === "number")
      .map((m) => m.id)
      .slice(0, 5);
    if (ids.length === 0) return;
    getActorMovieSuggestions(metadata.tmdb_id, item.media_type, ids, [])
      .then(setActorSuggestions)
      .catch(() => setActorSuggestions([]));
  }, [metadata?.tmdb_id, castCrew?.cast, item]);

  useEffect(() => {
    if (!metadata?.tmdb_id || !item) return;
    const knownFor = metadata.title || item.title;
    personCacheRef.current.clear();
    extendedInfoRef.current = null;
    preloadTmdbIdRef.current = metadata.tmdb_id;

    getMediaExtendedInfo(
      metadata.tmdb_id,
      item.media_type,
      metadata.title ?? item.title ?? undefined,
      metadata.release_date?.slice(0, 4) ?? undefined
    )
      .then((info) => {
        if (preloadTmdbIdRef.current === metadata.tmdb_id) {
          extendedInfoRef.current = info;
        }
      })
      .catch(() => {});

    const castIds = (castCrew?.cast ?? [])
      .filter((m): m is CastMember & { id: number } => typeof m.id === "number")
      .map((m) => m.id)
      .slice(0, 5);

    castIds.forEach((pid) => {
      getPersonDetails(pid, knownFor ?? undefined)
        .then((details) => {
          if (preloadTmdbIdRef.current === metadata.tmdb_id) {
            personCacheRef.current.set(pid, details);
          }
        })
        .catch(() => {});
    });
  }, [metadata?.tmdb_id, metadata?.title, metadata?.release_date, item, castCrew?.cast]);

  async function handleActorClick(member: CastMember) {
    if (member.id == null) return;
    const cached = personCacheRef.current.get(member.id);
    if (cached) {
      setPersonOverlay(cached);
      return;
    }
    setPersonLoading(true);
    setPersonOverlay(null);
    try {
      const knownFor = metadata?.title || item?.title;
      const details = await getPersonDetails(member.id, knownFor ?? undefined);
      personCacheRef.current.set(member.id, details);
      setPersonOverlay(details);
    } catch (err) {
      console.error("Person details failed:", err);
    } finally {
      setPersonLoading(false);
    }
  }

  async function handleTriviaClick() {
    if (!metadata?.tmdb_id || !item) return;
    const cached = extendedInfoRef.current;
    if (cached) {
      setExtendedOverlay(cached);
      return;
    }
    setExtendedLoading(true);
    setExtendedOverlay(null);
    try {
      const info = await getMediaExtendedInfo(
        metadata.tmdb_id,
        item.media_type,
        metadata?.title ?? item?.title ?? undefined,
        metadata?.release_date?.slice(0, 4) ?? undefined
      );
      extendedInfoRef.current = info;
      setExtendedOverlay(info);
    } catch (err) {
      console.error("Extended info failed:", err);
    } finally {
      setExtendedLoading(false);
    }
  }

  const movieVariants = useMemo(() => {
    if (!item || item.media_type !== "movie") {
      return item ? [{ item, metadata }] : [];
    }
    const groups = buildMovieGroups(libraryItems);
    const match = groups.find((group) => group.entries.some((entry) => entry.item.id === item.id));
    if (match) return match.entries;
    return [{ item, metadata }];
  }, [libraryItems, item, metadata]);

  useEffect(() => {
    if (movieVariants.length === 0) {
      setSelectedVariantId(null);
      return;
    }
    const hasSelected = selectedVariantId != null
      && movieVariants.some((entry) => entry.item.id === selectedVariantId);
    if (hasSelected) return;
    const preferred = item ? movieVariants.find((entry) => entry.item.id === item.id) : null;
    if (preferred) {
      setSelectedVariantId(preferred.item.id);
      return;
    }
    const fallback = [...movieVariants].sort(
      (a, b) => (b.item.file_size ?? 0) - (a.item.file_size ?? 0)
    )[0];
    setSelectedVariantId(fallback.item.id);
  }, [item, movieVariants, selectedVariantId]);

  const selectedVariant = useMemo(() => {
    if (movieVariants.length === 0) return null;
    return movieVariants.find((entry) => entry.item.id === selectedVariantId) ?? movieVariants[0];
  }, [movieVariants, selectedVariantId]);

  if (loading) return <div className="loading-state">Lade Details...</div>;
  if (!item) return <div className="empty-state">Medium nicht gefunden.</div>;

  const displayTitle = metadata?.title || item.title;
  const backdropUrl = metadata?.backdrop_url;
  const posterUrl = metadata?.poster_url;

  function formatRuntime(minutes: number): string {
    const h = Math.floor(minutes / 60);
    const m = minutes % 60;
    return h > 0 ? `${h} Std. ${m} Min.` : `${m} Min.`;
  }

  function formatFileSize(bytes: number | null): string | null {
    if (!bytes || bytes <= 0) return null;
    return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`;
  }

  function detectQualityLabel(entry: MediaItem): string {
    const source = `${entry.title} ${entry.file_path}`.toLowerCase();
    const qualityMatch = source.match(/\b(2160p|1080p|720p|480p|4k|8k|uhd)\b/i);
    if (qualityMatch?.[1]) return qualityMatch[1].toUpperCase();
    return "Standard";
  }

  return (
    <div className="view-details">
      {backdropUrl && (
        <div
          className="details-backdrop"
          style={{ backgroundImage: `url(${backdropUrl})` }}
        />
      )}
      <div className="details-content">
        <button className="back-button" onClick={() => navigate(-1)}>
          ← Zurück
        </button>
        <div className="details-layout">
          <div className="details-poster">
            {posterUrl ? (
              <img src={posterUrl} alt={displayTitle} />
            ) : (
              <div className="poster-placeholder">
                {item.media_type === "movie" ? "🎬" : "📺"}
              </div>
            )}
          </div>
          <div className="details-info">
            <h1>{displayTitle}</h1>

            <div className="details-meta">
              <span className={`media-badge ${item.media_type}`}>
                {item.media_type === "movie" ? "Film" : "Serie"}
              </span>
              {metadata?.release_date && (
                <span className="meta-year">
                  {metadata.release_date.split("-")[0]}
                </span>
              )}
              {metadata?.runtime != null && metadata.runtime > 0 && (
                <span className="meta-runtime">
                  {formatRuntime(metadata.runtime)}
                </span>
              )}
              {metadata?.rating != null && (
                <span className="meta-rating">
                  ★ {metadata.rating.toFixed(1)}
                </span>
              )}
            </div>

            {genres.length > 0 && (
              <div className="details-genres">
                {genres.map((g: string) => (
                  <span key={g} className="genre-tag">
                    {g}
                  </span>
                ))}
              </div>
            )}

            {metadata?.overview && (
              <p className="details-overview">{metadata.overview}</p>
            )}

            {directors.length > 0 && (
              <div className="details-crew-line">
                <strong>
                  {item.media_type === "series" ? "Creator" : "Regie"}:
                </strong>{" "}
                {directors.map((d) => d.name).join(", ")}
              </div>
            )}

            <button
              type="button"
              className="btn-secondary details-trivia-btn"
              disabled={extendedLoading}
              onClick={handleTriviaClick}
            >
              {extendedLoading ? "Lade…" : "Weitere Informationen & Trivia"}
            </button>

            {castCrew && castCrew.cast.length > 0 && (
              <div className="details-cast">
                <h3>Besetzung</h3>
                <div className="cast-grid">
                  {castCrew.cast.slice(0, 10).map((member) => (
                    <div
                      key={`${member.id ?? member.name}-${member.character}`}
                      className={`cast-card ${member.id != null ? "cast-card-clickable" : ""}`}
                      role={member.id != null ? "button" : undefined}
                      tabIndex={member.id != null ? 0 : undefined}
                      onClick={() => member.id != null && handleActorClick(member)}
                      onKeyDown={(e) =>
                        member.id != null &&
                        (e.key === "Enter" || e.key === " ") &&
                        handleActorClick(member)
                      }
                    >
                      {member.profile_url ? (
                        <img
                          src={member.profile_url}
                          alt={member.name}
                          className="cast-photo"
                        />
                      ) : (
                        <div className="cast-photo-placeholder">👤</div>
                      )}
                      <div className="cast-info">
                        <span className="cast-name">{member.name}</span>
                        {member.character && (
                          <span className="cast-character">
                            {member.character}
                          </span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {actorSuggestions.length > 0 && (
              <div className="details-suggestions">
                <h3>Dazu passend</h3>
                <div className="suggestions-grid">
                  {actorSuggestions.slice(0, 8).map((s) => (
                    <button
                      key={s.tmdb_id}
                      type="button"
                      className="suggestion-card"
                      onClick={() =>
                        navigate(`/discover-detail/${s.tmdb_id}/${s.media_type}`)
                      }
                    >
                      {s.poster_url ? (
                        <img src={s.poster_url} alt={s.title} />
                      ) : (
                        <div className="suggestion-placeholder">🎬</div>
                      )}
                      <span className="suggestion-title">{s.title}</span>
                      {s.year != null && (
                        <span className="suggestion-year">{s.year}</span>
                      )}
                    </button>
                  ))}
                </div>
              </div>
            )}

            {movieVariants.length > 1 && (
              <div className="details-variants">
                <h3>Dateiversion wählen ({movieVariants.length})</h3>
                <div className="variant-list">
                  {movieVariants
                    .slice()
                    .sort((a, b) => (b.item.file_size ?? 0) - (a.item.file_size ?? 0))
                    .map((entry, index) => (
                      <button
                        key={entry.item.id}
                        type="button"
                        className={`variant-btn ${selectedVariant?.item.id === entry.item.id ? "active" : ""}`}
                        onClick={() => setSelectedVariantId(entry.item.id)}
                      >
                        <span className="variant-main">
                          {detectQualityLabel(entry.item)} · Datei {index + 1}
                        </span>
                        {formatFileSize(entry.item.file_size) && (
                          <span className="variant-size">{formatFileSize(entry.item.file_size)}</span>
                        )}
                      </button>
                    ))}
                </div>
              </div>
            )}

            <div className="details-actions">
              <button
                className="btn-primary"
                onClick={() => navigate(`/player/${selectedVariant?.item.id ?? item.id}`)}
              >
                ▶ Abspielen
              </button>
              <button
                type="button"
                className="btn-secondary"
                disabled={refreshingMeta}
                onClick={async () => {
                  if (!id) return;
                  try {
                    setRefreshingMeta(true);
                    const meta = await fetchMetadata(id);
                    setMetadata(meta ?? null);
                  } catch (err) {
                    console.error("Metadaten-Aktualisierung fehlgeschlagen:", err);
                  } finally {
                    setRefreshingMeta(false);
                  }
                }}
              >
                {refreshingMeta ? "Aktualisiere…" : "Metadaten aktualisieren"}
              </button>
              {players.length > 0 && (
                <button
                  className="btn-secondary"
                  onClick={() =>
                    openInPlayer(players[0].id, selectedVariant?.item.file_path ?? item.file_path)
                  }
                >
                  In {players[0].name} öffnen
                </button>
              )}
            </div>

            <div className="details-file-info">
              <small>Pfad: {selectedVariant?.item.file_path ?? item.file_path}</small>
              {selectedVariant?.item.file_size && (
                <small>Größe: {(selectedVariant.item.file_size / 1024 / 1024 / 1024).toFixed(2)} GB</small>
              )}
            </div>
          </div>
        </div>
      </div>

      {personLoading &&
        createPortal(
          <div className="overlay-backdrop">
            <div className="overlay-content overlay-loading">Lade Schauspieler-Details…</div>
          </div>,
          document.body
        )}

      {personOverlay &&
        !personLoading &&
        createPortal(
        <div
          className="overlay-backdrop"
          onClick={() => setPersonOverlay(null)}
        >
          <div
            className="overlay-content overlay-person"
            onClick={(e) => e.stopPropagation()}
          >
            <button
              type="button"
              className="overlay-close"
              onClick={() => setPersonOverlay(null)}
              aria-label="Schließen"
            >
              ×
            </button>
            <div className="overlay-person-header">
              {personOverlay.profile_url ? (
                <img
                  src={personOverlay.profile_url}
                  alt={personOverlay.name}
                  className="overlay-person-photo"
                />
              ) : (
                <div className="overlay-person-photo-placeholder">👤</div>
              )}
              <div>
                <h2>{personOverlay.name}</h2>
                {personOverlay.known_for_department && (
                  <p className="overlay-person-dept">{personOverlay.known_for_department}</p>
                )}
                {(personOverlay.birthday || personOverlay.place_of_birth) && (
                  <p className="overlay-person-meta">
                    {personOverlay.birthday}
                    {personOverlay.place_of_birth && ` · ${personOverlay.place_of_birth}`}
                    {personOverlay.age != null && ` · ${personOverlay.age} Jahre`}
                  </p>
                )}
                {(personOverlay.height || personOverlay.partner_status) && (
                  <p className="overlay-person-meta overlay-person-header-facts">
                    {personOverlay.height && <span>Größe: {personOverlay.height}</span>}
                    {personOverlay.height && personOverlay.partner_status && " · "}
                    {personOverlay.partner_status && (
                      <span>Partner: {personOverlay.partner_status}</span>
                    )}
                  </p>
                )}
              </div>
            </div>
            {personOverlay.biography && (
              <div className="overlay-person-bio">
                <FormatBoldText text={personOverlay.biography} asParagraphs />
              </div>
            )}
            {(personOverlay.height || personOverlay.partner_status || personOverlay.children) && (
              <div className="overlay-person-facts">
                {personOverlay.height && (
                  <p><strong>Größe:</strong> {personOverlay.height}</p>
                )}
                {personOverlay.partner_status && (
                  <p><FormatBoldText text={`**Partner:** ${personOverlay.partner_status}`} /></p>
                )}
                {personOverlay.children && (
                  <p><FormatBoldText text={`**Kinder:** ${personOverlay.children}`} /></p>
                )}
              </div>
            )}
            {personOverlay.anecdotes.length > 0 && (
              <div className="overlay-person-anecdotes">
                <h4>Anekdoten</h4>
                <ul>
                  {personOverlay.anecdotes.map((a, i) => (
                    <li key={i}><FormatBoldText text={a} /></li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>,
        document.body
        )}

      {extendedOverlay &&
        !extendedLoading &&
        createPortal(
        <div
          className="overlay-backdrop"
          onClick={() => setExtendedOverlay(null)}
        >
          <div
            className="overlay-content overlay-extended"
            onClick={(e) => e.stopPropagation()}
          >
            <button
              type="button"
              className="overlay-close"
              onClick={() => setExtendedOverlay(null)}
              aria-label="Schließen"
            >
              ×
            </button>
            <h2>Weitere Informationen & Trivia</h2>
            {extendedOverlay.tagline && (
              <p className="overlay-tagline">
                <FormatBoldText text={extendedOverlay.tagline} />
              </p>
            )}
            {extendedOverlay.trivia_facts.length > 0 && (
              <div className="overlay-trivia">
                <h4>Trivia</h4>
                <ul>
                  {extendedOverlay.trivia_facts.map((t, i) => (
                    <li key={i}><FormatBoldText text={t} /></li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>,
        document.body
        )}
    </div>
  );
}
