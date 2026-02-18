//! Synology QuickConnect: Verbindungsprüfung und Pfad-Auflösung.

use reqwest::Client;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, serde::Serialize)]
pub struct QuickConnectStatus {
    pub connected: bool,
    pub message: String,
    pub server_url: Option<String>,
}

/// Prüft, ob die Synology NAS über QuickConnect erreichbar ist.
pub async fn check_quickconnect(quickconnect_id: &str) -> QuickConnectStatus {
    let id = quickconnect_id.trim();
    if id.is_empty() {
        return QuickConnectStatus {
            connected: false,
            message: "QuickConnect-ID fehlt.".to_string(),
            server_url: None,
        };
    }

    let client = match Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => Client::new(),
    };

    let urls_to_try = [
        format!("https://{}.quickconnect.to", id),
        format!("https://global.quickconnect.to/#!/{}", id),
    ];

    for url in &urls_to_try {
        match client
            .get(url)
            .header("User-Agent", "Stremio-VMB/1.0")
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() || status.is_redirection() {
                    return QuickConnectStatus {
                        connected: true,
                        message: "Verbunden".to_string(),
                        server_url: Some(format!("https://{}.quickconnect.to", id)),
                    };
                }
            }
            Err(e) => {
                log::debug!("QuickConnect check {}: {}", url, e);
            }
        }
    }

    QuickConnectStatus {
        connected: false,
        message: "Nicht erreichbar. Prüfe QuickConnect-ID und Internetverbindung.".to_string(),
        server_url: Some(format!("https://{}.quickconnect.to", id)),
    }
}

/// Prüft, ob ein lokaler Pfad erreichbar ist.
pub fn is_local_path_accessible(path: &str) -> bool {
    Path::new(path.trim()).exists()
}

/// Prüft, ob der Pfad lokal erreichbar ist. Bevorzugt lokalen Zugriff.
pub fn prefer_local_path(path: &str) -> bool {
    is_local_path_accessible(path)
}
