CREATE TABLE IF NOT EXISTS discover_lists (
    id TEXT PRIMARY KEY,
    list_type TEXT NOT NULL CHECK(list_type IN ('imdb_top', 'streaming_popular')),
    provider TEXT,
    country TEXT,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS discover_items (
    id TEXT PRIMARY KEY,
    list_id TEXT NOT NULL,
    media_type TEXT NOT NULL CHECK(media_type IN ('movie', 'tv')),
    external_id TEXT,
    tmdb_id INTEGER,
    title TEXT NOT NULL,
    year INTEGER,
    rating REAL,
    poster_url TEXT,
    overview TEXT,
    provider TEXT,
    raw_json TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (list_id) REFERENCES discover_lists(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_discover_items_list ON discover_items(list_id, sort_order);
