use crate::db::DbPool;
use crate::services::metadata_service::MetadataService;
use crate::services::quickconnect;
use crate::services::storage;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tauri::State;

#[tauri::command]
pub async fn check_quickconnect(quickconnect_id: String) -> Result<quickconnect::QuickConnectStatus, String> {
    Ok(quickconnect::check_quickconnect(&quickconnect_id).await)
}

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

/// Diagnose: Listet /Volumes und prüft einen Pfad. Hilft bei Zugriffsproblemen.
#[tauri::command]
pub async fn diagnose_path(path: String) -> Result<DiagnoseResult, String> {
    let path = path.trim();
    let volumes = fs::read_dir(Path::new("/Volumes"))
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|e| vec![format!("Fehler beim Lesen von /Volumes: {}", e)]);

    let path_exists = Path::new(path).exists();
    let path_is_dir = path_exists && Path::new(path).is_dir();

    Ok(DiagnoseResult {
        volumes,
        path_checked: path.to_string(),
        path_exists,
        path_is_dir,
    })
}

#[derive(serde::Serialize)]
pub struct DiagnoseResult {
    pub volumes: Vec<String>,
    pub path_checked: String,
    pub path_exists: bool,
    pub path_is_dir: bool,
}
