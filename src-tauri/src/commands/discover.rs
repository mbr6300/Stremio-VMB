use crate::db::DbPool;
use crate::services::discovery::{self, StreamSearchResult, TmdbGenre};
use crate::services::perplexity;
use crate::services::storage::{self, DiscoverItemRow, DiscoverList};
use tauri::State;

#[tauri::command]
pub async fn search_streams(
    pool: State<'_, DbPool>,
    query: String,
    media_type: String,
    genre_id: Option<i64>,
) -> Result<Vec<StreamSearchResult>, String> {
    let settings = storage::load_all_settings(&pool).await?;
    let api_key = settings
        .get("tmdb_api_key")
        .filter(|k| !k.is_empty())
        .ok_or("Kein TMDb API-Key konfiguriert. Bitte in den Einstellungen setzen.")?
        .clone();

    discovery::search_streams(&api_key, &query, &media_type, genre_id).await
}

#[tauri::command]
pub async fn get_tmdb_genres(pool: State<'_, DbPool>) -> Result<Vec<TmdbGenre>, String> {
    let settings = storage::load_all_settings(&pool).await?;
    let api_key = settings
        .get("tmdb_api_key")
        .filter(|k| !k.is_empty())
        .ok_or("Kein TMDb API-Key konfiguriert. Bitte in den Einstellungen setzen.")?
        .clone();

    discovery::get_tmdb_genres(&api_key).await
}

#[tauri::command]
pub async fn refresh_discover_lists(
    pool: State<'_, DbPool>,
    country: Option<String>,
) -> Result<Vec<DiscoverList>, String> {
    let settings = storage::load_all_settings(&pool).await?;
    let api_key = settings
        .get("tmdb_api_key")
        .filter(|k| !k.is_empty())
        .ok_or("Kein TMDb API-Key konfiguriert. Bitte in den Einstellungen setzen.")?
        .clone();

    let country = country
        .or_else(|| settings.get("discover_country").cloned())
        .unwrap_or_else(|| "CH".to_string());

    let lists = discovery::fetch_all_lists(&api_key, &country).await?;

    storage::clear_discover_data(&pool).await?;

    for list in &lists {
        let list_id = uuid::Uuid::new_v4().to_string();

        storage::insert_discover_list(
            &pool,
            &list_id,
            &list.list_type,
            list.provider.as_deref(),
            &list.country,
            &list.title,
        )
        .await?;

        for (i, item) in list.items.iter().enumerate() {
            storage::insert_discover_item(
                &pool,
                &uuid::Uuid::new_v4().to_string(),
                &list_id,
                &item.media_type,
                None,
                Some(item.tmdb_id),
                &item.title,
                item.year,
                item.rating,
                item.poster_url.as_deref(),
                Some(&item.overview),
                list.provider.as_deref(),
                &item.raw_json,
                i as i32,
            )
            .await?;
        }
    }

    storage::get_discover_lists(&pool).await
}

#[tauri::command]
pub async fn get_discover_lists(
    pool: State<'_, DbPool>,
) -> Result<Vec<DiscoverList>, String> {
    storage::get_discover_lists(&pool).await
}

#[tauri::command]
pub async fn get_ai_recommendations_list(
    pool: State<'_, DbPool>,
    preset: String,
    force_refresh: Option<bool>,
) -> Result<DiscoverList, String> {
    let force = force_refresh.unwrap_or(false);

    if !force {
        if let Some(cached) = storage::get_ai_recommendations_cached(&pool, &preset).await? {
            return Ok(cached);
        }
    }

    let settings = storage::load_all_settings(&pool).await?;
    let perplexity_key = settings
        .get("perplexity_api_key")
        .filter(|k| !k.is_empty())
        .ok_or("Perplexity API-Key fehlt. In Einstellungen setzen.")?
        .clone();
    let tmdb_key = settings
        .get("tmdb_api_key")
        .filter(|k| !k.is_empty())
        .ok_or("TMDb API-Key fehlt. In Einstellungen setzen.")?
        .clone();
    let favorite_films = settings.get("favorite_films").map(|s| s.as_str());

    let ai_items = perplexity::get_ai_recommendations(
        &perplexity_key,
        &preset,
        favorite_films,
    )
    .await?;

    let mut items = Vec::new();
    let list_id = format!("ai-{}", preset);

    for (i, ai) in ai_items.iter().enumerate() {
        let search_results = discovery::search_streams(
            &tmdb_key,
            &ai.title,
            "movie",
            None,
        )
        .await
        .unwrap_or_default();

        let first = search_results.into_iter().next();
        if let Some(sr) = first {
            items.push(DiscoverItemRow {
                id: uuid::Uuid::new_v4().to_string(),
                list_id: list_id.clone(),
                media_type: "movie".to_string(),
                external_id: None,
                tmdb_id: Some(sr.tmdb_id),
                title: sr.title,
                year: sr.year,
                rating: sr.rating,
                poster_url: sr.poster_url,
                overview: Some(sr.overview),
                provider: None,
                sort_order: i as i32,
            });
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let list = DiscoverList {
        id: list_id.clone(),
        list_type: "ai_recommendations".to_string(),
        provider: None,
        country: None,
        title: preset_title(&preset),
        created_at: String::new(),
        items,
    };

    storage::save_ai_recommendations_cache(&pool, &preset, &list).await?;

    Ok(list)
}

fn preset_title(preset: &str) -> String {
    match preset {
        "my_taste" => "My taste",
        "70s" => "Best of 70s",
        "80s" => "Best of 80s",
        "90s" => "Best of 90s",
        "00s" => "Best of 00s",
        "2010s" => "Best of 2010s",
        "action" => "Action",
        "comedy" => "Comedy",
        "drama" => "Drama",
        "thriller" => "Thriller",
        _ => preset,
    }
    .to_string()
}
