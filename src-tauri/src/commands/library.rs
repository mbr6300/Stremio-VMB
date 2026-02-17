use crate::db::DbPool;
use crate::services::local_media::{self, PathCheckResult, ScannedFile};
use crate::services::perplexity;
use crate::services::storage::{self, LibraryItemWithMeta, MediaItem};
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn scan_media_dirs_progressive(
    app: AppHandle,
    pool: State<'_, DbPool>,
    paths: Vec<String>,
) -> Result<(), String> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ScannedFile>(64);
    let paths_for_scan = paths.clone();
    let pool_guard = pool.inner().clone();

    let _scan_handle = tokio::task::spawn_blocking(move || {
        local_media::scan_directories_streaming(&paths_for_scan, |file| {
            let _ = tx.blocking_send(file);
        });
    });

    tauri::async_runtime::spawn(async move {
        while let Some(file) = rx.recv().await {
            let item = MediaItem {
                id: uuid::Uuid::new_v4().to_string(),
                title: file.title.clone(),
                file_path: file.file_path.clone(),
                file_hash: Some(file.file_hash.clone()),
                media_type: file.media_type.clone(),
                file_size: Some(file.file_size),
                series_name: None,
                created_at: String::new(),
                updated_at: String::new(),
            };
            if storage::upsert_media_item(&pool_guard, &item).await.is_ok() {
                if let Ok(Some(actual)) = storage::get_media_item_by_file_path(&pool_guard, &file.file_path).await {
                    let _ = app.emit("library-item-added", &actual);
                }
            }
        }
        let _ = app.emit("library-scan-complete", ());
    });

    Ok(())
}

#[tauri::command]
pub async fn check_media_path(path: String) -> Result<PathCheckResult, String> {
    let result = tokio::task::spawn_blocking(move || local_media::check_path(&path))
        .await
        .map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
pub async fn scan_media_dirs(pool: State<'_, DbPool>, paths: Vec<String>) -> Result<Vec<MediaItem>, String> {
    let scanned = tokio::task::spawn_blocking(move || {
        local_media::scan_directories(&paths)
    })
    .await
    .map_err(|e| e.to_string())?;

    for file in &scanned {
        let item = MediaItem {
            id: uuid::Uuid::new_v4().to_string(),
            title: file.title.clone(),
            file_path: file.file_path.clone(),
            file_hash: Some(file.file_hash.clone()),
            media_type: file.media_type.clone(),
            file_size: Some(file.file_size),
            series_name: None,
            created_at: String::new(),
            updated_at: String::new(),
        };
        storage::upsert_media_item(&pool, &item).await?;
    }

    storage::get_all_media_items(&pool).await
}

#[tauri::command]
pub async fn get_library(pool: State<'_, DbPool>) -> Result<Vec<MediaItem>, String> {
    storage::get_all_media_items(&pool).await
}

#[tauri::command]
pub async fn get_library_with_metadata(pool: State<'_, DbPool>) -> Result<Vec<LibraryItemWithMeta>, String> {
    storage::get_all_media_items_with_metadata(&pool).await
}

#[tauri::command]
pub async fn get_media_item(pool: State<'_, DbPool>, id: String) -> Result<Option<MediaItem>, String> {
    storage::get_media_item_by_id(&pool, &id).await
}

#[tauri::command]
pub async fn delete_media_item(pool: State<'_, DbPool>, id: String) -> Result<(), String> {
    storage::delete_media_item(&pool, &id).await
}

const PERPLEXITY_BATCH_SIZE: usize = 25;

#[tauri::command]
pub async fn improve_classifications_with_perplexity(
    app: AppHandle,
    pool: State<'_, DbPool>,
) -> Result<u32, String> {
    let settings = storage::load_all_settings(&pool).await?;
    let api_key = settings
        .get("perplexity_api_key")
        .filter(|k| !k.is_empty())
        .ok_or("Perplexity API-Key fehlt. In Einstellungen setzen.")?
        .clone();

    let items = storage::get_all_media_items(&pool).await?;
    if items.is_empty() {
        return Ok(0);
    }

    let mut updated = 0u32;

    for chunk in items.chunks(PERPLEXITY_BATCH_SIZE) {
        let titles: Vec<String> = chunk.iter().map(|i| i.title.clone()).collect();

        match perplexity::classify_media_titles(&api_key, &titles).await {
            Ok(classifications) => {
                for (i, item) in chunk.iter().enumerate() {
                    let class = classifications.get(i).or_else(|| {
                        classifications.iter().find(|c| c.title == item.title)
                    });
                    if let Some(class) = class {
                        let media_type = if class.media_type.to_lowercase().contains("series") {
                            "series"
                        } else {
                            "movie"
                        };
                        if storage::update_media_classification(
                            &pool,
                            &item.id,
                            media_type,
                            class.series_name.as_deref(),
                        )
                        .await
                        .is_ok()
                        {
                            updated += 1;
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Perplexity batch error: {}", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    let _ = app.emit("library-classifications-improved", ());
    Ok(updated)
}
