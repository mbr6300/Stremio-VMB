mod db;
mod services;
mod commands;

use services::metadata_service::MetadataService;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            app.handle().plugin(tauri_plugin_dialog::init())?;

            let app_handle = app.handle().clone();
            let app_data_dir = app_handle
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");

            tauri::async_runtime::block_on(async {
                let pool = db::init_db(&app_data_dir)
                    .await
                    .expect("Failed to initialize database");

                let meta_service = MetadataService::new();

                if let Ok(settings) = services::storage::load_all_settings(&pool).await {
                    if let Some(api_key) = settings.get("tmdb_api_key") {
                        if !api_key.is_empty() {
                            meta_service.set_tmdb_key(api_key.clone()).await;
                        }
                    }
                }

                app_handle.manage(pool);
                app_handle.manage(meta_service);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::library::check_media_path,
            commands::library::scan_media_dirs,
            commands::library::scan_media_dirs_progressive,
            commands::library::get_library,
            commands::library::get_library_with_metadata,
            commands::library::get_media_item,
            commands::library::delete_media_item,
            commands::library::improve_classifications_with_perplexity,
            commands::settings::save_settings,
            commands::settings::load_settings,
            commands::settings::check_quickconnect,
            commands::settings::diagnose_path,
            commands::realdebrid::rd_get_device_code,
            commands::realdebrid::rd_poll_credentials,
            commands::realdebrid::rd_save_api_key,
            commands::realdebrid::rd_unrestrict_link,
            commands::realdebrid::rd_get_status,
            commands::metadata::fetch_metadata,
            commands::metadata::fetch_metadata_by_tmdb_id,
            commands::metadata::fetch_metadata_batch,
            commands::metadata::get_metadata,
            commands::metadata::search_metadata,
            commands::metadata::get_person_details,
            commands::metadata::get_media_extended_info,
            commands::metadata::get_actor_movie_suggestions,
            commands::player::detect_players,
            commands::player::open_in_player,
            commands::player::open_in_default_player,
            commands::discover::search_streams,
            commands::discover::get_tmdb_genres,
            commands::discover::refresh_discover_lists,
            commands::discover::get_discover_lists,
            commands::discover::get_ai_recommendations_list,
            commands::rd_streams::search_rd_streams,
            commands::rd_streams::search_debrid_streams,
            commands::music::check_music_path,
            commands::music::scan_music_dirs_progressive,
            commands::music::get_music_library,
            commands::music::get_music_album,
            commands::music::get_music_genres,
            commands::music::fetch_music_album_metadata,
            commands::music::fetch_music_metadata_batch,
            commands::music::get_random_music_tracks,
            commands::music::get_music_playlist_by_id,
            commands::music::get_music_playlists,
            commands::music::create_music_playlist,
            commands::music::rename_music_playlist,
            commands::music::delete_music_playlist,
            commands::music::get_music_playlist_tracks,
            commands::music::add_track_to_music_playlist,
            commands::music::remove_track_from_music_playlist,
            commands::music::create_music_radio_playlist,
            commands::music::set_music_track_rating,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
