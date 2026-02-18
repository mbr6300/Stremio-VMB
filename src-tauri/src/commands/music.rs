use crate::db::DbPool;
use crate::services::music_metadata;
use crate::services::music_storage::{self, MusicAlbum, MusicTrack};
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub async fn check_music_path(path: String) -> Result<music_metadata::MusicPathCheckResult, String> {
    Ok(music_metadata::check_music_path(&path))
}

#[tauri::command]
pub async fn scan_music_dirs_progressive(
    app: AppHandle,
    pool: State<'_, DbPool>,
    paths: Vec<String>,
) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let cover_cache = app_data_dir.join("music-covers");
    std::fs::create_dir_all(&cover_cache).map_err(|e| e.to_string())?;

    let pool_guard = pool.inner().clone();
    tauri::async_runtime::spawn(async move {
        const EMIT_BATCH_SIZE: usize = 20;

        for path in paths {
            let mut albums: std::collections::HashMap<String, (MusicAlbum, Vec<MusicTrack>)> =
                std::collections::HashMap::new();

            for batch in music_metadata::scan_music_directory_batches(&path, EMIT_BATCH_SIZE) {
                for mf in batch {
                    let album_key = format!("{}|||{}", mf.artist, mf.album);
                    let music_path = std::path::Path::new(&mf.file_path)
                        .parent()
                        .and_then(|p| p.to_str())
                        .unwrap_or(&path)
                        .to_string();

                    let cover_path = if let Some(ref pic) = mf.picture {
                        music_metadata::save_cover_to_cache(pic, &mf.artist, &mf.album, &cover_cache)
                            .map(|p| p.to_string_lossy().to_string())
                    } else {
                        None
                    };

                    let track = MusicTrack {
                        id: uuid::Uuid::new_v4().to_string(),
                        album_id: String::new(),
                        title: mf.title.clone(),
                        track_number: mf.track_number.map(|n| n as i32),
                        duration: Some(mf.duration_secs as i32),
                        file_path: mf.file_path.clone(),
                        file_hash: Some(mf.file_hash.clone()),
                        created_at: String::new(),
                    };

                    albums
                        .entry(album_key)
                        .and_modify(|(album, tracks)| {
                            tracks.push(MusicTrack {
                                album_id: album.id.clone(),
                                ..track.clone()
                            });
                        })
                        .or_insert_with(|| {
                            let album_id = uuid::Uuid::new_v4().to_string();
                            let album = MusicAlbum {
                                id: album_id.clone(),
                                artist: mf.artist.clone(),
                                album_title: mf.album.clone(),
                                year: mf.year.map(|y| y as i32),
                                cover_path,
                                music_path,
                                created_at: String::new(),
                                updated_at: String::new(),
                            };
                            let track_with_album = MusicTrack {
                                album_id: album_id.clone(),
                                ..track
                            };
                            (album, vec![track_with_album])
                        });
                }

                // Nach jedem Batch: Alben in DB speichern und an Frontend emittieren
                for (album, tracks) in albums.values_mut() {
                    if music_storage::upsert_music_album(&pool_guard, album).await.is_ok() {
                        for track in std::mem::take(tracks) {
                            let _ = music_storage::upsert_music_track(&pool_guard, &track).await;
                        }
                        if let Ok(Some(a)) =
                            music_storage::get_music_album(&pool_guard, &album.id).await
                        {
                            let _ = app.emit("music-album-added", &a);
                        }
                    }
                }
            }
        }
        let _ = app.emit("music-scan-complete", ());
    });

    Ok(())
}

#[tauri::command]
pub async fn get_music_library(pool: State<'_, DbPool>) -> Result<Vec<MusicAlbum>, String> {
    music_storage::get_all_music_albums(pool.inner()).await
}

