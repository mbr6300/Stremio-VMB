use reqwest::Client;
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.perplexity.ai/chat/completions";

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Option<Vec<Choice>>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Option<Message>,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaClassification {
    pub title: String,
    pub media_type: String,
    pub series_name: Option<String>,
}

pub async fn classify_media_titles(
    api_key: &str,
    titles: &[String],
) -> Result<Vec<MediaClassification>, String> {
    if titles.is_empty() {
        return Ok(Vec::new());
    }

    let list = titles
        .iter()
        .enumerate()
        .map(|(i, t)| format!("{}. {}", i + 1, t))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"Classify these media file titles (from filenames). For each title, determine:
1. media_type: "movie" or "series" (use "series" if it's a TV show episode)
2. series_name: If it's a series episode, the canonical series name for grouping (e.g. "Breaking Bad", "The Office"). If movie, use null.

Rules:
- S01E01, 1x01, Episode X, Season X = series
- Standalone film titles = movie
- "The Office" US vs UK are different series - include country if needed (e.g. "The Office (US)")
- Return ONLY valid JSON, no markdown or explanation

Return a JSON array with one object per title, in the same order. Each object: {{"title":"original title","media_type":"movie|series","series_name":"Name or null"}}

Titles:
{}"#,
        list
    );

    let client = Client::new();
    let resp = client
        .post(API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&ChatRequest {
            model: "sonar".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: 4096,
            temperature: 0.1,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Perplexity API error {}: {}", status, body));
    }

    let data: ChatResponse = resp.json().await.map_err(|e| e.to_string())?;
    let content = data
        .choices
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.message)
        .and_then(|m| m.content)
        .ok_or("No response content")?;

    let content = content.trim();
    let content = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(content)
        .trim();

    let results: Vec<MediaClassification> = serde_json::from_str(content).map_err(|e| {
        format!("Failed to parse Perplexity response: {}. Raw: {}", e, &content[..content.len().min(200)])
    })?;

    if results.len() != titles.len() {
        log::warn!(
            "Perplexity returned {} items, expected {}",
            results.len(),
            titles.len()
        );
    }

    Ok(results)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRecommendation {
    pub title: String,
    pub year: Option<i32>,
}

pub async fn get_ai_recommendations(
    api_key: &str,
    preset: &str,
    favorite_films: Option<&str>,
) -> Result<Vec<AiRecommendation>, String> {
    let prompt = match preset {
        "my_taste" => {
            let favs = favorite_films
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("keine angegeben");
            format!(
                r#"Liste 12 Filmempfehlungen, die zu diesen Lieblingsfilmen passen: {}.
Gib Filme zurück, die ähnlichen Geschmack/Genre/Stil haben.
Antworte NUR mit einem JSON-Array. Jedes Objekt: {{"title":"Filmtitel","year":2020 oder null}}
Beispiel: [{{"title":"Inception","year":2010}},{{"title":"Interstellar","year":2014}}]"#,
                favs
            )
        }
        "70s" => "Liste die 12 besten Filme der 1970er Jahre. Antworte NUR mit JSON-Array: [{\"title\":\"...\",\"year\":1974},...]".to_string(),
        "80s" => "Liste die 12 besten Filme der 1980er Jahre. Antworte NUR mit JSON-Array: [{\"title\":\"...\",\"year\":1984},...]".to_string(),
        "90s" => "Liste die 12 besten Filme der 1990er Jahre. Antworte NUR mit JSON-Array: [{\"title\":\"...\",\"year\":1994},...]".to_string(),
        "00s" => "Liste die 12 besten Filme der 2000er Jahre (2000-2009). Antworte NUR mit JSON-Array: [{\"title\":\"...\",\"year\":2004},...]".to_string(),
        "2010s" => "Liste die 12 besten Filme der 2010er Jahre (2010-2019). Antworte NUR mit JSON-Array: [{\"title\":\"...\",\"year\":2014},...]".to_string(),
        "action" => "Liste die 12 besten Action-Filme aller Zeiten. Antworte NUR mit JSON-Array: [{\"title\":\"...\",\"year\":2010},...]".to_string(),
        "comedy" => "Liste die 12 besten Comedy-Filme aller Zeiten. Antworte NUR mit JSON-Array: [{\"title\":\"...\",\"year\":2010},...]".to_string(),
        "drama" => "Liste die 12 besten Drama-Filme aller Zeiten. Antworte NUR mit JSON-Array: [{\"title\":\"...\",\"year\":2010},...]".to_string(),
        "thriller" => "Liste die 12 besten Thriller-Filme aller Zeiten. Antworte NUR mit JSON-Array: [{\"title\":\"...\",\"year\":2010},...]".to_string(),
        _ => return Err(format!("Unbekannter Preset: {}", preset)),
    };

    let client = Client::new();
    let resp = client
        .post(API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&ChatRequest {
            model: "sonar".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: 2048,
            temperature: 0.3,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Perplexity API error {}: {}", status, body));
    }

    let data: ChatResponse = resp.json().await.map_err(|e| e.to_string())?;
    let content = data
        .choices
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.message)
        .and_then(|m| m.content)
        .ok_or("No response content")?;

    let content = content.trim();
    let content = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(content)
        .trim();

    let results: Vec<AiRecommendation> = serde_json::from_str(content)
        .map_err(|e| format!("Parse error: {}. Raw: {}", e, &content[..content.len().min(300)]))?;

    Ok(results)
}

/// Holt Anekdoten und persönliche Infos zu einer Person (Schauspieler).
pub async fn get_person_anecdotes(
    api_key: &str,
    person_name: &str,
    known_for: Option<&str>,
) -> Result<PersonAnecdotes, String> {
    let context = known_for
        .filter(|s| !s.is_empty())
        .map(|s| format!("Bekannt für: {}. ", s))
        .unwrap_or_default();

    let prompt = format!(
        r#"Gib zu dieser Person kurze, interessante Infos auf Deutsch: {}
{}Antworte NUR mit einem JSON-Objekt (kein Markdown):
{{"anecdotes": ["Anekdote 1", "Anekdote 2", ...], "height": "1.85 m" oder null, "partner_status": "Verheiratet mit X" oder null, "children": "2 Kinder" oder null}}
Maximal 5 Anekdoten, jeweils 1-2 Sätze. Wenn keine Infos: leeres anecdotes-Array."#,
        person_name, context
    );

    let client = Client::new();
    let resp = client
        .post(API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&ChatRequest {
            model: "sonar".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: 1024,
            temperature: 0.2,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Perplexity API error {}: {}", status, body));
    }

    let data: ChatResponse = resp.json().await.map_err(|e| e.to_string())?;
    let content = data
        .choices
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.message)
        .and_then(|m| m.content)
        .ok_or("No response content")?;

    let content = content.trim();
    let content = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(content)
        .trim();

    serde_json::from_str(content).map_err(|e| format!("Parse error: {}", e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonAnecdotes {
    pub anecdotes: Vec<String>,
    pub height: Option<String>,
    pub partner_status: Option<String>,
    pub children: Option<String>,
}

/// Holt Trivia-Facts zu einem Film/Serie.
pub async fn get_film_trivia(
    api_key: &str,
    title: &str,
    year: Option<&str>,
    media_type: &str,
) -> Result<Vec<String>, String> {
    let year_ctx = year
        .filter(|s| !s.is_empty())
        .map(|s| format!(" ({})", s))
        .unwrap_or_default();
    let typ = if media_type == "series" { "Serie" } else { "Film" };

    let prompt = format!(
        r#"Liste 5-8 kurze, interessante Trivia-Facts zu dem {} "{}"{}. 
Antworte NUR mit einem JSON-Array von Strings: ["Fact 1", "Fact 2", ...]
Jeder Fact 1-2 Sätze. Hintergrundinfos, Dreharbeiten, Easter Eggs, etc."#,
        typ, title, year_ctx
    );

    let client = Client::new();
    let resp = client
        .post(API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&ChatRequest {
            model: "sonar".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: 1024,
            temperature: 0.2,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Perplexity API error {}: {}", status, body));
    }

    let data: ChatResponse = resp.json().await.map_err(|e| e.to_string())?;
    let content = data
        .choices
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.message)
        .and_then(|m| m.content)
        .ok_or("No response content")?;

    let content = content.trim();
    let content = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(content)
        .trim();

    let arr: Vec<String> = serde_json::from_str(content)
        .map_err(|e| format!("Parse error: {}", e))?;
    Ok(arr)
}
