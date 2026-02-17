import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  getMusicAlbum,
  setMusicTrackRating,
} from "../api/commands";
import type { MusicAlbum, MusicTrack } from "../types";

interface MusicPlayerContextValue {
  currentTrack: MusicTrack | null;
  currentAlbum: MusicAlbum | null;
  queue: Array<{ track: MusicTrack; album: MusicAlbum }>;
  isPlaying: boolean;
  play: (track: MusicTrack, album: MusicAlbum, queue?: Array<{ track: MusicTrack; album: MusicAlbum }>) => void;
  playNext: () => void;
  togglePlayPause: () => void;
  setRating: (trackId: string, rating: number) => void;
}

const MusicPlayerContext = createContext<MusicPlayerContextValue | null>(null);

export function MusicPlayerProvider({ children }: { children: React.ReactNode }) {
  const [currentTrack, setCurrentTrack] = useState<MusicTrack | null>(null);
  const [currentAlbum, setCurrentAlbum] = useState<MusicAlbum | null>(null);
  const [queue, setQueue] = useState<Array<{ track: MusicTrack; album: MusicAlbum }>>([]);
  const [isPlaying, setIsPlaying] = useState(false);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  const play = useCallback(
    (
      track: MusicTrack,
      album: MusicAlbum,
      newQueue?: Array<{ track: MusicTrack; album: MusicAlbum }>
    ) => {
      setCurrentTrack(track);
      setCurrentAlbum(album);
      if (newQueue) setQueue(newQueue);
      setIsPlaying(true);
      if (audioRef.current) {
        audioRef.current.src = convertFileSrc(track.file_path);
        audioRef.current.play().catch(() => {});
      }
    },
    []
  );

  const playNext = useCallback(() => {
    if (queue.length === 0 || !currentTrack) return;
    const idx = queue.findIndex((q) => q.track.id === currentTrack.id);
    const next = queue[idx + 1];
    if (next) {
      play(next.track, next.album, queue);
    }
  }, [queue, currentTrack, play]);

  const togglePlayPause = useCallback(() => {
    if (!audioRef.current) return;
    if (isPlaying) {
      audioRef.current.pause();
    } else {
      audioRef.current.play().catch(() => {});
    }
    setIsPlaying(!isPlaying);
  }, [isPlaying]);

  const setRating = useCallback(async (trackId: string, rating: number) => {
    try {
      await setMusicTrackRating(trackId, rating);
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => {
    const audio = new Audio();
    audioRef.current = audio;
    audio.addEventListener("ended", playNext);
    return () => {
      audio.removeEventListener("ended", playNext);
      audioRef.current = null;
    };
  }, [playNext]);

  return (
    <MusicPlayerContext.Provider
      value={{
        currentTrack,
        currentAlbum,
        queue,
        isPlaying,
        play,
        playNext,
        togglePlayPause,
        setRating,
      }}
    >
      {children}
    </MusicPlayerContext.Provider>
  );
}

export function useMusicPlayer() {
  const ctx = useContext(MusicPlayerContext);
  if (!ctx) throw new Error("useMusicPlayer must be used within MusicPlayerProvider");
  return ctx;
}
