CREATE TABLE IF NOT EXISTS media_items (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL UNIQUE,
    file_hash TEXT,
    media_type TEXT NOT NULL CHECK(media_type IN ('movie', 'series')),
    file_size INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS metadata (
    id TEXT PRIMARY KEY,
    media_item_id TEXT NOT NULL,
    tmdb_id INTEGER,
    title TEXT,
    overview TEXT,
    poster_url TEXT,
    backdrop_url TEXT,
    release_date TEXT,
    rating REAL,
    genres TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (media_item_id) REFERENCES media_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS realdebrid_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id TEXT NOT NULL,
    client_secret TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
