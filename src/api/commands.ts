import { invoke } from "@tauri-apps/api/core";
import type {
  PathCheckResult,
  MediaItem,
  LibraryItemWithMeta,
  RdStreamLink,
  DebridSearchResult,
  MediaMetadata,
  MetadataSearchResult,
  PersonDetails,
  MediaExtendedInfo,
  ActorMovieSuggestion,
  DiscoverList,
  StreamSearchResult,
  TmdbGenre,
  DeviceCodeResponse,
  UnrestrictedLink,
  RdUserInfo,
  RdStatusInfo,
  ExternalPlayer,
  Settings,
  MusicAlbum,
  MusicTrack,
  MusicPlaylist,
} from "../types";

export async function checkMediaPath(path: string): Promise<PathCheckResult> {
  return invoke("check_media_path", { path });
}

export async function scanMediaDirs(paths: string[]): Promise<MediaItem[]> {
  return invoke("scan_media_dirs", { paths });
}

export async function scanMediaDirsProgressive(paths: string[]): Promise<void> {
  return invoke("scan_media_dirs_progressive", { paths });
}

export async function getLibrary(): Promise<MediaItem[]> {
  return invoke("get_library");
}

export async function getLibraryWithMetadata(): Promise<LibraryItemWithMeta[]> {
  return invoke("get_library_with_metadata");
}

export async function getMediaItem(id: string): Promise<MediaItem | null> {
  return invoke("get_media_item", { id });
}

export async function deleteMediaItem(id: string): Promise<void> {
  return invoke("delete_media_item", { id });
}

export async function improveClassificationsWithPerplexity(): Promise<number> {
  return invoke("improve_classifications_with_perplexity");
}

