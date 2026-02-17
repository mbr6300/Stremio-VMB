//! Musik-Datenbank-Operationen.

use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicAlbum {
    pub id: String,
    pub artist: String,
    pub album_title: String,
    pub year: Option<i32>,
    pub cover_path: Option<String>,
    pub music_path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicTrack {
    pub id: String,
    pub album_id: String,
    pub title: String,
    pub track_number: Option<i32>,
    pub duration: Option<i32>,
    pub file_path: String,
    pub file_hash: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicPlaylist {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

pub async fn upsert_music_album(pool: &DbPool, album: &MusicAlbum) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO music_albums (id, artist, album_title, year, cover_path, music_path, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'))
         ON CONFLICT(id) DO UPDATE SET
           artist = excluded.artist,
           album_title = excluded.album_title,
           year = excluded.year,
           cover_path = excluded.cover_path,
           music_path = excluded.music_path,
           updated_at = datetime('now')"
    )
    .bind(&album.id)
    .bind(&album.artist)
    .bind(&album.album_title)
    .bind(&album.year)
    .bind(&album.cover_path)
    .bind(&album.music_path)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn upsert_music_track(pool: &DbPool, track: &MusicTrack) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO music_tracks (id, album_id, title, track_number, duration, file_path, file_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))
         ON CONFLICT(file_path) DO UPDATE SET
           album_id = excluded.album_id,
           title = excluded.title,
           track_number = excluded.track_number,
           duration = excluded.duration,
           file_hash = excluded.file_hash"
    )
    .bind(&track.id)
    .bind(&track.album_id)
    .bind(&track.title)
    .bind(&track.track_number)
    .bind(&track.duration)
    .bind(&track.file_path)
    .bind(&track.file_hash)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_all_music_albums(pool: &DbPool) -> Result<Vec<MusicAlbum>, String> {
    let rows = sqlx::query(
        "SELECT id, artist, album_title, year, cover_path, music_path, created_at, updated_at
         FROM music_albums ORDER BY artist, album_title"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|row| MusicAlbum {
        id: row.get("id"),
        artist: row.get("artist"),
        album_title: row.get("album_title"),
        year: row.get("year"),
        cover_path: row.get("cover_path"),
        music_path: row.get("music_path"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect())
}

pub async fn get_music_album(pool: &DbPool, album_id: &str) -> Result<Option<MusicAlbum>, String> {
    let row = sqlx::query(
        "SELECT id, artist, album_title, year, cover_path, music_path, created_at, updated_at
         FROM music_albums WHERE id = ?1"
    )
    .bind(album_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(row.map(|row| MusicAlbum {
        id: row.get("id"),
        artist: row.get("artist"),
        album_title: row.get("album_title"),
        year: row.get("year"),
        cover_path: row.get("cover_path"),
        music_path: row.get("music_path"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }))
}

pub async fn get_music_tracks_by_album(pool: &DbPool, album_id: &str) -> Result<Vec<MusicTrack>, String> {
    let rows = sqlx::query(
        "SELECT id, album_id, title, track_number, duration, file_path, file_hash, created_at
         FROM music_tracks WHERE album_id = ?1 ORDER BY track_number, title"
    )
    .bind(album_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.iter().map(|row| MusicTrack {
        id: row.get("id"),
        album_id: row.get("album_id"),
        title: row.get("title"),
        track_number: row.get("track_number"),
        duration: row.get("duration"),
        file_path: row.get("file_path"),
        file_hash: row.get("file_hash"),
        created_at: row.get("created_at"),
    }).collect())
}

pub async fn get_music_track_rating(pool: &DbPool, track_id: &str) -> Result<Option<i32>, String> {
    let row = sqlx::query_scalar::<_, i32>("SELECT rating FROM music_track_ratings WHERE track_id = ?1")
        .bind(track_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row)
}

pub async fn set_music_track_rating(pool: &DbPool, track_id: &str, rating: i32) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO music_track_ratings (track_id, rating) VALUES (?1, ?2)
         ON CONFLICT(track_id) DO UPDATE SET rating = excluded.rating"
    )
    .bind(track_id)
    .bind(rating)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_music_playlists(pool: &DbPool) -> Result<Vec<MusicPlaylist>, String> {
    let rows = sqlx::query("SELECT id, name, created_at FROM music_playlists ORDER BY name")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.iter().map(|row| MusicPlaylist {
        id: row.get("id"),
        name: row.get("name"),
        created_at: row.get("created_at"),
    }).collect())
}

pub async fn get_music_genres(_pool: &DbPool) -> Result<Vec<String>, String> {
    Ok(Vec::new())
}
