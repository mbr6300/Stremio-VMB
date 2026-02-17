use reqwest::Client;
use serde::{Deserialize, Serialize};

const RD_BASE_URL: &str = "https://api.real-debrid.com";
const RD_OPEN_CLIENT_ID: &str = "X245A4XAIBGVM";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub interval: u64,
    pub expires_in: u64,
    pub verification_url: String,
    pub direct_verification_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialsResponse {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrestrictedLink {
    pub id: String,
    pub filename: String,
    pub filesize: u64,
    pub link: String,
    pub host: String,
    pub download: String,
    pub streamable: u8,
}

pub async fn get_device_code() -> Result<DeviceCodeResponse, String> {
    let client = Client::new();
    let url = format!("{}/oauth/v2/device/code?client_id={}&new_credentials=yes", RD_BASE_URL, RD_OPEN_CLIENT_ID);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response
        .json::<DeviceCodeResponse>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn poll_credentials(device_code: &str) -> Result<Option<CredentialsResponse>, String> {
    let client = Client::new();
    let url = format!(
        "{}/oauth/v2/device/credentials?client_id={}&code={}",
        RD_BASE_URL, RD_OPEN_CLIENT_ID, device_code
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status() == 403 {
        return Ok(None);
    }

    let creds = response
        .json::<CredentialsResponse>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(Some(creds))
}

pub async fn get_token(client_id: &str, client_secret: &str, device_code: &str) -> Result<TokenResponse, String> {
    let client = Client::new();
    let url = format!("{}/oauth/v2/token", RD_BASE_URL);

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", device_code),
        ("grant_type", "http://oauth.net/grant_type/device/1.0"),
    ];

    let response = client
        .post(&url)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response
        .json::<TokenResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdUser {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub premium: u64,
    pub expiration: String,
}

pub async fn get_user(access_token: &str) -> Result<RdUser, String> {
    let client = Client::new();
    let url = format!("{}/rest/1.0/user", RD_BASE_URL);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("RealDebrid API error: {}", response.status()));
    }

    response
        .json::<RdUser>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn unrestrict_link(access_token: &str, link: &str) -> Result<UnrestrictedLink, String> {
    let client = Client::new();
    let url = format!("{}/rest/1.0/unrestrict/link", RD_BASE_URL);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .form(&[("link", link)])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response
        .json::<UnrestrictedLink>()
        .await
        .map_err(|e| e.to_string())
}
