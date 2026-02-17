use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMG: &str = "https://image.tmdb.org/t/p/w342";

pub struct DiscoveryProvider {
    pub id: &'static str,
    pub name: &'static str,
    pub tmdb_provider_id: u32,
}

pub const PROVIDERS: &[DiscoveryProvider] = &[
    DiscoveryProvider { id: "netflix",  name: "Netflix",            tmdb_provider_id: 8 },
    DiscoveryProvider { id: "prime",    name: "Prime Video",        tmdb_provider_id: 9 },
    DiscoveryProvider { id: "disney",   name: "Disney+",            tmdb_provider_id: 337 },
    DiscoveryProvider { id: "apple",    name: "Apple TV+",          tmdb_provider_id: 350 },
];

#[derive(Debug, Clone)]
pub struct DiscoverItem {
    pub tmdb_id: i64,
    pub media_type: String,
    pub title: String,
    pub year: Option<i32>,
    pub rating: Option<f64>,
    pub poster_url: Option<String>,
    pub overview: String,
    pub raw_json: String,
}

#[derive(Debug, Clone)]
pub struct DiscoverListResult {
    pub list_type: String,
    pub provider: Option<String>,
    pub country: String,
    pub title: String,
    pub items: Vec<DiscoverItem>,
}

#[derive(Debug, Deserialize)]
struct TmdbPage {
    results: Vec<Value>,
}

fn parse_movie(v: &Value) -> Option<DiscoverItem> {
    let id = v.get("id")?.as_i64()?;
    let title = v.get("title").and_then(|t| t.as_str()).unwrap_or("").to_string();
    if title.is_empty() { return None; }
    let year = v.get("release_date")
        .and_then(|d| d.as_str())
        .and_then(|d| d.get(..4))
        .and_then(|y| y.parse::<i32>().ok());
    Some(DiscoverItem {
        tmdb_id: id,
        media_type: "movie".into(),
        title,
        year,
        rating: v.get("vote_average").and_then(|r| r.as_f64()),
        poster_url: v.get("poster_path").and_then(|p| p.as_str()).map(|p| format!("{}{}", TMDB_IMG, p)),
        overview: v.get("overview").and_then(|o| o.as_str()).unwrap_or("").to_string(),
        raw_json: v.to_string(),
    })
}

fn parse_tv(v: &Value) -> Option<DiscoverItem> {
    let id = v.get("id")?.as_i64()?;
    let title = v.get("name").and_then(|t| t.as_str()).unwrap_or("").to_string();
    if title.is_empty() { return None; }
    let year = v.get("first_air_date")
        .and_then(|d| d.as_str())
        .and_then(|d| d.get(..4))
        .and_then(|y| y.parse::<i32>().ok());
    Some(DiscoverItem {
        tmdb_id: id,
        media_type: "tv".into(),
        title,
        year,
        rating: v.get("vote_average").and_then(|r| r.as_f64()),
        poster_url: v.get("poster_path").and_then(|p| p.as_str()).map(|p| format!("{}{}", TMDB_IMG, p)),
        overview: v.get("overview").and_then(|o| o.as_str()).unwrap_or("").to_string(),
        raw_json: v.to_string(),
    })
}

pub async fn fetch_top_rated_movies(api_key: &str, country: &str) -> Result<DiscoverListResult, String> {
    let client = Client::new();
    let lang = language_for_country(country);
    let mut all_items = Vec::new();

    for page in 1..=3 {
        let url = format!(
            "{}/movie/top_rated?api_key={}&language={}&region={}&page={}",
            TMDB_BASE, api_key, lang, country, page
        );
        let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
        let data: TmdbPage = resp.json().await.map_err(|e| e.to_string())?;
        all_items.extend(data.results.iter().filter_map(parse_movie));
    }

    Ok(DiscoverListResult {
        list_type: "imdb_top".into(),
        provider: None,
        country: country.into(),
        title: "Top IMDb Filme".into(),
        items: all_items,
    })
}

pub async fn fetch_provider_movies(
    api_key: &str,
    provider: &DiscoveryProvider,
    country: &str,
) -> Result<DiscoverListResult, String> {
    let client = Client::new();
    let lang = language_for_country(country);
    let mut all_items = Vec::new();

    for page in 1..=2 {
        let url = format!(
            "{}/discover/movie?api_key={}&language={}&watch_region={}&with_watch_providers={}&sort_by=popularity.desc&page={}",
            TMDB_BASE, api_key, lang, country, provider.tmdb_provider_id, page
        );
        let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
        let data: TmdbPage = resp.json().await.map_err(|e| e.to_string())?;
        all_items.extend(data.results.iter().filter_map(parse_movie));
    }

    Ok(DiscoverListResult {
        list_type: "streaming_popular".into(),
        provider: Some(provider.id.into()),
        country: country.into(),
        title: format!("Top Filme auf {}", provider.name),
        items: all_items,
    })
}

