use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: String,
    pub title: String,
    pub file_path: String,
    pub file_hash: Option<String>,
    pub media_type: String,
    pub file_size: Option<i64>,
    #[serde(default)]
    pub series_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub id: String,
    pub media_item_id: String,
    pub tmdb_id: Option<i64>,
    pub title: Option<String>,
    pub overview: Option<String>,
    pub poster_url: Option<String>,
    pub backdrop_url: Option<String>,
    pub release_date: Option<String>,
    pub rating: Option<f64>,
    pub runtime: Option<i64>,
    pub genres: Option<String>,
    pub cast_crew: Option<String>,
    pub raw_response: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealDebridToken {
    pub id: i64,
    pub client_id: String,
    pub client_secret: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: String,
    pub created_at: String,
}

pub async fn upsert_media_item(pool: &DbPool, item: &MediaItem) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO media_items (id, title, file_path, file_hash, media_type, file_size, series_name, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'), datetime('now'))
         ON CONFLICT(file_path) DO UPDATE SET
           title = excluded.title,
           file_hash = excluded.file_hash,
           media_type = excluded.media_type,
           file_size = excluded.file_size,
           updated_at = datetime('now')"
    )
    .bind(&item.id)
    .bind(&item.title)
    .bind(&item.file_path)
    .bind(&item.file_hash)
    .bind(&item.media_type)
    .bind(&item.file_size)
    .bind(&item.series_name)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_items_without_metadata(pool: &DbPool) -> Result<Vec<MediaItem>, String> {
    let rows = sqlx::query(
        "SELECT m.id, m.title, m.file_path, m.file_hash, m.media_type, m.file_size, m.series_name, m.created_at, m.updated_at
         FROM media_items m
         LEFT JOIN metadata meta ON meta.media_item_id = m.id
         WHERE meta.id IS NULL
         ORDER BY m.title"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let items = rows.iter().map(|row| MediaItem {
        id: row.get("id"),
        title: row.get("title"),
        file_path: row.get("file_path"),
        file_hash: row.get("file_hash"),
        media_type: row.get("media_type"),
        file_size: row.get("file_size"),
        series_name: row.get("series_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(items)
}

pub async fn get_all_media_items(pool: &DbPool) -> Result<Vec<MediaItem>, String> {
    let rows = sqlx::query(
        "SELECT id, title, file_path, file_hash, media_type, file_size, series_name, created_at, updated_at
         FROM media_items ORDER BY title"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let items = rows.iter().map(|row| MediaItem {
        id: row.get("id"),
        title: row.get("title"),
        file_path: row.get("file_path"),
        file_hash: row.get("file_hash"),
        media_type: row.get("media_type"),
        file_size: row.get("file_size"),
        series_name: row.get("series_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(items)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemWithMeta {
    pub item: MediaItem,
    pub metadata: Option<MediaMetadata>,
}

pub async fn get_all_media_items_with_metadata(pool: &DbPool) -> Result<Vec<LibraryItemWithMeta>, String> {
    let rows = sqlx::query(
        "SELECT m.id, m.title, m.file_path, m.file_hash, m.media_type, m.file_size, m.series_name,
                m.created_at, m.updated_at,
                meta.id as meta_id, meta.media_item_id, meta.tmdb_id, meta.title as meta_title,
                meta.overview, meta.poster_url, meta.backdrop_url, meta.release_date,
                meta.rating, meta.runtime, meta.genres, meta.cast_crew, meta.raw_response,
                meta.created_at as meta_created, meta.updated_at as meta_updated
         FROM media_items m
         LEFT JOIN metadata meta ON meta.media_item_id = m.id
         ORDER BY m.title"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let items = rows.iter().map(|row| {
        let item = MediaItem {
            id: row.get("id"),
            title: row.get("title"),
            file_path: row.get("file_path"),
            file_hash: row.get("file_hash"),
            media_type: row.get("media_type"),
            file_size: row.get("file_size"),
            series_name: row.get("series_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        let meta_id: Option<String> = row.get("meta_id");
        let metadata = meta_id.map(|_| MediaMetadata {
            id: row.get("meta_id"),
            media_item_id: row.get("media_item_id"),
            tmdb_id: row.get("tmdb_id"),
            title: row.get("meta_title"),
            overview: row.get("overview"),
            poster_url: row.get("poster_url"),
            backdrop_url: row.get("backdrop_url"),
            release_date: row.get("release_date"),
            rating: row.get("rating"),
            runtime: row.get("runtime"),
            genres: row.get("genres"),
            cast_crew: row.get("cast_crew"),
            raw_response: row.get("raw_response"),
            created_at: row.get("meta_created"),
            updated_at: row.get("meta_updated"),
        });
        LibraryItemWithMeta { item, metadata }
    }).collect();

    Ok(items)
}

pub async fn get_media_item_by_file_path(pool: &DbPool, file_path: &str) -> Result<Option<MediaItem>, String> {
    let row = sqlx::query(
        "SELECT id, title, file_path, file_hash, media_type, file_size, series_name, created_at, updated_at
         FROM media_items WHERE file_path = ?1"
    )
    .bind(file_path)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|row| MediaItem {
        id: row.get("id"),
        title: row.get("title"),
        file_path: row.get("file_path"),
        file_hash: row.get("file_hash"),
        media_type: row.get("media_type"),
        file_size: row.get("file_size"),
        series_name: row.get("series_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

pub async fn get_media_item_by_id(pool: &DbPool, id: &str) -> Result<Option<MediaItem>, String> {
    let row = sqlx::query(
        "SELECT id, title, file_path, file_hash, media_type, file_size, series_name, created_at, updated_at
         FROM media_items WHERE id = ?1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|row| MediaItem {
        id: row.get("id"),
        title: row.get("title"),
        file_path: row.get("file_path"),
        file_hash: row.get("file_hash"),
        media_type: row.get("media_type"),
        file_size: row.get("file_size"),
        series_name: row.get("series_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

pub async fn update_media_classification(
    pool: &DbPool,
    id: &str,
    media_type: &str,
    series_name: Option<&str>,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE media_items SET media_type = ?1, series_name = ?2, updated_at = datetime('now')
         WHERE id = ?3"
    )
    .bind(media_type)
    .bind(series_name)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn delete_media_item(pool: &DbPool, id: &str) -> Result<(), String> {
    sqlx::query("DELETE FROM media_items WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn upsert_metadata(pool: &DbPool, meta: &MediaMetadata) -> Result<(), String> {
    let existing = get_metadata_for_item(pool, &meta.media_item_id).await?;
    if let Some(_existing_meta) = existing {
        sqlx::query(
            "UPDATE metadata SET
               tmdb_id = ?1, title = ?2, overview = ?3, poster_url = ?4, backdrop_url = ?5,
               release_date = ?6, rating = ?7, runtime = ?8, genres = ?9, cast_crew = ?10,
               raw_response = ?11, updated_at = datetime('now')
             WHERE media_item_id = ?12"
        )
        .bind(meta.tmdb_id)
        .bind(&meta.title)
        .bind(&meta.overview)
        .bind(&meta.poster_url)
        .bind(&meta.backdrop_url)
        .bind(&meta.release_date)
        .bind(meta.rating)
        .bind(meta.runtime)
        .bind(&meta.genres)
        .bind(&meta.cast_crew)
        .bind(&meta.raw_response)
        .bind(&meta.media_item_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        sqlx::query(
            "INSERT INTO metadata (id, media_item_id, tmdb_id, title, overview, poster_url, backdrop_url, release_date, rating, runtime, genres, cast_crew, raw_response, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'), datetime('now'))"
        )
        .bind(&meta.id)
        .bind(&meta.media_item_id)
        .bind(meta.tmdb_id)
        .bind(&meta.title)
        .bind(&meta.overview)
        .bind(&meta.poster_url)
        .bind(&meta.backdrop_url)
        .bind(&meta.release_date)
        .bind(meta.rating)
        .bind(meta.runtime)
        .bind(&meta.genres)
        .bind(&meta.cast_crew)
        .bind(&meta.raw_response)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn get_metadata_for_item(pool: &DbPool, media_item_id: &str) -> Result<Option<MediaMetadata>, String> {
    let row = sqlx::query(
        "SELECT id, media_item_id, tmdb_id, title, overview, poster_url, backdrop_url,
                release_date, rating, runtime, genres, cast_crew, raw_response,
                created_at, updated_at
         FROM metadata WHERE media_item_id = ?1
         ORDER BY updated_at DESC LIMIT 1"
    )
    .bind(media_item_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|row| MediaMetadata {
        id: row.get("id"),
        media_item_id: row.get("media_item_id"),
        tmdb_id: row.get("tmdb_id"),
        title: row.get("title"),
        overview: row.get("overview"),
        poster_url: row.get("poster_url"),
        backdrop_url: row.get("backdrop_url"),
        release_date: row.get("release_date"),
        rating: row.get("rating"),
        runtime: row.get("runtime"),
        genres: row.get("genres"),
        cast_crew: row.get("cast_crew"),
        raw_response: row.get("raw_response"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

pub async fn save_setting(pool: &DbPool, key: &str, value: &str) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value"
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn load_all_settings(pool: &DbPool) -> Result<std::collections::HashMap<String, String>, String> {
    let rows = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut map = std::collections::HashMap::new();
    for row in rows {
        map.insert(row.get("key"), row.get("value"));
    }
    Ok(map)
}

pub async fn save_rd_token(pool: &DbPool, token: &RealDebridToken) -> Result<(), String> {
    sqlx::query("DELETE FROM realdebrid_tokens")
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO realdebrid_tokens (client_id, client_secret, access_token, refresh_token, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )
    .bind(&token.client_id)
    .bind(&token.client_secret)
    .bind(&token.access_token)
    .bind(&token.refresh_token)
    .bind(&token.expires_at)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn load_rd_token(pool: &DbPool) -> Result<Option<RealDebridToken>, String> {
    let row = sqlx::query(
        "SELECT id, client_id, client_secret, access_token, refresh_token, expires_at, created_at
         FROM realdebrid_tokens ORDER BY id DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|row| RealDebridToken {
        id: row.get("id"),
        client_id: row.get("client_id"),
        client_secret: row.get("client_secret"),
        access_token: row.get("access_token"),
        refresh_token: row.get("refresh_token"),
        expires_at: row.get("expires_at"),
        created_at: row.get("created_at"),
    }))
}

// ── Discover ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverList {
    pub id: String,
    pub list_type: String,
    pub provider: Option<String>,
    pub country: Option<String>,
    pub title: String,
    pub created_at: String,
    pub items: Vec<DiscoverItemRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverItemRow {
    pub id: String,
    pub list_id: String,
    pub media_type: String,
    pub external_id: Option<String>,
    pub tmdb_id: Option<i64>,
    pub title: String,
    pub year: Option<i32>,
    pub rating: Option<f64>,
    pub poster_url: Option<String>,
    pub overview: Option<String>,
    pub provider: Option<String>,
    pub sort_order: i32,
}

pub async fn clear_discover_data(pool: &DbPool) -> Result<(), String> {
    sqlx::query("DELETE FROM discover_items")
        .execute(pool).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM discover_lists")
        .execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn insert_discover_list(
    pool: &DbPool,
    id: &str,
    list_type: &str,
    provider: Option<&str>,
    country: &str,
    title: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO discover_lists (id, list_type, provider, country, title)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )
    .bind(id)
    .bind(list_type)
    .bind(provider)
    .bind(country)
    .bind(title)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn insert_discover_item(
    pool: &DbPool,
    id: &str,
    list_id: &str,
    media_type: &str,
    external_id: Option<&str>,
    tmdb_id: Option<i64>,
    title: &str,
    year: Option<i32>,
    rating: Option<f64>,
    poster_url: Option<&str>,
    overview: Option<&str>,
    provider: Option<&str>,
    raw_json: &str,
    sort_order: i32,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO discover_items (id, list_id, media_type, external_id, tmdb_id, title, year, rating, poster_url, overview, provider, raw_json, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
    )
    .bind(id)
    .bind(list_id)
    .bind(media_type)
    .bind(external_id)
    .bind(tmdb_id)
    .bind(title)
    .bind(year)
    .bind(rating)
    .bind(poster_url)
    .bind(overview)
    .bind(provider)
    .bind(raw_json)
    .bind(sort_order)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_discover_lists(pool: &DbPool) -> Result<Vec<DiscoverList>, String> {
    let list_rows = sqlx::query(
        "SELECT id, list_type, provider, country, title, created_at
         FROM discover_lists ORDER BY rowid"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut lists = Vec::new();

    for lr in &list_rows {
        let list_id: String = lr.get("id");

        let item_rows = sqlx::query(
            "SELECT id, list_id, media_type, external_id, tmdb_id, title, year, rating, poster_url, overview, provider, sort_order
             FROM discover_items WHERE list_id = ?1 ORDER BY sort_order"
        )
        .bind(&list_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

        let items: Vec<DiscoverItemRow> = item_rows.iter().map(|r| DiscoverItemRow {
            id: r.get("id"),
            list_id: r.get("list_id"),
            media_type: r.get("media_type"),
            external_id: r.get("external_id"),
            tmdb_id: r.get("tmdb_id"),
            title: r.get("title"),
            year: r.get("year"),
            rating: r.get("rating"),
            poster_url: r.get("poster_url"),
            overview: r.get("overview"),
            provider: r.get("provider"),
            sort_order: r.get("sort_order"),
        }).collect();

        lists.push(DiscoverList {
            id: list_id,
            list_type: lr.get("list_type"),
            provider: lr.get("provider"),
            country: lr.get("country"),
            title: lr.get("title"),
            created_at: lr.get("created_at"),
            items,
        });
    }

    Ok(lists)
}

// ── AI Recommendations Cache ──

pub async fn get_ai_recommendations_cached(
    pool: &DbPool,
    preset: &str,
) -> Result<Option<DiscoverList>, String> {
    let row = sqlx::query(
        "SELECT data_json FROM ai_recommendations_cache WHERE preset = ?1"
    )
    .bind(preset)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.and_then(|r| {
        let json: String = r.get("data_json");
        serde_json::from_str(&json).ok()
    }))
}

pub async fn save_ai_recommendations_cache(
    pool: &DbPool,
    preset: &str,
    list: &DiscoverList,
) -> Result<(), String> {
    let json = serde_json::to_string(list).map_err(|e| e.to_string())?;
    sqlx::query(
        "INSERT INTO ai_recommendations_cache (preset, data_json, updated_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(preset) DO UPDATE SET data_json = excluded.data_json, updated_at = datetime('now')"
    )
    .bind(preset)
    .bind(&json)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
