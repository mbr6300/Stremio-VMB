import { useEffect, useState, useMemo, useRef } from "react";
import { createPortal } from "react-dom";
import { useParams, useNavigate } from "react-router-dom";
import {
  fetchMetadataByTmdbId,
  detectPlayers,
  openInPlayer,
  searchRdStreams,
  getPersonDetails,
  getMediaExtendedInfo,
  getActorMovieSuggestions,
} from "../api/commands";
import FormatBoldText from "../components/FormatBoldText";
import type {
  MediaMetadata,
  CastCrew,
  CastMember,
  ExternalPlayer,
  RdStreamLink,
  PersonDetails,
  MediaExtendedInfo,
  ActorMovieSuggestion,
} from "../types";

export default function DiscoverDetail() {
  const { tmdbId, mediaType } = useParams<{ tmdbId: string; mediaType: string }>();
  const navigate = useNavigate();
  const [metadata, setMetadata] = useState<MediaMetadata | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [players, setPlayers] = useState<ExternalPlayer[]>([]);
  const [streams, setStreams] = useState<RdStreamLink[]>([]);
  const [streamsLoading, setStreamsLoading] = useState(false);
  const [personOverlay, setPersonOverlay] = useState<PersonDetails | null>(null);
  const [personLoading, setPersonLoading] = useState(false);
  const [extendedOverlay, setExtendedOverlay] = useState<MediaExtendedInfo | null>(null);
  const [extendedLoading, setExtendedLoading] = useState(false);
  const [actorSuggestions, setActorSuggestions] = useState<ActorMovieSuggestion[]>([]);
  const personCacheRef = useRef<Map<number, PersonDetails>>(new Map());
  const extendedInfoRef = useRef<MediaExtendedInfo | null>(null);
  const preloadTmdbIdRef = useRef<number | null>(null);

  useEffect(() => {
    if (tmdbId && mediaType) loadDetail(Number(tmdbId), mediaType);
    detectPlayers().then(setPlayers).catch(() => {});
  }, [tmdbId, mediaType]);

  useEffect(() => {
    if (metadata && (mediaType === "movie" || mediaType === "tv")) {
      loadStreams();
    } else {
      setStreams([]);
    }
  }, [metadata?.title, metadata?.release_date, mediaType, tmdbId]);

  async function loadDetail(id: number, type: string) {
    try {
      setLoading(true);
      setError("");
      const meta = await fetchMetadataByTmdbId(
        `discover-${id}`,
        id,
        type === "tv" ? "series" : "movie"
      );
      setMetadata(meta);
    } catch (err) {
      setError(`Metadaten konnten nicht geladen werden: ${err}`);
    } finally {
      setLoading(false);
    }
  }

  async function loadStreams() {
    if (!metadata?.title) return;
    try {
      setStreamsLoading(true);
      setStreams([]);
      const year = metadata.release_date
        ? parseInt(metadata.release_date.slice(0, 4), 10)
        : null;
      const type = mediaType === "tv" ? "series" : "movie";
      const links = await searchRdStreams(
        metadata.title,
        year,
        type,
        tmdbId ? parseInt(tmdbId, 10) || null : null
      );
      setStreams(links);
    } catch {
      setStreams([]);
    } finally {
      setStreamsLoading(false);
    }
  }

  const genres: string[] = useMemo(() => {
    if (!metadata?.genres) return [];
    try { return JSON.parse(metadata.genres); } catch { return []; }
  }, [metadata?.genres]);

  const castCrew: CastCrew | null = useMemo(() => {
    if (!metadata?.cast_crew) return null;
    try { return JSON.parse(metadata.cast_crew); } catch { return null; }
  }, [metadata?.cast_crew]);

  const directors = useMemo(
    () => castCrew?.crew.filter((c) => c.job === "Director" || c.job === "Creator") ?? [],
    [castCrew]
  );

  const mediaTypeForApi = mediaType === "tv" ? "series" : "movie";

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
    if (!metadata?.tmdb_id || !castCrew?.cast?.length) return;
    const ids = castCrew.cast
      .filter((m): m is CastMember & { id: number } => typeof m.id === "number")
      .map((m) => m.id)
      .slice(0, 5);
    if (ids.length === 0) return;
    getActorMovieSuggestions(metadata.tmdb_id, mediaTypeForApi, ids, [])
      .then(setActorSuggestions)
      .catch(() => setActorSuggestions([]));
  }, [metadata?.tmdb_id, castCrew?.cast, mediaTypeForApi]);

  useEffect(() => {
    if (!metadata?.tmdb_id) return;
    const displayTitle = metadata.title || "Unbekannt";
    personCacheRef.current.clear();
    extendedInfoRef.current = null;
    preloadTmdbIdRef.current = metadata.tmdb_id;

    getMediaExtendedInfo(
      metadata.tmdb_id,
      mediaTypeForApi,
      displayTitle,
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
      getPersonDetails(pid, displayTitle)
        .then((details) => {
          if (preloadTmdbIdRef.current === metadata.tmdb_id) {
            personCacheRef.current.set(pid, details);
          }
        })
        .catch(() => {});
    });
  }, [metadata?.tmdb_id, metadata?.title, metadata?.release_date, mediaTypeForApi, castCrew?.cast]);

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
      const details = await getPersonDetails(member.id, displayTitle);
      personCacheRef.current.set(member.id, details);
      setPersonOverlay(details);
    } catch (err) {
      console.error("Person details failed:", err);
    } finally {
      setPersonLoading(false);
    }
  }

  async function handleTriviaClick() {
    if (!metadata?.tmdb_id) return;
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
        mediaTypeForApi,
        displayTitle,
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

  if (loading) return <div className="loading-state">Lade Details...</div>;
  if (error) return <div className="empty-state"><p>{error}</p></div>;
  if (!metadata) return <div className="empty-state">Keine Metadaten gefunden.</div>;

  const displayTitle = metadata.title || "Unbekannt";
  const isMovie = mediaType === "movie";

  function formatRuntime(minutes: number): string {
    const h = Math.floor(minutes / 60);
    const m = minutes % 60;
    return h > 0 ? `${h} Std. ${m} Min.` : `${m} Min.`;
  }

  return (
    <div className="view-details">
      {metadata.backdrop_url && (
        <div
          className="details-backdrop"
          style={{ backgroundImage: `url(${metadata.backdrop_url})` }}
        />
      )}
      <div className="details-content">
        <button className="back-button" onClick={() => navigate(-1)}>
          ← Zurück
        </button>
        <div className="details-layout">
          <div className="details-poster">
            {metadata.poster_url ? (
              <img src={metadata.poster_url} alt={displayTitle} />
            ) : (
              <div className="poster-placeholder">{isMovie ? "🎬" : "📺"}</div>
            )}
          </div>
          <div className="details-info">
            <h1>{displayTitle}</h1>

            <div className="details-meta">
              <span className={`media-badge ${isMovie ? "movie" : "series"}`}>
                {isMovie ? "Film" : "Serie"}
              </span>
              {metadata.release_date && (
                <span className="meta-year">{metadata.release_date.split("-")[0]}</span>
              )}
              {metadata.runtime != null && metadata.runtime > 0 && (
                <span className="meta-runtime">{formatRuntime(metadata.runtime)}</span>
              )}
              {metadata.rating != null && (
                <span className="meta-rating">★ {metadata.rating.toFixed(1)}</span>
              )}
            </div>

            {genres.length > 0 && (
              <div className="details-genres">
                {genres.map((g) => (
                  <span key={g} className="genre-tag">{g}</span>
                ))}
              </div>
            )}

            {metadata.overview && (
              <p className="details-overview">{metadata.overview}</p>
            )}

            {directors.length > 0 && (
              <div className="details-crew-line">
                <strong>{isMovie ? "Regie" : "Creator"}:</strong>{" "}
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
                        <img src={member.profile_url} alt={member.name} className="cast-photo" />
                      ) : (
                        <div className="cast-photo-placeholder">👤</div>
                      )}
                      <div className="cast-info">
                        <span className="cast-name">{member.name}</span>
                        {member.character && (
                          <span className="cast-character">{member.character}</span>
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

            {players.length > 0 && (
              <div className="details-actions">
                <p className="settings-hint">
                  Dieses Medium stammt aus Streams und hat keine lokale Datei.
                </p>
              </div>
            )}
          </div>

          {(mediaType === "movie" || mediaType === "tv") && (
            <aside className="details-streams">
              <h3>Streams</h3>
              {streamsLoading ? (
                <p className="streams-loading">Suche Streams…</p>
              ) : streams.length === 0 ? (
                <p className="streams-empty">Keine Streams gefunden.</p>
              ) : (
                <ul className="stream-list">
                  {streams.map((s, i) => {
                    const name = (s.title || `${s.quality}${s.size !== "?" ? ` · ${s.size}` : ""}`).replace(/\n/g, " · ").trim();
                    const displayName = name.length > 30 ? `${name.slice(0, 30)}…` : name;
                    return (
                      <li key={i} className="stream-item">
                        <div className="stream-info">
                          <span className="stream-name" title={name}>
                            {displayName}
                          </span>
                          {s.title && (
                            <span className="stream-meta">
                              {s.quality}{s.size !== "?" ? ` · ${s.size}` : ""}
                            </span>
                          )}
                        </div>
                        <button
                          className="btn-primary btn-sm stream-play-btn"
                          onClick={() => {
                            const vlc = players.find((p) => p.id === "vlc");
                            if (vlc) {
                              openInPlayer("vlc", s.stream_url);
                            } else {
                              window.open(s.stream_url, "_blank");
                            }
                          }}
                        >
                          Play
                        </button>
                      </li>
                    );
                  })}
                </ul>
              )}
            </aside>
          )}
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
