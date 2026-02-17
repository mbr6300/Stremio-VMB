//! Erweiterte TMDb-API: Person-Details, Extended Info, Actor-Suggestions.

use reqwest::Client;
use serde::Deserialize;

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMG: &str = "https://image.tmdb.org/t/p";
const TMDB_PROFILE: &str = "https://image.tmdb.org/t/p/w185";

#[derive(Debug, serde::Serialize)]
pub struct PersonDetails {
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

#[derive(Debug, serde::Serialize)]
pub struct MediaExtendedInfo {
    pub tagline: Option<String>,
    pub trivia_facts: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ActorMovieSuggestion {
    pub tmdb_id: i64,
    pub title: String,
    pub year: Option<i32>,
    pub poster_url: Option<String>,
    pub media_type: String,
    pub match_reason: String,
}

#[derive(Debug, Deserialize)]
struct PersonResponse {
    id: i64,
    name: Option<String>,
    biography: Option<String>,
    profile_path: Option<String>,
    birthday: Option<String>,
    deathday: Option<String>,
    place_of_birth: Option<String>,
    known_for_department: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MovieCreditsCast {
    id: Option<i64>,
    title: Option<String>,
    release_date: Option<String>,
    poster_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MovieCreditsResponse {
    cast: Vec<MovieCreditsCast>,
}

fn profile_url(path: &str) -> String {
    format!("{}{}", TMDB_PROFILE, path)
}

fn poster_url(path: &str) -> String {
    format!("{}/w342{}", TMDB_IMG, path)
}

pub async fn fetch_person_details(api_key: &str, person_id: i64) -> Result<PersonDetails, String> {
    let url = format!(
        "{}/person/{}?api_key={}&language=de-DE",
        TMDB_BASE, person_id, api_key
    );
    let client = Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let p: PersonResponse = resp.json().await.map_err(|e| e.to_string())?;

    let age = p.birthday.as_ref().and_then(|b| {
        chrono::NaiveDate::parse_from_str(b, "%Y-%m-%d")
            .ok()
            .map(|birth| {
                let today = chrono::Local::now().naive_local().date();
                (today.signed_duration_since(birth).num_days() / 365) as i32
            })
    });

    Ok(PersonDetails {
        id: p.id,
        name: p.name.unwrap_or_default(),
        biography: p.biography.filter(|s| !s.trim().is_empty()),
        profile_url: p.profile_path.as_deref().map(profile_url),
        birthday: p.birthday,
        deathday: p.deathday,
        place_of_birth: p.place_of_birth,
        known_for_department: p.known_for_department,
        age: age.map(|d| d as i32),
        anecdotes: Vec::new(),
        height: None,
        partner_status: None,
        children: None,
    })
}

pub async fn fetch_media_extended(
    api_key: &str,
    tmdb_id: i64,
    media_type: &str,
) -> Result<MediaExtendedInfo, String> {
    let endpoint = if media_type == "series" { "tv" } else { "movie" };
    let url = format!(
        "{}/{}/{}?api_key={}&language=de-DE",
        TMDB_BASE, endpoint, tmdb_id, api_key
    );
    let client = Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let tagline = json.get("tagline").and_then(|v| v.as_str()).map(String::from);
    Ok(MediaExtendedInfo {
        tagline,
        trivia_facts: Vec::new(),
    })
}

pub async fn fetch_actor_movie_suggestions(
    api_key: &str,
    tmdb_id: i64,
    media_type: &str,
    actor_ids: &[i64],
    director_ids: &[i64],
    limit: usize,
) -> Result<Vec<ActorMovieSuggestion>, String> {
    let mut results = Vec::new();
    for &actor_id in actor_ids.iter().take(5) {
        let url = format!(
            "{}/person/{}/movie_credits?api_key={}&language=de-DE",
            TMDB_BASE, actor_id, api_key
        );
        let client = Client::new();
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(data) = resp.json::<MovieCreditsResponse>().await {
                for c in data.cast.into_iter().take(limit) {
                    if let (Some(id), Some(title)) = (c.id, c.title) {
                        let year = c
                            .release_date
                            .as_deref()
                            .and_then(|s| s.get(..4))
                            .and_then(|s| s.parse::<i32>().ok());
                        let poster_url = c.poster_path.as_deref().map(poster_url);
                        results.push(ActorMovieSuggestion {
                            tmdb_id: id,
                            title,
                            year,
                            poster_url,
                            media_type: "movie".to_string(),
                            match_reason: "Mit diesem Schauspieler".to_string(),
                        });
                    }
                }
            }
        }
    }
    results.dedup_by(|a, b| a.tmdb_id == b.tmdb_id);
    results.truncate(limit);
    Ok(results)
}
