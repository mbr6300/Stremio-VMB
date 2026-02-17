use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

const RD_REST: &str = "https://api.real-debrid.com/rest/1.0";
const TORRENTIO_BASE: &str = "https://torrentio.strem.fun";
const TMDB_BASE: &str = "https://api.themoviedb.org/3";

#[derive(Debug, Clone, serde::Serialize)]
pub struct RdStreamLink {
    pub title: String,
    pub quality: String,
    pub size: String,
    pub stream_url: String,
}

fn resolution_rank(quality: &str) -> u32 {
    let q = quality.to_lowercase();
    if q.contains("2160") || q.contains("4k") || q.contains("uhd") {
        100
    } else if q.contains("1080") || q.contains("full hd") {
        80
    } else if q.contains("720") || q.contains("hd") {
        60
    } else if q.contains("480") || q.contains("sd") {
        40
    } else if q.contains("bdrip") || q.contains("bluray") {
        75
    } else if q.contains("web-dl") || q.contains("webdl") {
        70
    } else if q.contains("dvd") || q.contains("cam") {
        20
    } else {
        50
    }
}

pub fn sort_streams_by_resolution(streams: &mut [RdStreamLink]) {
    streams.sort_by(|a, b| {
        let ra = resolution_rank(&a.quality);
        let rb = resolution_rank(&b.quality);
        rb.cmp(&ra).then_with(|| a.quality.cmp(&b.quality))
    });
}

#[derive(Debug, Deserialize)]
struct YtsResponse {
    data: Option<YtsData>,
}

