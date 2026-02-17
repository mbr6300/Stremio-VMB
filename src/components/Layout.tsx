import { NavLink, Outlet } from "react-router-dom";
import { useScan } from "../context/ScanContext";
import { StreamsIcon, LibraryIcon, MusicIcon, SettingsIcon } from "./NavIcons";
import MusicPlayerBar from "./MusicPlayerBar";

export default function Layout() {
  const { isScanning, isFetchingMetadata } = useScan();

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
        <div className="main-content-inner">
          <Outlet />
        </div>
      </main>
      <MusicPlayerBar />
    </div>
  );
}
