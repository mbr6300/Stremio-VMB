export interface PathCheckResult {
  path: string;
  exists: boolean;
  is_directory: boolean;
  files_found: number;
  sample_files: string[];
  sample_all: string[];
  subdirs: string[];
  error: string | null;
}

export interface MediaItem {
  id: string;
  title: string;
  file_path: string;
  file_hash: string | null;
  media_type: "movie" | "series";
  file_size: number | null;
  series_name?: string | null;
  created_at: string;
  updated_at: string;
}

export interface MediaMetadata {
  id: string;
  media_item_id: string;
  tmdb_id: number | null;
  title: string | null;
  overview: string | null;
  poster_url: string | null;
  backdrop_url: string | null;
  release_date: string | null;
  rating: number | null;
  runtime: number | null;
  genres: string | null;
  cast_crew: string | null;
  raw_response: string | null;
  created_at: string;
  updated_at: string;
}

export interface CastMember {
  id?: number | null;
  name: string;
  character: string | null;
  profile_url: string | null;
}

export interface PersonDetails {
  id: number;
  name: string;
  biography: string | null;
  profile_url: string | null;
  birthday: string | null;
  deathday: string | null;
  place_of_birth: string | null;
  known_for_department: string | null;
  age: number | null;
  anecdotes: string[];
  height: string | null;
  partner_status: string | null;
  children: string | null;
}

export interface MediaExtendedInfo {
  tagline: string | null;
  trivia_facts: string[];
}

export interface ActorMovieSuggestion {
  tmdb_id: number;
  title: string;
  year: number | null;
  poster_url: string | null;
  media_type: string;
  match_reason: string;
}

export interface CrewMember {
  name: string;
  job: string;
  department: string;
  profile_url: string | null;
}

export interface CastCrew {
  cast: CastMember[];
  crew: CrewMember[];
}

export interface MetadataSearchResult {
  tmdb_id: number;
  title: string;
  overview: string;
  poster_url: string | null;
  release_date: string | null;
  rating: number | null;
}

export interface RdUserInfo {
  username: string;
  email: string;
  premium: boolean;
  expiration: string;
}

export interface RdStatusInfo {
  status: string;
  user: RdUserInfo | null;
}

export interface ApiServiceStatus {
  configured: boolean;
  connected: boolean;
  message: string;
}

export interface ApiConfigurationStatus {
  tmdb: ApiServiceStatus;
  perplexity: ApiServiceStatus;
}

export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  interval: number;
  expires_in: number;
  verification_url: string;
  direct_verification_url: string;
}

export interface RdStreamLink {
  title: string;
  quality: string;
  size: string;
  stream_url: string;
}

export interface DebridSearchResult {
  matched_title: string | null;
  matched_year: number | null;
  matched_tmdb_id: number | null;
  media_type: string;
  streams: RdStreamLink[];
}

export interface UnrestrictedLink {
  id: string;
  filename: string;
  filesize: number;
  link: string;
  host: string;
  download: string;
  streamable: number;
}

export interface StreamSearchResult {
  tmdb_id: number;
  media_type: string;
  title: string;
  year: number | null;
  rating: number | null;
  poster_url: string | null;
  overview: string;
  genre_ids: number[];
}

export interface TmdbGenre {
  id: number;
  name: string;
  media_type: string;
}

export interface DiscoverItem {
  id: string;
  list_id: string;
  media_type: "movie" | "tv";
  external_id: string | null;
  tmdb_id: number | null;
  title: string;
  year: number | null;
  rating: number | null;
  poster_url: string | null;
  overview: string | null;
  provider: string | null;
  sort_order: number;
}

export interface DiscoverList {
  id: string;
  list_type: "imdb_top" | "streaming_popular";
  provider: string | null;
  country: string | null;
  title: string;
  created_at: string;
  items: DiscoverItem[];
}

export interface ExternalPlayer {
  id: string;
  name: string;
  path: string;
  installed: boolean;
}

export interface Settings {
  [key: string]: string;
}

export interface LibraryItemWithMeta {
  item: MediaItem;
  metadata: MediaMetadata | null;
}

export interface MovieGroup {
  key: string;
  representative: LibraryItemWithMeta;
  entries: LibraryItemWithMeta[];
}

export interface SeriesCluster {
  key: string;
  title: string;
  episodes: LibraryItemWithMeta[];
  metadata: MediaMetadata | null;
}

export interface EpisodeInfo {
  season: number;
  episode: number;
  display: string;
}

export interface MusicAlbum {
  id: string;
  artist: string;
  album_title: string;
  year: number | null;
  cover_path: string | null;
  music_path: string;
  created_at: string;
  updated_at: string;
}

export interface MusicTrack {
  id: string;
  album_id: string;
  title: string;
  track_number: number | null;
  duration: number | null;
  file_path: string;
  file_hash: string | null;
  created_at: string;
}

export interface MusicPlaylist {
  id: string;
  name: string;
  created_at: string;
}

export interface MusicTrackWithAlbum extends MusicTrack {
  album?: MusicAlbum;
}
