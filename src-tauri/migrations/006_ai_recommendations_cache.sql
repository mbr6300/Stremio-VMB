CREATE TABLE IF NOT EXISTS ai_recommendations_cache (
    preset TEXT PRIMARY KEY,
    data_json TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
