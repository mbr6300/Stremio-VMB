use crate::db::DbPool;
use crate::services::metadata_service::MetadataService;
use crate::services::perplexity;
use crate::services::storage::{self, MediaMetadata};
use crate::services::tmdb_api;
use regex::Regex;
use tauri::{AppHandle, Emitter, State};

fn extract_series_name(title: &str) -> String {
    let re_sxe = Regex::new(r#"(?i)[Ss]\d{1,2}[Ee]\d{1,2}"#).unwrap();
    let re_x = Regex::new(r#"\d{1,2}[xX]\d{1,2}"#).unwrap();
    let re_ep = Regex::new(r#"(?i)[Ee]p?\d{1,2}"#).unwrap();
    let re_season_ep = Regex::new(r#"(?i)[Ss]eason\s*\d{1,2}\s*[Ee]pisode\s*\d{1,2}"#).unwrap();

    let mut cleaned = re_sxe.replace_all(title, "").into_owned();
    cleaned = re_x.replace_all(&cleaned, "").into_owned();
    cleaned = re_ep.replace_all(&cleaned, "").into_owned();
    cleaned = re_season_ep.replace_all(&cleaned, "").into_owned();
    cleaned = cleaned.replace('.', " ").replace('_', " ").replace('-', " ");
    let cleaned = cleaned.trim();

    let parts: Vec<&str> = cleaned.split_whitespace().filter(|s| !s.is_empty()).collect();
    let last = parts.last().and_then(|s| s.parse::<u32>().ok());
    let parts: Vec<&str> = if last.is_some() && parts.len() > 1 {
        parts[..parts.len() - 1].to_vec()
    } else {
        parts
    };
    let result = parts.join(" ").trim().to_string();
    if result.is_empty() {
        title.to_string()
    } else {
        result
    }
}

#[tauri::command]
pub async fn fetch_metadata(
    pool: State<'_, DbPool>,
    meta_service: State<'_, MetadataService>,
    media_item_id: String,
    year: Option<u16>,
) -> Result<Option<MediaMetadata>, String> {
    let item = storage::get_media_item_by_id(&pool, &media_item_id)
        .await?
        .ok_or("Media item not found")?;

    if !meta_service.has_providers().await {
        return Err("No metadata providers configured. Set a TMDb API key in Settings.".into());
    }

    let results = meta_service
        .search(&item.title, year, &item.media_type)
        .await?;

    let first = match results.into_iter().next() {
        Some(r) => r,
        None => return Ok(None),
    };

    let detail = meta_service
        .fetch_details(first.provider_id, &item.media_type, Some("tmdb"))
        .await?;

    let detail = match detail {
        Some(d) => d,
        None => first,
    };

    let metadata = MediaMetadata {
        id: uuid::Uuid::new_v4().to_string(),
        media_item_id: media_item_id.clone(),
        tmdb_id: Some(detail.provider_id),
        title: Some(detail.title),
        overview: Some(detail.overview),
        poster_url: detail.poster_url,
        backdrop_url: detail.backdrop_url,
        release_date: detail.release_date,
        rating: detail.rating,
        runtime: detail.runtime,
        genres: Some(serde_json::to_string(&detail.genres).unwrap_or_default()),
        cast_crew: detail
            .cast_crew
            .as_ref()
            .map(|cc| serde_json::to_string(cc).unwrap_or_default()),
        raw_response: Some(detail.raw_response),
        created_at: String::new(),
        updated_at: String::new(),
    };

    storage::upsert_metadata(&pool, &metadata).await?;
    storage::get_metadata_for_item(&pool, &media_item_id).await
}

#[tauri::command]
pub async fn fetch_metadata_by_tmdb_id(
    pool: State<'_, DbPool>,
    meta_service: State<'_, MetadataService>,
    media_item_id: String,
    tmdb_id: i64,
    media_type: String,
) -> Result<Option<MediaMetadata>, String> {
    if !meta_service.has_providers().await {
        return Err("No metadata providers configured. Set a TMDb API key in Settings.".into());
    }

    let detail = meta_service
        .fetch_details(tmdb_id, &media_type, Some("tmdb"))
        .await?;

    let detail = match detail {
        Some(d) => d,
        None => return Ok(None),
    };

    let metadata = MediaMetadata {
        id: uuid::Uuid::new_v4().to_string(),
        media_item_id: media_item_id.clone(),
        tmdb_id: Some(detail.provider_id),
        title: Some(detail.title),
        overview: Some(detail.overview),
        poster_url: detail.poster_url,
        backdrop_url: detail.backdrop_url,
        release_date: detail.release_date,
        rating: detail.rating,
        runtime: detail.runtime,
        genres: Some(serde_json::to_string(&detail.genres).unwrap_or_default()),
        cast_crew: detail
            .cast_crew
            .as_ref()
            .map(|cc| serde_json::to_string(cc).unwrap_or_default()),
        raw_response: Some(detail.raw_response),
        created_at: String::new(),
        updated_at: String::new(),
    };

    let persist = storage::get_media_item_by_id(&pool, &media_item_id)
        .await?
        .is_some();
    if persist {
        storage::upsert_metadata(&pool, &metadata).await?;
        storage::get_metadata_for_item(&pool, &media_item_id).await
    } else {
        Ok(Some(metadata))
    }
}

#[tauri::command]
pub async fn get_metadata(
    pool: State<'_, DbPool>,
    media_item_id: String,
) -> Result<Option<MediaMetadata>, String> {
    storage::get_metadata_for_item(&pool, &media_item_id).await
}

#[tauri::command]
pub async fn search_metadata(
    meta_service: State<'_, MetadataService>,
    title: String,
    year: Option<u16>,
    media_type: String,
) -> Result<Vec<MetadataSearchResult>, String> {
    if !meta_service.has_providers().await {
        return Err("No metadata providers configured. Set a TMDb API key in Settings.".into());
    }

    let results = meta_service.search(&title, year, &media_type).await?;

    Ok(results
        .into_iter()
        .map(|r| MetadataSearchResult {
            tmdb_id: r.provider_id,
            title: r.title,
            overview: r.overview,
            poster_url: r.poster_url,
            release_date: r.release_date,
            rating: r.rating,
        })
        .collect())
}

/// Fetches metadata for all library items that don't have it yet.
/// Emits "metadata-fetched" for each item, then "metadata-batch-complete".
#[tauri::command]
pub async fn fetch_metadata_batch(
    app: AppHandle,
    pool: State<'_, DbPool>,
    meta_service: State<'_, MetadataService>,
) -> Result<u32, String> {
    if !meta_service.has_providers().await {
        return Err("No metadata providers configured. Set a TMDb API key in Settings.".into());
    }

    let items_without_meta = storage::get_items_without_metadata(&pool).await?;
    let mut fetched = 0u32;

    for item in items_without_meta {
        let year = extract_year_from_title(&item.title);
        match fetch_metadata_inner(
            pool.inner(),
            meta_service.inner(),
            &item.id,
            &item.title,
            &item.media_type,
            year,
        )
        .await
        {
            Ok(Some(meta)) => {
                fetched += 1;
                let _ = app.emit("metadata-fetched", (&item, &meta));
                tokio::time::sleep(tokio::time::Duration::from_millis(350)).await;
            }
            Ok(None) | Err(_) => {}
        }
    }

    let _ = app.emit("metadata-batch-complete", ());
    Ok(fetched)
}

fn extract_year_from_title(title: &str) -> Option<u16> {
    let start = title.find('(')?;
    let rest = title.get(start + 1..)?;
    let year_str = rest.get(..4)?;
    if year_str.chars().all(|c| c.is_ascii_digit()) {
        year_str.parse().ok()
    } else {
        None
    }
}

async fn fetch_metadata_inner(
    pool: &DbPool,
    meta_service: &MetadataService,
    media_item_id: &str,
    title: &str,
    media_type: &str,
    year: Option<u16>,
) -> Result<Option<MediaMetadata>, String> {
    let search_title = if media_type == "series" {
        extract_series_name(title)
    } else {
        title.to_string()
    };
    let results = meta_service.search(&search_title, year, media_type).await?;
    let first = match results.into_iter().next() {
        Some(r) => r,
        None => return Ok(None),
    };

    let detail = meta_service
        .fetch_details(first.provider_id, media_type, Some("tmdb"))
        .await?;

    let detail = match detail {
        Some(d) => d,
        None => first,
    };

    let metadata = MediaMetadata {
        id: uuid::Uuid::new_v4().to_string(),
        media_item_id: media_item_id.to_string(),
        tmdb_id: Some(detail.provider_id),
        title: Some(detail.title),
        overview: Some(detail.overview),
        poster_url: detail.poster_url,
        backdrop_url: detail.backdrop_url,
        release_date: detail.release_date,
        rating: detail.rating,
        runtime: detail.runtime,
        genres: Some(serde_json::to_string(&detail.genres).unwrap_or_default()),
        cast_crew: detail
            .cast_crew
            .as_ref()
            .map(|cc| serde_json::to_string(cc).unwrap_or_default()),
        raw_response: Some(detail.raw_response),
        created_at: String::new(),
        updated_at: String::new(),
    };

    storage::upsert_metadata(&pool, &metadata).await?;
    storage::get_metadata_for_item(&pool, media_item_id).await
}

#[derive(serde::Serialize)]
pub struct MetadataSearchResult {
    pub tmdb_id: i64,
    pub title: String,
    pub overview: String,
    pub poster_url: Option<String>,
    pub release_date: Option<String>,
    pub rating: Option<f64>,
}

#[tauri::command]
pub async fn get_person_details(
    pool: State<'_, DbPool>,
    person_id: i64,
    known_for: Option<String>,
) -> Result<PersonDetailsResponse, String> {
    let settings = storage::load_all_settings(&pool).await?;
    let tmdb_key = settings
        .get("tmdb_api_key")
        .filter(|k| !k.is_empty())
        .ok_or("Kein TMDb API-Key konfiguriert.")?
        .clone();

    let mut details = tmdb_api::fetch_person_details(&tmdb_key, person_id).await?;

    if let Some(pp_key) = settings.get("perplexity_api_key").filter(|k| !k.is_empty()) {
        if let Ok(anecdotes) = perplexity::get_person_anecdotes(
            pp_key,
            &details.name,
            known_for.as_deref(),
        )
        .await
        {
            details.anecdotes = anecdotes.anecdotes;
            if details.height.is_none() {
                details.height = anecdotes.height;
            }
            if details.partner_status.is_none() {
                details.partner_status = anecdotes.partner_status;
            }
            if details.children.is_none() {
                details.children = anecdotes.children;
            }
        }
    }

    Ok(PersonDetailsResponse {
        id: details.id,
        name: details.name,
        biography: details.biography,
        profile_url: details.profile_url,
        birthday: details.birthday,
        deathday: details.deathday,
        place_of_birth: details.place_of_birth,
        known_for_department: details.known_for_department,
        age: details.age,
        anecdotes: details.anecdotes,
        height: details.height,
        partner_status: details.partner_status,
        children: details.children,
    })
}

#[derive(serde::Serialize)]
pub struct PersonDetailsResponse {
    pub id: i64,
    pub name: String,
    pub biography: Option<String>,
    pub profile_url: Option<String>,
    pub birthday: Option<String>,
    pub deathday: Option<String>,
    pub place_of_birth: Option<String>,
    pub known_for_department: Option<String>,
    pub age: Option<i32>,
    pub anecdotes: Vec<String>,
    pub height: Option<String>,
    pub partner_status: Option<String>,
    pub children: Option<String>,
}

#[tauri::command]
pub async fn get_media_extended_info(
    pool: State<'_, DbPool>,
    tmdb_id: i64,
    media_type: String,
    title: Option<String>,
    year: Option<String>,
) -> Result<MediaExtendedInfoResponse, String> {
    let settings = storage::load_all_settings(&pool).await?;
    let tmdb_key = settings
        .get("tmdb_api_key")
        .filter(|k| !k.is_empty())
        .ok_or("Kein TMDb API-Key konfiguriert.")?
        .clone();

    let mut info = tmdb_api::fetch_media_extended(&tmdb_key, tmdb_id, &media_type).await?;

    if let Some(pp_key) = settings.get("perplexity_api_key").filter(|k| !k.is_empty()) {
        let t = title.as_deref().unwrap_or("Unbekannt");
        if let Ok(trivia) =
            perplexity::get_film_trivia(pp_key, t, year.as_deref(), &media_type).await
        {
            info.trivia_facts = trivia;
        }
    }

    Ok(MediaExtendedInfoResponse {
        tagline: info.tagline,
        trivia_facts: info.trivia_facts,
    })
}

#[derive(serde::Serialize)]
pub struct MediaExtendedInfoResponse {
    pub tagline: Option<String>,
    pub trivia_facts: Vec<String>,
}

#[tauri::command]
pub async fn get_actor_movie_suggestions(
    pool: State<'_, DbPool>,
    tmdb_id: i64,
    media_type: String,
    actor_ids: Vec<i64>,
    director_ids: Vec<i64>,
) -> Result<Vec<ActorMovieSuggestionResponse>, String> {
    let settings = storage::load_all_settings(&pool).await?;
    let api_key = settings
        .get("tmdb_api_key")
        .filter(|k| !k.is_empty())
        .ok_or("Kein TMDb API-Key konfiguriert.")?
        .clone();

    let suggestions =
        tmdb_api::fetch_actor_movie_suggestions(&api_key, tmdb_id, &media_type, &actor_ids, &director_ids, 12)
            .await?;

    Ok(suggestions
        .into_iter()
        .map(|s| ActorMovieSuggestionResponse {
            tmdb_id: s.tmdb_id,
            title: s.title,
            year: s.year,
            poster_url: s.poster_url,
            media_type: s.media_type,
            match_reason: s.match_reason,
        })
        .collect())
}

#[derive(serde::Serialize)]
pub struct ActorMovieSuggestionResponse {
    pub tmdb_id: i64,
    pub title: String,
    pub year: Option<i32>,
    pub poster_url: Option<String>,
    pub media_type: String,
    pub match_reason: String,
}