#[tauri::command]
pub async fn get_music_album(
    pool: State<'_, DbPool>,
    album_id: String,
) -> Result<Option<(MusicAlbum, Vec<MusicTrack>)>, String> {
    let album = music_storage::get_music_album(pool.inner(), &album_id).await?;
    match album {
        Some(a) => {
            let tracks = music_storage::get_music_tracks_by_album(pool.inner(), &album_id).await?;
            Ok(Some((a, tracks)))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_music_genres(pool: State<'_, DbPool>) -> Result<Vec<String>, String> {
    music_storage::get_music_genres(pool.inner()).await
}

#[tauri::command]
pub async fn fetch_music_album_metadata(
    _pool: State<'_, DbPool>,
    _album_id: String,
) -> Result<Option<MusicAlbum>, String> {
    Ok(None)
}

#[tauri::command]
pub async fn fetch_music_metadata_batch(_pool: State<'_, DbPool>) -> Result<u32, String> {
    Ok(0)
}

#[tauri::command]
pub async fn get_random_music_tracks(
    pool: State<'_, DbPool>,
    limit: Option<u32>,
) -> Result<Vec<(MusicTrack, MusicAlbum)>, String> {
    let albums = music_storage::get_all_music_albums(pool.inner()).await?;
    let mut result = Vec::new();
    let limit = limit.unwrap_or(100).min(100) as usize;
    for album in albums.into_iter().take(limit) {
        let tracks = music_storage::get_music_tracks_by_album(pool.inner(), &album.id).await?;
        for track in tracks.into_iter().take(1) {
            result.push((track, album.clone()));
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn get_music_playlist_by_id(
    _pool: State<'_, DbPool>,
    _playlist_id: String,
) -> Result<Option<music_storage::MusicPlaylist>, String> {
    Ok(None)
}

#[tauri::command]
pub async fn get_music_playlists(pool: State<'_, DbPool>) -> Result<Vec<music_storage::MusicPlaylist>, String> {
    music_storage::get_music_playlists(pool.inner()).await
}

#[tauri::command]
pub async fn create_music_playlist(
    _pool: State<'_, DbPool>,
    _name: String,
) -> Result<music_storage::MusicPlaylist, String> {
    Err("Noch nicht implementiert".to_string())
}

#[tauri::command]
pub async fn rename_music_playlist(
    _pool: State<'_, DbPool>,
    _playlist_id: String,
    _name: String,
) -> Result<(), String> {
    Err("Noch nicht implementiert".to_string())
}

#[tauri::command]
pub async fn delete_music_playlist(
    _pool: State<'_, DbPool>,
    _playlist_id: String,
) -> Result<(), String> {
    Err("Noch nicht implementiert".to_string())
}

#[tauri::command]
pub async fn get_music_playlist_tracks(
    _pool: State<'_, DbPool>,
    _playlist_id: String,
) -> Result<Vec<(MusicTrack, MusicAlbum)>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub async fn add_track_to_music_playlist(
    _pool: State<'_, DbPool>,
    _playlist_id: String,
    _track_id: String,
) -> Result<(), String> {
    Err("Noch nicht implementiert".to_string())
}

#[tauri::command]
pub async fn remove_track_from_music_playlist(
    _pool: State<'_, DbPool>,
    _playlist_id: String,
    _track_id: String,
) -> Result<(), String> {
    Err("Noch nicht implementiert".to_string())
}

#[tauri::command]
pub async fn create_music_radio_playlist(
    _pool: State<'_, DbPool>,
    _track_id: String,
) -> Result<music_storage::MusicPlaylist, String> {
    Err("Noch nicht implementiert".to_string())
}

#[tauri::command]
pub async fn set_music_track_rating(
    pool: State<'_, DbPool>,
    track_id: String,
    rating: i32,
) -> Result<(), String> {
    if !(1..=5).contains(&rating) {
        return Err("Bewertung muss zwischen 1 und 5 liegen".to_string());
    }
    music_storage::set_music_track_rating(pool.inner(), &track_id, rating).await
}
