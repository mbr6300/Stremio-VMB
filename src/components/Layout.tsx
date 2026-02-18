import { useState, useCallback, useEffect } from "react";
import { NavLink, Outlet } from "react-router-dom";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useScan } from "../context/ScanContext";
import { StreamsIcon, LibraryIcon, MusicIcon, SettingsIcon, FullscreenIcon, FullscreenExitIcon } from "./NavIcons";
import MusicPlayerBar from "./MusicPlayerBar";

export default function Layout() {
  const { isScanning, isFetchingMetadata } = useScan();
  const [isFullscreen, setIsFullscreen] = useState(false);

  const toggleFullscreen = useCallback(async () => {
    if (isTauri()) {
      try {
        const win = getCurrentWindow();
        const next = !(await win.isFullscreen());
        await win.setFullscreen(next);
        setIsFullscreen(next);
      } catch (err) {
        console.error("Fullscreen toggle failed:", err);
      }
    } else {
      if (!document.fullscreenElement) {
        await document.documentElement.requestFullscreen();
        setIsFullscreen(true);
      } else {
        await document.exitFullscreen();
        setIsFullscreen(false);
      }
    }
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

  return (
    <div className="app-layout">
      {(isScanning || isFetchingMetadata) && (
        <div className="scan-banner">
          {isScanning ? "Bibliothek wird gescannt…" : "Metadaten werden geladen…"}
        </div>
      )}
      <nav className="sidebar">
        <NavLink to="/" end className="sidebar-brand">
          <img src="/logo.png" alt="Stremio VMB" className="sidebar-logo" />
        </NavLink>
        <ul className="sidebar-nav">
          <li>
            <NavLink to="/" end>
              <span className="nav-icon"><StreamsIcon /></span>
              Streams
            </NavLink>
          </li>
          <li>
            <NavLink to="/library">
              <span className="nav-icon"><LibraryIcon /></span>
              Bibliothek
            </NavLink>
          </li>
          <li>
            <NavLink to="/music">
              <span className="nav-icon"><MusicIcon /></span>
              Musik
            </NavLink>
          </li>
          <li>
            <NavLink to="/settings">
              <span className="nav-icon"><SettingsIcon /></span>
              Einstellungen
            </NavLink>
          </li>
        </ul>
      </nav>
      <main className={`main-content ${isScanning || isFetchingMetadata ? "has-scan-banner" : ""}`}>
        <header className="main-header">
          <div className="main-header-spacer" />
          <button
            type="button"
            className="fullscreen-btn"
            onClick={toggleFullscreen}
            title={isFullscreen ? "Vollbild beenden" : "Vollbild"}
            aria-label={isFullscreen ? "Vollbild beenden" : "Vollbild"}
          >
            {isFullscreen ? <FullscreenExitIcon /> : <FullscreenIcon />}
          </button>
        </header>
        <div className="main-content-inner">
          <Outlet />
        </div>
      </main>
      <MusicPlayerBar />
    </div>
  );
}