pub async fn fetch_provider_series(
    api_key: &str,
    provider: &DiscoveryProvider,
    country: &str,
) -> Result<DiscoverListResult, String> {
    let client = Client::new();
    let lang = language_for_country(country);
    let mut all_items = Vec::new();

    for page in 1..=2 {
        let url = format!(
            "{}/discover/tv?api_key={}&language={}&watch_region={}&with_watch_providers={}&sort_by=popularity.desc&page={}",
            TMDB_BASE, api_key, lang, country, provider.tmdb_provider_id, page
        );
        let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
        let data: TmdbPage = resp.json().await.map_err(|e| e.to_string())?;
        all_items.extend(data.results.iter().filter_map(parse_tv));
    }

    Ok(DiscoverListResult {
        list_type: "streaming_popular".into(),
        provider: Some(provider.id.into()),
        country: country.into(),
        title: format!("Top Serien auf {}", provider.name),
        items: all_items,
    })
}

pub async fn fetch_all_lists(api_key: &str, country: &str) -> Result<Vec<DiscoverListResult>, String> {
    let mut results = Vec::new();

    results.push(fetch_top_rated_movies(api_key, country).await?);

    for provider in PROVIDERS {
        results.push(fetch_provider_movies(api_key, provider, country).await?);
        results.push(fetch_provider_series(api_key, provider, country).await?);
    }

    Ok(results)
}

fn language_for_country(country: &str) -> &str {
    match country {
        "CH" | "DE" | "AT" => "de-DE",
        "FR" => "fr-FR",
        "IT" => "it-IT",
        _ => "en-US",
    }
}

// --- Stream Search (TMDb multi-search) ---

#[derive(Debug, Deserialize)]
struct MultiSearchResponse {
    results: Option<Vec<MultiSearchEntry>>,
}

#[derive(Debug, Deserialize)]
struct MultiSearchEntry {
    id: Option<i64>,
    #[serde(alias = "name")]
    title: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    #[serde(alias = "first_air_date")]
    release_date: Option<String>,
    vote_average: Option<f64>,
    genre_ids: Option<Vec<i64>>,
    #[serde(rename = "media_type")]
    media_type: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StreamSearchResult {
    pub tmdb_id: i64,
    pub media_type: String,
    pub title: String,
    pub year: Option<i32>,
    pub rating: Option<f64>,
    pub poster_url: Option<String>,
    pub overview: String,
    pub genre_ids: Vec<i64>,
}

pub async fn search_streams(
    api_key: &str,
    query: &str,
    media_type: &str,
    genre_id: Option<i64>,
) -> Result<Vec<StreamSearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let client = Client::new();
    let encoded: String = query
        .chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            c if c.is_alphanumeric() || "-_.~".contains(c) => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect();

    let url = format!(
        "{}/search/multi?api_key={}&query={}&language=de-DE&include_adult=false",
        TMDB_BASE, api_key, encoded
    );

    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let data: MultiSearchResponse = resp.json().await.map_err(|e| e.to_string())?;

    let results = data.results.unwrap_or_default();

    let mut out: Vec<StreamSearchResult> = results
        .into_iter()
        .filter_map(|e| {
            let mt = e.media_type.as_deref().unwrap_or("");
            if mt != "movie" && mt != "tv" {
                return None;
            }
            let filter_type = match media_type {
                "movie" => "movie",
                "tv" | "series" => "tv",
                _ => "all",
            };
            if filter_type != "all" && mt != filter_type {
                return None;
            }
            let id = e.id?;
            let title = e.title.unwrap_or_default();
            if title.is_empty() {
                return None;
            }
            let genre_ids = e.genre_ids.unwrap_or_default();
            if let Some(gid) = genre_id {
                if !genre_ids.contains(&gid) {
                    return None;
                }
            }
            let year = e
                .release_date
                .as_deref()
                .and_then(|d| d.get(..4))
                .and_then(|y| y.parse::<i32>().ok());

            Some(StreamSearchResult {
                tmdb_id: id,
                media_type: if mt == "tv" { "tv".into() } else { "movie".into() },
                title,
                year,
                rating: e.vote_average,
                poster_url: e
                    .poster_path
                    .as_deref()
                    .map(|p| format!("{}{}", TMDB_IMG, p)),
                overview: e.overview.unwrap_or_default(),
                genre_ids,
            })
        })
        .collect();

    out.dedup_by(|a, b| a.tmdb_id == b.tmdb_id && a.media_type == b.media_type);
    Ok(out)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TmdbGenre {
    pub id: i64,
    pub name: String,
    pub media_type: String,
}

#[derive(Debug, Deserialize)]
struct GenreListResponse {
    genres: Vec<GenreEntry>,
}

#[derive(Debug, Deserialize)]
struct GenreEntry {
    id: i64,
    name: String,
}

pub async fn get_tmdb_genres(api_key: &str) -> Result<Vec<TmdbGenre>, String> {
    let client = Client::new();
    let mut out = Vec::new();

    for (endpoint, mt) in [("movie", "movie"), ("tv", "tv")] {
        let url = format!(
            "{}/genre/{}/list?api_key={}&language=de-DE",
            TMDB_BASE, endpoint, api_key
        );
        let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
        let data: GenreListResponse = resp.json().await.map_err(|e| e.to_string())?;
        for g in data.genres {
            out.push(TmdbGenre {
                id: g.id,
                name: g.name,
                media_type: mt.to_string(),
            });
        }
    }

    Ok(out)
}
