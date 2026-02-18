use crate::db::DbPool;
use crate::services::discovery;
use crate::services::metadata_service::MetadataService;
use crate::services::perplexity;
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

#[tauri::command]
pub async fn check_api_configuration_status(
    pool: State<'_, DbPool>,
    tmdb_api_key: Option<String>,
    perplexity_api_key: Option<String>,
) -> Result<ApiConfigurationStatus, String> {
    let settings = storage::load_all_settings(&pool).await?;

    let effective_tmdb_key = tmdb_api_key
        .or_else(|| settings.get("tmdb_api_key").cloned())
        .unwrap_or_default()
        .trim()
        .to_string();
    let effective_perplexity_key = perplexity_api_key
        .or_else(|| settings.get("perplexity_api_key").cloned())
        .unwrap_or_default()
        .trim()
        .to_string();

    let tmdb = if effective_tmdb_key.is_empty() {
        ApiServiceStatus {
            configured: false,
            connected: false,
            message: "Kein API-Key gesetzt.".to_string(),
        }
    } else {
        match discovery::get_tmdb_genres(&effective_tmdb_key).await {
            Ok(genres) => ApiServiceStatus {
                configured: true,
                connected: true,
                message: format!("Verbindung ok ({} Genres geladen).", genres.len()),
            },
            Err(err) => ApiServiceStatus {
                configured: true,
                connected: false,
                message: format!("Verbindung fehlgeschlagen: {}", err),
            },
        }
    };

    let perplexity = if effective_perplexity_key.is_empty() {
        ApiServiceStatus {
            configured: false,
            connected: false,
            message: "Kein API-Key gesetzt.".to_string(),
        }
    } else {
        let probe_titles = vec!["Inception (2010)".to_string()];
        match perplexity::classify_media_titles(&effective_perplexity_key, &probe_titles).await {
            Ok(_) => ApiServiceStatus {
                configured: true,
                connected: true,
                message: "Verbindung ok.".to_string(),
            },
            Err(err) => ApiServiceStatus {
                configured: true,
                connected: false,
                message: format!("Verbindung fehlgeschlagen: {}", err),
            },
        }
    };

    Ok(ApiConfigurationStatus { tmdb, perplexity })
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

#[derive(serde::Serialize)]
pub struct ApiConfigurationStatus {
    pub tmdb: ApiServiceStatus,
    pub perplexity: ApiServiceStatus,
}

#[derive(serde::Serialize)]
pub struct ApiServiceStatus {
    pub configured: bool,
    pub connected: bool,
    pub message: String,
}
