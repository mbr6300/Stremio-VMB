import { BrowserRouter, Routes, Route } from "react-router-dom";
import { ScanProvider } from "./context/ScanContext";
import { LibraryProvider } from "./context/LibraryContext";
import { MusicPlayerProvider } from "./context/MusicPlayerContext";
import Layout from "./components/Layout";
import Discover from "./views/Discover";
import Library from "./views/Library";
import LibrarySeries from "./views/LibrarySeries";
import Details from "./views/Details";
import DiscoverDetail from "./views/DiscoverDetail";
import Player from "./views/Player";
import Settings from "./views/Settings";
import MusicLibrary from "./views/MusicLibrary";
import MusicAlbumDetail from "./views/MusicAlbumDetail";
import MusicNowPlaying from "./views/MusicNowPlaying";

export default function App() {
  return (
    <BrowserRouter>
      <ScanProvider>
        <LibraryProvider>
          <MusicPlayerProvider>
          <Routes>
            <Route element={<Layout />}>
              <Route path="/" element={<Discover />} />
              <Route path="/library" element={<Library />} />
              <Route path="/library/series/:seriesId" element={<LibrarySeries />} />
              <Route path="/details/:id" element={<Details />} />
              <Route path="/discover-detail/:tmdbId/:mediaType" element={<DiscoverDetail />} />
              <Route path="/player/:id" element={<Player />} />
              <Route path="/music" element={<MusicLibrary />} />
              <Route path="/music/album/:albumId" element={<MusicAlbumDetail />} />
              <Route path="/music/now-playing" element={<MusicNowPlaying />} />
              <Route path="/settings" element={<Settings />} />
            </Route>
          </Routes>
          </MusicPlayerProvider>
        </LibraryProvider>
      </ScanProvider>
    </BrowserRouter>
  );
}
