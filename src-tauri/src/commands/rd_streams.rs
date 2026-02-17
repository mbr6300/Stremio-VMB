use crate::db::DbPool;
use crate::services::rd_streams::{self, RdStreamLink};
use crate::services::storage;
use tauri::State;

async fn do_search_rd_streams(
    pool: &DbPool,
    title: &str,
    year: Option<u32>,
    media_type: &str,
    tmdb_id: Option<i64>,
) -> Result<Vec<RdStreamLink>, String> {
    let settings = storage::load_all_settings(pool).await?;
    let api_key = settings.get("tmdb_api_key").filter(|k| !k.is_empty());
    let debridio_url = settings.get("debridio_url").filter(|k| !k.is_empty());
    let rd_token = storage::load_rd_token(&pool).await?.map(|t| t.access_token);

    let has_debridio = debridio_url
        .and_then(|u| rd_streams::extract_debridio_base_url(u))
        .is_some();
    let has_rd = rd_token.is_some();

    if !has_debridio && !has_rd {
        return Err(
            "Weder Debridio noch RealDebrid konfiguriert. Bitte in Einstellungen Debridio-URL oder RealDebrid API-Key eintragen."
                .to_string(),
        );
    }

    let mut streams = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut effective_tmdb_id = tmdb_id;
    if effective_tmdb_id.is_none() && api_key.is_some() {
        if let Ok(Some(id)) = rd_streams::search_tmdb_for_id(
            api_key.as_ref().unwrap(),
            &title,
            year,
            &media_type,
        )
        .await
        {
            effective_tmdb_id = Some(id);
        }
    }

    let imdb_id_opt = if (media_type == "movie" || media_type == "series")
        && effective_tmdb_id.is_some()
        && api_key.is_some()
    {
        rd_streams::get_imdb_id(
            api_key.as_ref().unwrap(),
            effective_tmdb_id.unwrap(),
            &media_type,
        )
        .await
        .ok()
        .flatten()
    } else {
        None
    };

    if let (Some(base_url), Some(imdb_id)) = (
        debridio_url.and_then(|u| rd_streams::extract_debridio_base_url(u)),
        imdb_id_opt.as_deref(),
    ) {
        if let Ok(debridio_streams) = rd_streams::fetch_debridio_streams(
            &base_url,
            imdb_id,
            &media_type,
            None,
            None,
            rd_token.as_deref(),
        )
        .await
        {
            for link in debridio_streams {
                let key = format!("{}:{}", link.quality, link.size);
                if seen.insert(key) {
                    streams.push(link);
                }
            }
        }
    }

    if has_rd {
        let token_clone = rd_token.as_ref().unwrap().clone();

        let (torrentio_streams, yts_magnets) = tokio::join!(
            async {
                let mut out = Vec::new();
                if let Some(imdb_id) = &imdb_id_opt {
                    if let Ok(t) = rd_streams::fetch_torrentio_streams(
                        imdb_id,
                        &media_type,
                        None,
                        None,
                    )
                    .await
                    {
                        out = t.into_iter().take(15).collect();
                    }
                }
                out
            },
            async {
                if media_type == "movie" {
                    rd_streams::search_movie_torrents(&title, year).await.unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
        );

        let to_resolve: Vec<_> = torrentio_streams
            .into_iter()
            .filter(|(_, _, q, s)| seen.insert(format!("{}:{}", q, s)))
            .collect();

        let torrentio_handles: Vec<_> = to_resolve
            .into_iter()
            .map(|(hash, idx, qual, sz)| {
                let token = token_clone.clone();
                tokio::spawn(async move {
                    rd_streams::info_hash_to_stream(&token, &hash, idx, &qual, &sz).await
                })
            })
            .collect();

        for h in torrentio_handles {
            if let Ok(Ok(Some(link))) = h.await {
                streams.push(link);
            }
        }

        for (magnet, quality, size) in yts_magnets.into_iter().take(8) {
            let key = format!("{}:{}", quality, size);
            if seen.insert(key) {
                if let Ok(Some(link)) =
                    rd_streams::magnet_to_stream(&token_clone, &magnet, &quality, &size).await
                {
                    streams.push(link);
                }
            }
        }
    }

    rd_streams::sort_streams_by_resolution(&mut streams);
    Ok(streams)
}

#[tauri::command]
pub async fn search_rd_streams(
    pool: State<'_, DbPool>,
    title: String,
    year: Option<u32>,
    media_type: String,
    tmdb_id: Option<i64>,
) -> Result<Vec<RdStreamLink>, String> {
    do_search_rd_streams(pool.inner(), &title, year, &media_type, tmdb_id).await
}

#[derive(serde::Serialize)]
pub struct DebridSearchResult {
    pub matched_title: Option<String>,
    pub matched_year: Option<i32>,
    pub matched_tmdb_id: Option<i64>,
    pub media_type: String,
    pub streams: Vec<RdStreamLink>,
}

/// Search Debridio/RealDebrid for streams by title. Returns metadata + streams.
#[tauri::command]
pub async fn search_debrid_streams(
    pool: State<'_, DbPool>,
    title: String,
    media_type: String,
    year: Option<u32>,
) -> Result<DebridSearchResult, String> {
    let media_type = if media_type == "tv" {
        "series".to_string()
    } else {
        media_type
    };
    let settings = storage::load_all_settings(&pool).await?;
    let api_key = settings.get("tmdb_api_key").filter(|k| !k.is_empty());
    let debridio_url = settings.get("debridio_url").filter(|k| !k.is_empty());
    let rd_token = storage::load_rd_token(&pool).await?.map(|t| t.access_token);

    let has_debridio = debridio_url
        .and_then(|u| rd_streams::extract_debridio_base_url(u))
        .is_some();
    let has_rd = rd_token.is_some();

    if !has_debridio && !has_rd {
        return Err(
            "Weder Debridio noch RealDebrid konfiguriert. Bitte in Einstellungen Debridio-URL oder RealDebrid API-Key eintragen."
                .to_string(),
        );
    }

    let mut matched_title: Option<String> = None;
    let mut matched_year: Option<i32> = year.map(|y| y as i32);
    let mut matched_tmdb_id: Option<i64> = None;

    if let Some(key) = &api_key {
        if let Ok(Some((tmdb_id, tmdb_title, tmdb_year))) =
            rd_streams::search_tmdb_for_match(key, &title, year, &media_type).await
        {
            matched_tmdb_id = Some(tmdb_id);
            matched_title = Some(tmdb_title);
            matched_year = tmdb_year;
        }
    }

    let streams = do_search_rd_streams(
        pool.inner(),
        &title,
        year,
        &media_type,
        matched_tmdb_id,
    )
    .await
    .unwrap_or_default();

    Ok(DebridSearchResult {
        matched_title: matched_title.or(Some(title.clone())),
        matched_year,
        matched_tmdb_id,
        media_type: media_type.clone(),
        streams,
    })
}
