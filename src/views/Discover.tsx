import React, { useEffect, useState, useRef } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  getDiscoverLists,
  refreshDiscoverLists,
  getAiRecommendationsList,
  searchDebridStreams,
  detectPlayers,
  openInPlayer,
} from "../api/commands";
import DiscoverCard from "../components/DiscoverCard";
import { FullscreenExitIcon, FullscreenIcon } from "../components/NavIcons";
import type { DiscoverList, DebridSearchResult, ExternalPlayer } from "../types";

const DISCOVER_COUNTRIES = ["CH", "DE", "US", "UK"] as const;

const AI_PRESETS = [
  { id: "my_taste", label: "My taste" },
  { id: "70s", label: "Best of 70s" },
  { id: "80s", label: "Best of 80s" },
  { id: "90s", label: "Best of 90s" },
  { id: "00s", label: "Best of 00s" },
  { id: "2010s", label: "Best of 2010s" },
  { id: "action", label: "Action" },
  { id: "comedy", label: "Comedy" },
  { id: "drama", label: "Drama" },
  { id: "thriller", label: "Thriller" },
] as const;

type ViewMode = "empfehlungen" | "suche";
type MediaFilter = "movie" | "tv";

export default function Discover() {
  const [viewMode, setViewMode] = useState<ViewMode>("empfehlungen");
  const [lists, setLists] = useState<DiscoverList[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [selectedCountry, setSelectedCountry] = useState<string>("CH");
  const [openCountryMenuFor, setOpenCountryMenuFor] = useState<string | null>(null);

  const [aiPreset, setAiPreset] = useState<string>("my_taste");
  const [aiList, setAiList] = useState<DiscoverList | null>(null);
  const [aiLoading, setAiLoading] = useState(false);
  const aiScrollRef = useRef<HTMLDivElement>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [mediaFilter, setMediaFilter] = useState<MediaFilter>("movie");
  const [debridResult, setDebridResult] = useState<DebridSearchResult | null>(null);
  const [searching, setSearching] = useState(false);
  const [searchError, setSearchError] = useState("");
  const [players, setPlayers] = useState<ExternalPlayer[]>([]);
  const [isFullscreen, setIsFullscreen] = useState(false);

  useEffect(() => {
    loadLists();
    detectPlayers().then(setPlayers).catch(() => {});
  }, []);

  useEffect(() => {
    if (isTauri()) {
      getCurrentWindow().isFullscreen().then(setIsFullscreen).catch(() => {});
      const unlisten = getCurrentWindow().onResized(() => {
        getCurrentWindow().isFullscreen().then(setIsFullscreen).catch(() => {});
      });
      return () => { unlisten.then((fn) => fn()); };
    }
    const handler = () => setIsFullscreen(!!document.fullscreenElement);
    document.addEventListener("fullscreenchange", handler);
    return () => document.removeEventListener("fullscreenchange", handler);
  }, []);

  useEffect(() => {
    function onDocumentClick(e: MouseEvent) {
      const target = e.target as HTMLElement | null;
      if (target?.closest(".discover-country-picker")) return;
      setOpenCountryMenuFor(null);
    }
    document.addEventListener("mousedown", onDocumentClick);
    return () => document.removeEventListener("mousedown", onDocumentClick);
  }, []);

  useEffect(() => {
    loadAiList(aiPreset, false);
  }, [aiPreset]);

  function scrollAi(direction: "left" | "right") {
    if (!aiScrollRef.current) return;
    const amount = aiScrollRef.current.clientWidth * 0.85;
    aiScrollRef.current.scrollBy({
      left: direction === "left" ? -amount : amount,
      behavior: "smooth",
    });
  }

  async function loadAiList(preset: string, forceRefresh: boolean) {
    try {
      setAiLoading(true);
      if (forceRefresh) setAiList(null);
      const data = await getAiRecommendationsList(preset, forceRefresh);
      setAiList(data);
    } catch (err) {
      console.error("AI-Empfehlungen fehlgeschlagen:", err);
      setAiList(null);
    } finally {
      setAiLoading(false);
    }
  }

  async function loadLists() {
    try {
      setLoading(true);
      const data = await getDiscoverLists();
      if (data.length > 0) {
        setLists(data);
        const country = data[0].country;
        if (country) setSelectedCountry(country);
      }
    } catch (err) {
      console.error("Failed to load discover lists:", err);
    } finally {
      setLoading(false);
    }
  }

  async function handleRefresh(country?: string) {
    const c = country ?? selectedCountry;
    try {
      setRefreshing(true);
      const data = await refreshDiscoverLists(c);
      setLists(data);
      setSelectedCountry(c);
    } catch (err) {
      console.error("Failed to refresh:", err);
    } finally {
      setRefreshing(false);
    }
  }

  function handleCountrySelect(country: string) {
    setSelectedCountry(country);
    setOpenCountryMenuFor(null);
    handleRefresh(country);
  }

  async function handleDebridSearch() {
    if (!searchQuery.trim()) return;
    try {
      setSearching(true);
      setSearchError("");
      setDebridResult(null);
      const result = await searchDebridStreams(
        searchQuery.trim(),
        mediaFilter,
        null
      );
      setDebridResult(result);
    } catch (err) {
      setSearchError(String(err));
      setDebridResult(null);
    } finally {
      setSearching(false);
    }
  }

  async function toggleFullscreen() {
    if (isTauri()) {
      try {
        const win = getCurrentWindow();
        const next = !(await win.isFullscreen());
        await win.setFullscreen(next);
        setIsFullscreen(next);
      } catch (err) {
        console.error("Fullscreen toggle failed:", err);
      }
      return;
    }
    if (!document.fullscreenElement) {
      await document.documentElement.requestFullscreen();
      setIsFullscreen(true);
    } else {
      await document.exitFullscreen();
      setIsFullscreen(false);
    }
  }

  return (
    <div className="view-discover">
      <header className="view-header">
        <div className="discover-header-row">
          <h1>Streams</h1>
          <div className="discover-header-actions">
            {viewMode === "empfehlungen" ? (
              <>
                <button
                  className="btn-secondary discover-action-btn"
                  onClick={() => setViewMode("suche")}
                >
                  Suche
                </button>
                <button
                  className="btn-secondary discover-action-btn"
                  onClick={() => handleRefresh()}
                  disabled={refreshing}
                >
                  {refreshing ? "Aktualisiere…" : "Aktualisieren"}
                </button>
                <button
                  type="button"
                  className="btn-secondary btn-icon discover-action-btn discover-action-icon"
                  onClick={toggleFullscreen}
                  title={isFullscreen ? "Vollbild beenden" : "Vollbild"}
                  aria-label={isFullscreen ? "Vollbild beenden" : "Vollbild"}
                >
                  {isFullscreen ? <FullscreenExitIcon /> : <FullscreenIcon />}
                </button>
              </>
            ) : (
              <>
                <button
                  className="btn-secondary discover-action-btn"
                  onClick={() => setViewMode("empfehlungen")}
                >
                  Empfehlungen
                </button>
                <button
                  type="button"
                  className="btn-secondary btn-icon discover-action-btn discover-action-icon"
                  onClick={toggleFullscreen}
                  title={isFullscreen ? "Vollbild beenden" : "Vollbild"}
                  aria-label={isFullscreen ? "Vollbild beenden" : "Vollbild"}
                >
                  {isFullscreen ? <FullscreenExitIcon /> : <FullscreenIcon />}
                </button>
              </>
            )}
          </div>
        </div>
      </header>

      {viewMode === "empfehlungen" ? (
        <>
          {loading && lists.length === 0 ? (
            <div className="loading-state">Lade Empfehlungen…</div>
          ) : lists.length === 0 ? (
            <div className="empty-state">
              <p>Noch keine Empfehlungen geladen.</p>
              <p>
                Setze einen <strong>TMDb API-Key</strong> in den Einstellungen
                und klicke <strong>Aktualisieren</strong>.
              </p>
            </div>
          ) : (
            <div className="discover-sections">
              <section className="discover-section discover-section-ai">
                <div className="discover-section-header discover-ai-header">
                  <h2 className="discover-section-ai-title">
                    <span className="provider-icon">✨</span>
                    Empfehlungen für dich
                  </h2>
                  <div className="discover-ai-tabs-row">
                    <div className="discover-ai-tabs">
                      {AI_PRESETS.map((p) => (
                        <button
                          key={p.id}
                          type="button"
                          className={`discover-country-tab discover-preset-chip ${aiPreset === p.id ? "active" : ""}`}
                          onClick={() => setAiPreset(p.id)}
                          disabled={aiLoading}
                        >
                          {p.label}
                        </button>
                      ))}
                    </div>
                    <button
                      type="button"
                      className="discover-ai-reload-btn"
                      onClick={() => loadAiList(aiPreset, true)}
                      disabled={aiLoading}
                      title="Empfehlungen neu laden"
                      aria-label="Empfehlungen neu laden"
                    >
                      ↻
                    </button>
                  </div>
                </div>
                {aiLoading ? (
                  <div className="loading-state loading-inline">Lade Empfehlungen…</div>
                ) : aiList && aiList.items.length > 0 ? (
                  <div className="discover-section-row discover-carousel-container">
                    <div className="discover-carousel-wrapper">
                      <div className="discover-carousel discover-carousel-ai" ref={aiScrollRef}>
                        {aiList.items.map((item) => (
                          <DiscoverCard key={item.id} item={item} />
                        ))}
                      </div>
                      <button
                        type="button"
                        className="carousel-btn carousel-btn-left"
                        onClick={() => scrollAi("left")}
                        aria-label="Nach links scrollen"
                      >
                        ‹
                      </button>
                      <button
                        type="button"
                        className="carousel-btn carousel-btn-right"
                        onClick={() => scrollAi("right")}
                        aria-label="Nach rechts scrollen"
                      >
                        ›
                      </button>
                    </div>
                  </div>
                ) : aiList && aiList.items.length === 0 ? (
                  <p className="discover-ai-empty">
                    Keine Empfehlungen. Perplexity- und TMDb-API-Key in Einstellungen setzen.
                    Für „My taste“ Lieblingsfilme angeben.
                  </p>
                ) : null}
              </section>
              {lists.map((list) => (
                <DiscoverSection
                  key={list.id}
                  list={list}
                  selectedCountry={selectedCountry}
                  refreshing={refreshing}
                  isCountryMenuOpen={openCountryMenuFor === list.id}
                  onToggleCountryMenu={() =>
                    setOpenCountryMenuFor((prev) => (prev === list.id ? null : list.id))
                  }
                  onSelectCountry={handleCountrySelect}
                />
              ))}
            </div>
          )}
        </>
      ) : (
        <section className="streams-search-section">
          <h2 className="search-section-title">Suche in Debridio/RealDebrid</h2>
          <p className="settings-hint">
            Nach Titel suchen – durchsucht Debridio und RealDebrid nach
            verfügbaren Streams.
          </p>
          <div className="search-controls">
            <input
              type="text"
              className="search-input search-input-wide discover-search-input"
              placeholder="z.B. Inception, Breaking Bad…"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleDebridSearch()}
            />
            <div className="filter-tabs">
              <button
                className={mediaFilter === "movie" ? "active" : ""}
                onClick={() => setMediaFilter("movie")}
              >
                Filme
              </button>
              <button
                className={mediaFilter === "tv" ? "active" : ""}
                onClick={() => setMediaFilter("tv")}
              >
                Serien
              </button>
            </div>
            <button
              className="btn-primary discover-search-btn"
              onClick={handleDebridSearch}
              disabled={searching || !searchQuery.trim()}
            >
              {searching ? "Suche…" : "Suchen"}
            </button>
          </div>

          {searchError && (
            <div className="path-check-result error" style={{ marginTop: 12 }}>
              {searchError}
            </div>
          )}

          {debridResult && (
            <div className="debrid-search-results">
              <div className="debrid-result-header">
                <h3>
                  {debridResult.matched_title || searchQuery}
                  {debridResult.matched_year && (
                    <span className="debrid-result-year">
                      {" "}({debridResult.matched_year})
                    </span>
                  )}
                </h3>
                <span className="media-badge debrid-badge">
                  {debridResult.media_type === "series" ? "Serie" : "Film"}
                </span>
              </div>
              {debridResult.streams.length === 0 ? (
                <p className="streams-empty">Keine Streams gefunden.</p>
              ) : (
                <ul className="stream-list debrid-stream-list">
                  {debridResult.streams.map((s, i) => {
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
            </div>
          )}
        </section>
      )}
    </div>
  );
}

function DiscoverSection({
  list,
  selectedCountry,
  refreshing,
  isCountryMenuOpen,
  onToggleCountryMenu,
  onSelectCountry,
}: {
  list: DiscoverList;
  selectedCountry: string;
  refreshing: boolean;
  isCountryMenuOpen: boolean;
  onToggleCountryMenu: () => void;
  onSelectCountry: (country: string) => void;
}) {
  const scrollRef = useRef<HTMLDivElement>(null);

  function scroll(direction: "left" | "right") {
    if (!scrollRef.current) return;
    const amount = scrollRef.current.clientWidth * 0.85;
    scrollRef.current.scrollBy({
      left: direction === "left" ? -amount : amount,
      behavior: "smooth",
    });
  }

  if (list.items.length === 0) return null;

  const sectionIcon = getSectionIcon(list);

  return (
    <section className="discover-section">
      <div className="discover-section-header">
        <h2>
          {sectionIcon}
          {list.title}
          <span className="discover-country-picker">
            <button
              type="button"
              className="discover-country-trigger"
              onClick={onToggleCountryMenu}
              disabled={refreshing}
              aria-haspopup="menu"
              aria-expanded={isCountryMenuOpen}
              title="Land wechseln"
            >
              {selectedCountry}
            </button>
            {isCountryMenuOpen && (
              <div className="discover-country-menu" role="menu">
                {DISCOVER_COUNTRIES.map((c) => (
                  <button
                    key={c}
                    type="button"
                    role="menuitem"
                    className={`discover-country-menu-item ${selectedCountry === c ? "active" : ""}`}
                    onClick={() => onSelectCountry(c)}
                    disabled={refreshing}
                  >
                    {c}
                  </button>
                ))}
              </div>
            )}
          </span>
        </h2>
      </div>
      <div className="discover-section-row discover-carousel-container">
        <div className="discover-carousel-wrapper">
          <div className="discover-carousel" ref={scrollRef}>
            {list.items.map((item) => (
              <DiscoverCard key={item.id} item={item} />
            ))}
          </div>
          <button
            type="button"
            className="carousel-btn carousel-btn-left"
            onClick={() => scroll("left")}
            aria-label="Nach links scrollen"
          >
            ‹
          </button>
          <button
            type="button"
            className="carousel-btn carousel-btn-right"
            onClick={() => scroll("right")}
            aria-label="Nach rechts scrollen"
          >
            ›
          </button>
        </div>
      </div>
    </section>
  );
}

const PROVIDER_LOGOS: Record<string, string> = {
  imdb: "https://cdn.simpleicons.org/imdb/F5C518",
  netflix: "https://cdn.simpleicons.org/netflix/E50914",
  disney: "/icons/disneyplus.svg",
  prime: "https://cdn.simpleicons.org/amazonprime/00A8E1",
  apple: "https://cdn.simpleicons.org/appletv/FFFFFF",
};

function getSectionIcon(list: DiscoverList): React.ReactElement | null {
  let src: string | null = null;
  let alt = "";
  if (list.list_type === "imdb_top") {
    src = PROVIDER_LOGOS.imdb;
    alt = "IMDb";
  } else if (list.provider && PROVIDER_LOGOS[list.provider]) {
    src = PROVIDER_LOGOS[list.provider];
    alt = list.provider;
  }
  if (!src) return null;
  return (
    <img
      src={src}
      alt={alt}
      className="provider-icon provider-icon-img"
      width={20}
      height={20}
    />
  );
}