#[derive(Debug, Deserialize)]
struct YtsData {
    movies: Option<Vec<YtsMovie>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct YtsMovie {
    title: Option<String>,
    year: Option<u32>,
    torrents: Option<Vec<YtsTorrent>>,
}

#[derive(Debug, Deserialize)]
struct YtsTorrent {
    url: Option<String>,
    quality: Option<String>,
    size: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RdAddMagnetResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RdTorrentInfo {
    id: String,
    status: String,
    links: Option<Vec<String>>,
    files: Option<Vec<RdTorrentFile>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RdTorrentFile {
    id: i64,
    path: String,
    bytes: i64,
}

#[derive(Debug, Deserialize)]
struct TmdbExternalIds {
    imdb_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TmdbSearchResponse {
    results: Option<Vec<TmdbSearchResult>>,
}

#[derive(Debug, Deserialize)]
struct TmdbSearchResult {
    id: i64,
    #[serde(alias = "name")]
    title: Option<String>,
    #[serde(alias = "first_air_date")]
    release_date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TorrentioStream {
    name: Option<String>,
    title: Option<String>,
    #[serde(alias = "infoHash")]
    info_hash: Option<String>,
    #[serde(alias = "fileIdx")]
    file_idx: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TorrentioResponse {
    streams: Option<Vec<TorrentioStream>>,
}

/// Stremio addon stream (Debridio etc.) - supports direct url or infoHash
#[derive(Debug, Deserialize)]
struct AddonStream {
    name: Option<String>,
    title: Option<String>,
    #[serde(alias = "infoHash")]
    info_hash: Option<String>,
    #[serde(alias = "fileIdx")]
    file_idx: Option<i64>,
    url: Option<String>,
    tag: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AddonStreamResponse {
    streams: Option<Vec<AddonStream>>,
}

pub fn extract_debridio_base_url(url: &str) -> Option<String> {
    let s = url.trim();
    if s.is_empty() || !s.starts_with("http") {
        return None;
    }
    let without_query = s.split('?').next().unwrap_or(s);
    let base = without_query
        .trim_end_matches('/')
        .trim_end_matches("manifest.json")
        .trim_end_matches('/');
    if base.is_empty() {
        None
    } else {
        Some(base.to_string())
    }
}

pub async fn fetch_debridio_streams(
    base_url: &str,
    imdb_id: &str,
    media_type: &str,
    season: Option<u32>,
    episode: Option<u32>,
    rd_token: Option<&str>,
) -> Result<Vec<RdStreamLink>, String> {
    let stream_type = if media_type == "series" { "series" } else { "movie" };
    let path = if media_type == "series" {
        let s = season.unwrap_or(1);
        let e = episode.unwrap_or(1);
        format!("{}:{}:{}", imdb_id, s, e)
    } else {
        imdb_id.to_string()
    };
    let url = format!("{}/stream/{}/{}.json", base_url, stream_type, path);
    let client = Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let data: AddonStreamResponse = resp.json().await.map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    if let Some(streams) = data.streams {
        for s in streams {
            if let Some(direct_url) = s.url {
                let name = s.name.as_deref().unwrap_or("").trim().to_string();
                let quality = s.tag.as_deref().unwrap_or("")
                    .to_string();
                let quality = if quality.is_empty() {
                    extract_quality_from_name(&name)
                } else {
                    quality
                };
                let size = extract_size_from_title(&name);
                results.push(RdStreamLink {
                    title: s.title.unwrap_or_else(|| name.clone()),
                    quality,
                    size,
                    stream_url: direct_url,
                });
            } else if let Some(hash) = s.info_hash {
                if let Some(token) = rd_token {
                    let file_idx = s.file_idx.unwrap_or(0);
                    let name = s.name.as_deref().unwrap_or("").trim().to_string();
                    let quality = s.tag.as_deref().unwrap_or("").to_string();
                    let quality = if quality.is_empty() {
                        extract_quality_from_name(&name)
                    } else {
                        quality
                    };
                    let size = extract_size_from_title(&name);
                    if let Ok(Some(link)) =
                        info_hash_to_stream(token, &hash, file_idx, &quality, &size).await
                    {
                        results.push(link);
                    }
                }
            }
        }
    }
    Ok(results)
}

pub async fn search_tmdb_for_id(
    api_key: &str,
    title: &str,
    year: Option<u32>,
    media_type: &str,
) -> Result<Option<i64>, String> {
    search_tmdb_for_match(api_key, title, year, media_type)
        .await
        .map(|o| o.map(|(id, _, _)| id))
}

/// Returns (tmdb_id, title, year) for the first TMDb search match.
pub async fn search_tmdb_for_match(
    api_key: &str,
    title: &str,
    year: Option<u32>,
    media_type: &str,
) -> Result<Option<(i64, String, Option<i32>)>, String> {
    let endpoint = if media_type == "series" { "tv" } else { "movie" };
    let query = urlencode_query(title);
    let mut url = format!(
        "{}/search/{}?api_key={}&query={}&language=de-DE",
        TMDB_BASE, endpoint, api_key, query
    );
    if let Some(y) = year {
        let param = if media_type == "series" {
            "first_air_date_year"
        } else {
            "year"
        };
        url.push_str(&format!("&{}={}", param, y));
    }
    let client = Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let data: TmdbSearchResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data.results.and_then(|r| {
        r.into_iter().next().map(|r| {
            let id = r.id;
            let title = r.title.unwrap_or_default();
            let year = r
                .release_date
                .as_deref()
                .and_then(|d| d.get(..4))
                .and_then(|y| y.parse().ok());
            (id, title, year)
        })
    }))
}

pub async fn get_imdb_id(api_key: &str, tmdb_id: i64, media_type: &str) -> Result<Option<String>, String> {
    let endpoint = if media_type == "series" { "tv" } else { "movie" };
    let url = format!(
        "{}/{}/{}/external_ids?api_key={}",
        TMDB_BASE, endpoint, tmdb_id, api_key
    );
    let client = Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let data: TmdbExternalIds = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data.imdb_id)
}

pub async fn fetch_torrentio_streams(
    imdb_id: &str,
    media_type: &str,
    season: Option<u32>,
    episode: Option<u32>,
) -> Result<Vec<(String, i64, String, String)>, String> {
    let stream_type = if media_type == "series" { "series" } else { "movie" };
    let path = if media_type == "series" {
        let s = season.unwrap_or(1);
        let e = episode.unwrap_or(1);
        format!("{}:{}:{}", imdb_id, s, e)
    } else {
        imdb_id.to_string()
    };
    let url = format!("{}/stream/{}/{}.json", TORRENTIO_BASE, stream_type, path);
    let client = Client::new();
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let data: TorrentioResponse = resp.json().await.map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    if let Some(streams) = data.streams {
        for s in streams {
            if let Some(hash) = s.info_hash {
                let file_idx = s.file_idx.unwrap_or(0);
                let name = s.name.as_deref().unwrap_or("").replace("Torrentio\n", "").trim().to_string();
                let quality = extract_quality_from_name(&name);
                let size = extract_size_from_title(&name);
                results.push((hash, file_idx, quality, size));
            }
        }
    }
    Ok(results)
}

fn urlencode_query(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            c if c.is_alphanumeric() || "-_.~".contains(c) => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}

fn extract_quality_from_name(name: &str) -> String {
    let lines: Vec<&str> = name.lines().map(str::trim).filter(|s| !s.is_empty()).collect();
    for line in &lines {
        let lower = line.to_lowercase();
        if lower.starts_with("1080p") || lower.starts_with("720p") || lower.starts_with("2160p")
            || lower.starts_with("4k") || lower.starts_with("480p") || lower.starts_with("cam")
            || lower.starts_with("bdrip") || lower.starts_with("dvdrip") || lower.starts_with("web-dl")
        {
            return line.to_string();
        }
    }
    if let Some(first) = lines.first() {
        return first.to_string();
    }
    "?".to_string()
}

fn normalize_title_for_search(title: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let base = title
        .trim()
        .replace([':', ';', ',', '!', '?'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if !base.is_empty() {
        variants.push(base.clone());
    }
    let without_articles: String = title
        .trim()
        .split_whitespace()
        .filter(|w| {
            let l = w.to_lowercase();
            l != "the" && l != "a" && l != "an" && l != "die" && l != "der" && l != "das"
        })
        .collect::<Vec<_>>()
        .join(" ");
    if !without_articles.is_empty() && !variants.contains(&without_articles) {
        variants.push(without_articles);
    }
    let first_words: String = title
        .trim()
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ");
    if first_words.len() >= 3 && !variants.contains(&first_words) {
        variants.push(first_words);
    }
    variants
}

fn extract_size_from_title(title: &str) -> String {
    if let Some(start) = title.find("💾") {
        let rest = &title[start + 2..];
        if let Some(end) = rest.find(' ') {
            return rest[..end].trim().to_string();
        }
        return rest.split_whitespace().next().unwrap_or("?").to_string();
    }
    let lower = title.to_lowercase();
    for (suffix, unit) in [("gb", "GB"), ("mb", "MB")] {
        if let Some(pos) = lower.find(suffix) {
            let before = title[..pos].trim();
            let num_part: String = before
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            if !num_part.is_empty() {
                return format!("{} {}", num_part.trim(), unit);
            }
        }
    }
    "?".to_string()
}

const YTS_APIS: &[&str] = &[
    "https://yts.mx/api/v2/list_movies.json",
    "https://yts.lt/api/v2/list_movies.json",
];

pub async fn search_movie_torrents(title: &str, _year: Option<u32>) -> Result<Vec<(String, String, String)>, String> {
    let client = Client::new();
    let mut results = Vec::new();
    let variants = normalize_title_for_search(title);

    for query_term in variants.iter().take(3) {
        let query = query_term.replace(' ', "+");
        for &base in YTS_APIS {
            let url = format!("{}?query_term={}&limit=10", base, query);
            if let Ok(resp) = client.get(&url).send().await {
                if let Ok(data) = resp.json::<YtsResponse>().await {
                    if let Some(d) = data.data {
                        if let Some(movies) = d.movies {
                            for m in movies {
                                if let Some(torrents) = m.torrents {
                                    for t in torrents {
                                        if let Some(magnet) = t.url {
                                            if magnet.starts_with("magnet:") {
                                                let quality = t.quality.as_deref().unwrap_or("?").to_string();
                                                let size = t.size.as_deref().unwrap_or("?").to_string();
                                                results.push((magnet, quality, size));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !results.is_empty() {
                        return Ok(results);
                    }
                }
            }
        }
    }

    Ok(results)
}

pub async fn info_hash_to_stream(
    access_token: &str,
    info_hash: &str,
    file_idx: i64,
    quality: &str,
    size: &str,
) -> Result<Option<RdStreamLink>, String> {
    let hash_lower = info_hash.to_lowercase();
    let magnet = format!("magnet:?xt=urn:btih:{}", hash_lower);

    let client = Client::new();
    let add_url = format!("{}/torrents/addMagnet", RD_REST);
    let add_resp = client
        .post(&add_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .form(&[("magnet", magnet.as_str())])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !add_resp.status().is_success() {
        let err: Value = add_resp.json().await.unwrap_or(Value::Null);
        return Err(format!("RD addMagnet failed: {:?}", err));
    }

    let add_data: RdAddMagnetResponse = add_resp.json().await.map_err(|e| e.to_string())?;

    for _ in 0..45 {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let info_url = format!("{}/torrents/info/{}", RD_REST, add_data.id);
        let info_resp = client
            .get(&info_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let info: RdTorrentInfo = info_resp.json().await.map_err(|e| e.to_string())?;

        if info.status == "downloaded" {
            if let Some(links) = info.links {
                let idx = file_idx.max(0) as usize;
                let link = if idx < links.len() {
                    links.get(idx).cloned()
                } else {
                    links.first().cloned()
                };
                if let Some(link) = link {
                    let unrestrict = unrestrict_rd_link(access_token, &link).await?;
                    return Ok(Some(RdStreamLink {
                        title: unrestrict.filename,
                        quality: quality.to_string(),
                        size: size.to_string(),
                        stream_url: unrestrict.download,
                    }));
                }
            }
        } else if info.status == "waiting_files_selection" || info.status == "magnet_conversion" {
            if let Some(ref files) = info.files {
                if !files.is_empty() {
                    let idx = file_idx.max(0) as usize;
                    let file_id = if idx < files.len() {
                        files[idx].id
                    } else {
                        files[0].id
                    };
                    let select_url = format!("{}/torrents/selectFiles/{}", RD_REST, add_data.id);
                    let _ = client
                        .post(&select_url)
                        .header("Authorization", format!("Bearer {}", access_token))
                        .form(&[("files", file_id.to_string())])
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;
                }
            }
        } else if info.status == "error" || info.status == "dead" || info.status == "virus" {
            break;
        }
    }

    Ok(None)
}

pub async fn magnet_to_stream(
    access_token: &str,
    magnet: &str,
    quality: &str,
    size: &str,
) -> Result<Option<RdStreamLink>, String> {
    let client = Client::new();

    let add_url = format!("{}/torrents/addMagnet", RD_REST);
    let add_resp = client
        .post(&add_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .form(&[("magnet", magnet)])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !add_resp.status().is_success() {
        let err: Value = add_resp.json().await.unwrap_or(Value::Null);
        return Err(format!("RD addMagnet failed: {:?}", err));
    }

    let add_data: RdAddMagnetResponse = add_resp.json().await.map_err(|e| e.to_string())?;

    let select_url = format!("{}/torrents/selectFiles/{}", RD_REST, add_data.id);
    let _ = client
        .post(&select_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .form(&[("files", "all")])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    for _ in 0..30 {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let info_url = format!("{}/torrents/info/{}", RD_REST, add_data.id);
        let info_resp = client
            .get(&info_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let info: RdTorrentInfo = info_resp.json().await.map_err(|e| e.to_string())?;

        if info.status == "downloaded" {
            if let Some(links) = info.links {
                if let Some(link) = links.first() {
                    let unrestrict = unrestrict_rd_link(access_token, link).await?;
                    return Ok(Some(RdStreamLink {
                        title: unrestrict.filename,
                        quality: quality.to_string(),
                        size: size.to_string(),
                        stream_url: unrestrict.download,
                    }));
                }
            }
        } else if info.status == "error" || info.status == "dead" || info.status == "virus" {
            break;
        }
    }

    Ok(None)
}

#[derive(Debug, Deserialize)]
struct UnrestrictResponse {
    download: String,
    filename: String,
}

async fn unrestrict_rd_link(access_token: &str, link: &str) -> Result<UnrestrictResponse, String> {
    let client = Client::new();
    let url = format!("{}/unrestrict/link", RD_REST);

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .form(&[("link", link)])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    resp.json().await.map_err(|e| e.to_string())
}