export async function saveSettings(settings: Settings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function loadSettings(): Promise<Settings> {
  return invoke("load_settings");
}

export async function fetchMetadata(
  mediaItemId: string,
  year?: number
): Promise<MediaMetadata | null> {
  return invoke("fetch_metadata", { mediaItemId, year: year ?? null });
}

export async function fetchMetadataByTmdbId(
  mediaItemId: string,
  tmdbId: number,
  mediaType: string
): Promise<MediaMetadata | null> {
  return invoke("fetch_metadata_by_tmdb_id", { mediaItemId, tmdbId, mediaType });
}

export async function getMetadata(
  mediaItemId: string
): Promise<MediaMetadata | null> {
  return invoke("get_metadata", { mediaItemId });
}

export async function fetchMetadataBatch(): Promise<number> {
  return invoke("fetch_metadata_batch");
}

export async function searchMetadata(
  title: string,
  year: number | null,
  mediaType: string
): Promise<MetadataSearchResult[]> {
  return invoke("search_metadata", { title, year, mediaType });
}

export async function getPersonDetails(
  personId: number,
  knownFor?: string | null
): Promise<PersonDetails> {
  return invoke("get_person_details", {
    person_id: personId,
    known_for: knownFor ?? null,
  });
}

export async function getMediaExtendedInfo(
  tmdbId: number,
  mediaType: string,
  title?: string | null,
  year?: string | null
): Promise<MediaExtendedInfo> {
  return invoke("get_media_extended_info", {
    tmdb_id: tmdbId,
    media_type: mediaType,
    title: title ?? null,
    year: year ?? null,
  });
}

export async function getActorMovieSuggestions(
  tmdbId: number,
  mediaType: string,
  actorIds: number[],
  directorIds?: number[]
): Promise<ActorMovieSuggestion[]> {
  return invoke("get_actor_movie_suggestions", {
    tmdb_id: tmdbId,
    media_type: mediaType,
    actor_ids: actorIds,
    director_ids: directorIds ?? [],
  });
}

export async function searchStreams(
  query: string,
  media_type: string,
  genre_id?: number | null
): Promise<StreamSearchResult[]> {
  return invoke("search_streams", {
    query,
    media_type,
    genre_id: genre_id ?? null,
  });
}

export async function getTmdbGenres(): Promise<TmdbGenre[]> {
  return invoke("get_tmdb_genres");
}

export async function refreshDiscoverLists(country?: string): Promise<DiscoverList[]> {
  return invoke("refresh_discover_lists", { country: country ?? null });
}

export async function getDiscoverLists(): Promise<DiscoverList[]> {
  return invoke("get_discover_lists");
}

export async function getAiRecommendationsList(
  preset: string,
  forceRefresh?: boolean
): Promise<DiscoverList> {
  return invoke("get_ai_recommendations_list", {
    preset,
    force_refresh: forceRefresh ?? false,
  });
}

export async function searchRdStreams(
  title: string,
  year: number | null,
  mediaType: string,
  tmdbId?: number | null
): Promise<RdStreamLink[]> {
  return invoke("search_rd_streams", {
    title,
    year: year ?? null,
    mediaType,
    tmdbId: tmdbId ?? null,
  });
}

export async function searchDebridStreams(
  title: string,
  mediaType: string,
  year?: number | null
): Promise<DebridSearchResult> {
  return invoke("search_debrid_streams", {
    title,
    mediaType,
    year: year ?? null,
  });
}

export async function rdGetDeviceCode(): Promise<DeviceCodeResponse> {
  return invoke("rd_get_device_code");
}

export async function rdPollCredentials(deviceCode: string): Promise<boolean> {
  return invoke("rd_poll_credentials", { deviceCode });
}

export async function rdSaveApiKey(apiKey: string): Promise<RdUserInfo> {
  return invoke("rd_save_api_key", { apiKey });
}

export async function rdUnrestrictLink(link: string): Promise<UnrestrictedLink> {
  return invoke("rd_unrestrict_link", { link });
}

export async function rdGetStatus(): Promise<RdStatusInfo | null> {
  return invoke("rd_get_status");
}

export async function detectPlayers(): Promise<ExternalPlayer[]> {
  return invoke("detect_players");
}

export async function openInPlayer(playerId: string, filePath: string): Promise<void> {
  return invoke("open_in_player", { playerId, filePath });
}

export async function openInDefaultPlayer(filePath: string): Promise<void> {
  return invoke("open_in_default_player", { filePath });
}

export async function checkMusicPath(path: string): Promise<PathCheckResult> {
  return invoke("check_music_path", { path });
}

export async function scanMusicDirsProgressive(paths: string[]): Promise<void> {
  return invoke("scan_music_dirs_progressive", { paths });
}

export async function getMusicLibrary(): Promise<MusicAlbum[]> {
  return invoke("get_music_library");
}

export async function getMusicAlbum(
  albumId: string
): Promise<{ album: MusicAlbum; tracks: MusicTrack[] } | null> {
  const result = await invoke<[MusicAlbum, MusicTrack[]] | null>("get_music_album", {
    albumId,
  });
  if (!result || !Array.isArray(result)) return null;
  const [album, tracks] = result;
  return { album, tracks: tracks ?? [] };
}

export async function getMusicGenres(): Promise<string[]> {
  return invoke("get_music_genres");
}

export async function fetchMusicAlbumMetadata(
  albumId: string
): Promise<MusicAlbum | null> {
  return invoke("fetch_music_album_metadata", { albumId });
}

export async function fetchMusicMetadataBatch(): Promise<number> {
  return invoke("fetch_music_metadata_batch");
}

export async function getRandomMusicTracks(
  limit?: number
): Promise<Array<{ track: MusicTrack; album: MusicAlbum }>> {
  const result = await invoke<[MusicTrack, MusicAlbum][]>("get_random_music_tracks", {
    limit: limit ?? 100,
  });
  return result.map(([track, album]) => ({ track, album }));
}

export async function getMusicPlaylistById(
  playlistId: string
): Promise<MusicPlaylist | null> {
  return invoke("get_music_playlist_by_id", { playlistId });
}

export async function getMusicPlaylists(): Promise<MusicPlaylist[]> {
  return invoke("get_music_playlists");
}

export async function createMusicPlaylist(name: string): Promise<MusicPlaylist> {
  return invoke("create_music_playlist", { name });
}

export async function renameMusicPlaylist(
  playlistId: string,
  name: string
): Promise<void> {
  return invoke("rename_music_playlist", { playlistId, name });
}

export async function deleteMusicPlaylist(playlistId: string): Promise<void> {
  return invoke("delete_music_playlist", { playlistId });
}

export async function getMusicPlaylistTracks(
  playlistId: string
): Promise<Array<{ track: MusicTrack; album: MusicAlbum }>> {
  const result = await invoke<[MusicTrack, MusicAlbum][]>("get_music_playlist_tracks", {
    playlistId,
  });
  return result.map(([track, album]) => ({ track, album }));
}

export async function addTrackToMusicPlaylist(
  playlistId: string,
  trackId: string
): Promise<void> {
  return invoke("add_track_to_music_playlist", { playlistId, trackId });
}

export async function removeTrackFromMusicPlaylist(
  playlistId: string,
  trackId: string
): Promise<void> {
  return invoke("remove_track_from_music_playlist", { playlistId, trackId });
}

export async function createMusicRadioPlaylist(
  trackId: string
): Promise<MusicPlaylist> {
  return invoke("create_music_radio_playlist", { trackId });
}

export async function setMusicTrackRating(
  trackId: string,
  rating: number
): Promise<void> {
  return invoke("set_music_track_rating", { trackId, rating });
}
