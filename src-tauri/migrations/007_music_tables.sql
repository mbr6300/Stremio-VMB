CREATE TABLE IF NOT EXISTS music_albums (
    id TEXT PRIMARY KEY,
    artist TEXT NOT NULL,
    album_title TEXT NOT NULL,
    year INTEGER,
    cover_path TEXT,
    music_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS music_tracks (
    id TEXT PRIMARY KEY,
    album_id TEXT NOT NULL,
    title TEXT NOT NULL,
    track_number INTEGER,
    duration INTEGER,
    file_path TEXT NOT NULL UNIQUE,
    file_hash TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (album_id) REFERENCES music_albums(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS music_playlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS music_playlist_tracks (
    playlist_id TEXT NOT NULL,
    track_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, track_id),
    FOREIGN KEY (playlist_id) REFERENCES music_playlists(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES music_tracks(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS music_track_ratings (
    track_id TEXT PRIMARY KEY,
    rating INTEGER NOT NULL CHECK(rating >= 1 AND rating <= 5),
    FOREIGN KEY (track_id) REFERENCES music_tracks(id) ON DELETE CASCADE
);
