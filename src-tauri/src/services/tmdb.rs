use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use super::metadata_provider::{
    CastCrew, CastMember, CrewMember, MetadataProvider, MetadataResult, SearchQuery,
};

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMG: &str = "https://image.tmdb.org/t/p";
const TMDB_PROFILE_IMG: &str = "https://image.tmdb.org/t/p/w185";

pub struct TmdbProvider {
    api_key: String,
    client: Client,
}

impl TmdbProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    fn poster_url(path: &str) -> String {
        format!("{}/w500{}", TMDB_IMG, path)
    }

    fn backdrop_url(path: &str) -> String {
        format!("{}/w1280{}", TMDB_IMG, path)
    }
}

// --- TMDb response types ---

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<SearchEntry>,
}

#[derive(Debug, Deserialize)]
struct SearchEntry {
    id: i64,
    #[serde(alias = "name")]
    title: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    #[serde(alias = "first_air_date")]
    release_date: Option<String>,
    vote_average: Option<f64>,
    genre_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
struct Genre {
    name: String,
}

#[derive(Debug, Deserialize)]
struct MovieDetail {
    id: i64,
    title: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    release_date: Option<String>,
    vote_average: Option<f64>,
    runtime: Option<i64>,
    genres: Option<Vec<Genre>>,
}

#[derive(Debug, Deserialize)]
struct TvDetail {
    id: i64,
    name: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    first_air_date: Option<String>,
    vote_average: Option<f64>,
    episode_run_time: Option<Vec<i64>>,
    genres: Option<Vec<Genre>>,
}

#[derive(Debug, Deserialize)]
struct CreditsCast {
    id: Option<i64>,
    name: String,
    character: Option<String>,
    profile_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreditsCrew {
    name: String,
    job: String,
    department: String,
    profile_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreditsResponse {
    cast: Vec<CreditsCast>,
    crew: Vec<CreditsCrew>,
}

// --- trait implementation ---

#[async_trait]
impl MetadataProvider for TmdbProvider {
    fn name(&self) -> &str {
        "tmdb"
    }

    async fn search(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<MetadataResult>, String> {
        let endpoint = if query.media_type == "series" { "tv" } else { "movie" };
        let mut url = format!(
            "{}/search/{}?api_key={}&query={}&language=de-DE",
            TMDB_BASE, endpoint, self.api_key,
            urlencoded(&query.title),
        );
        if let Some(year) = query.year {
            let param = if query.media_type == "series" {
                "first_air_date_year"
            } else {
                "year"
            };
            url.push_str(&format!("&{}={}", param, year));
        }

        let resp = self.client.get(&url).send().await.map_err(|e| e.to_string())?;
        let raw_text = resp.text().await.map_err(|e| e.to_string())?;
        let search_resp: SearchResponse =
            serde_json::from_str(&raw_text).map_err(|e| e.to_string())?;

        let results = search_resp
            .results
            .into_iter()
            .take(5)
            .map(|entry| MetadataResult {
                provider_id: entry.id,
                title: entry.title.unwrap_or_default(),
                overview: entry.overview.unwrap_or_default(),
                poster_url: entry.poster_path.as_deref().map(Self::poster_url),
                backdrop_url: entry.backdrop_path.as_deref().map(Self::backdrop_url),
                release_date: entry.release_date,
                rating: entry.vote_average,
                runtime: None,
                genres: entry
                    .genre_ids
                    .unwrap_or_default()
                    .iter()
                    .map(|id| id.to_string())
                    .collect(),
                cast_crew: None,
                raw_response: String::new(),
            })
            .collect();

        Ok(results)
    }

    async fn fetch_details(
        &self,
        provider_id: i64,
        media_type: &str,
    ) -> Result<Option<MetadataResult>, String> {
        let endpoint = if media_type == "series" { "tv" } else { "movie" };

        let detail_url = format!(
            "{}/{}/{}?api_key={}&language=de-DE",
            TMDB_BASE, endpoint, provider_id, self.api_key
        );
        let credits_url = format!(
            "{}/{}/{}/credits?api_key={}&language=de-DE",
            TMDB_BASE, endpoint, provider_id, self.api_key
        );

        let (detail_resp, credits_resp) = tokio::join!(
            self.client.get(&detail_url).send(),
            self.client.get(&credits_url).send(),
        );

        let detail_text = detail_resp
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let raw_detail: Value =
            serde_json::from_str(&detail_text).map_err(|e| e.to_string())?;

        let credits = match credits_resp {
            Ok(r) => r.json::<CreditsResponse>().await.ok(),
            Err(_) => None,
        };

        let cast_crew = credits.map(|c| CastCrew {
            cast: c
                .cast
                .into_iter()
                .take(20)
                .map(|m| CastMember {
                    id: m.id,
                    name: m.name,
                    character: m.character,
                    profile_url: m.profile_path.map(|p| format!("{}{}", TMDB_PROFILE_IMG, p)),
                })
                .collect(),
            crew: c
                .crew
                .into_iter()
                .filter(|m| {
                    matches!(
                        m.job.as_str(),
                        "Director" | "Screenplay" | "Writer" | "Producer"
                            | "Executive Producer" | "Creator"
                    )
                })
                .map(|m| CrewMember {
                    name: m.name,
                    job: m.job,
                    department: m.department,
                    profile_url: m.profile_path.map(|p| format!("{}{}", TMDB_PROFILE_IMG, p)),
                })
                .collect(),
        });

        if media_type == "series" {
            let detail: TvDetail =
                serde_json::from_value(raw_detail.clone()).map_err(|e| e.to_string())?;
            let runtime = detail
                .episode_run_time
                .as_ref()
                .and_then(|v| v.first().copied());

            Ok(Some(MetadataResult {
                provider_id: detail.id,
                title: detail.name.unwrap_or_default(),
                overview: detail.overview.unwrap_or_default(),
                poster_url: detail.poster_path.as_deref().map(Self::poster_url),
                backdrop_url: detail.backdrop_path.as_deref().map(Self::backdrop_url),
                release_date: detail.first_air_date,
                rating: detail.vote_average,
                runtime,
                genres: detail
                    .genres
                    .unwrap_or_default()
                    .into_iter()
                    .map(|g| g.name)
                    .collect(),
                cast_crew,
                raw_response: raw_detail.to_string(),
            }))
        } else {
            let detail: MovieDetail =
                serde_json::from_value(raw_detail.clone()).map_err(|e| e.to_string())?;

            Ok(Some(MetadataResult {
                provider_id: detail.id,
                title: detail.title.unwrap_or_default(),
                overview: detail.overview.unwrap_or_default(),
                poster_url: detail.poster_path.as_deref().map(Self::poster_url),
                backdrop_url: detail.backdrop_path.as_deref().map(Self::backdrop_url),
                release_date: detail.release_date,
                rating: detail.vote_average,
                runtime: detail.runtime,
                genres: detail
                    .genres
                    .unwrap_or_default()
                    .into_iter()
                    .map(|g| g.name)
                    .collect(),
                cast_crew,
                raw_response: raw_detail.to_string(),
            }))
        }
    }
}

fn urlencoded(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            c if c.is_alphanumeric() || "-_.~".contains(c) => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}
