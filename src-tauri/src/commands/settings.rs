use crate::db::DbPool;
use crate::services::metadata_service::MetadataService;
use crate::services::storage;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn save_settings(
    pool: State<'_, DbPool>,
    meta_service: State<'_, MetadataService>,
    settings: HashMap<String, String>,
) -> Result<(), String> {
    for (key, value) in &settings {
        storage::save_setting(&pool, key, value).await?;
    }

    if let Some(api_key) = settings.get("tmdb_api_key") {
        if !api_key.is_empty() {
            meta_service.set_tmdb_key(api_key.clone()).await;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn load_settings(pool: State<'_, DbPool>) -> Result<HashMap<String, String>, String> {
    storage::load_all_settings(&pool).await
}
