use crate::db::DbPool;
use crate::services::realdebrid as rd;
use crate::services::storage::{self, RealDebridToken};
use tauri::State;

#[tauri::command]
pub async fn rd_get_device_code() -> Result<rd::DeviceCodeResponse, String> {
    rd::get_device_code().await
}

#[tauri::command]
pub async fn rd_poll_credentials(
    pool: State<'_, DbPool>,
    device_code: String,
) -> Result<bool, String> {
    let creds = rd::poll_credentials(&device_code).await?;

    let creds = match creds {
        Some(c) => c,
        None => return Ok(false),
    };

    let token_response = rd::get_token(&creds.client_id, &creds.client_secret, &device_code).await?;
    let expires_at = chrono::Utc::now()
        + chrono::Duration::seconds(token_response.expires_in as i64);

    let token = RealDebridToken {
        id: 0,
        client_id: creds.client_id,
        client_secret: creds.client_secret,
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_at: expires_at.to_rfc3339(),
        created_at: String::new(),
    };

    storage::save_rd_token(&pool, &token).await?;
    Ok(true)
}

#[tauri::command]
pub async fn rd_save_api_key(
    pool: State<'_, DbPool>,
    api_key: String,
) -> Result<RdUserInfo, String> {
    let user = rd::get_user(&api_key).await?;

    let token = RealDebridToken {
        id: 0,
        client_id: "api_key".to_string(),
        client_secret: String::new(),
        access_token: api_key,
        refresh_token: String::new(),
        expires_at: user.expiration.clone(),
        created_at: String::new(),
    };

    storage::save_rd_token(&pool, &token).await?;

    Ok(RdUserInfo {
        username: user.username,
        email: user.email,
        premium: user.premium > 0,
        expiration: user.expiration,
    })
}

#[tauri::command]
pub async fn rd_unrestrict_link(
    pool: State<'_, DbPool>,
    link: String,
) -> Result<rd::UnrestrictedLink, String> {
    let token = storage::load_rd_token(&pool)
        .await?
        .ok_or("Kein RealDebrid-Token gefunden. Bitte zuerst authentifizieren.")?;

    rd::unrestrict_link(&token.access_token, &link).await
}

#[tauri::command]
pub async fn rd_get_status(pool: State<'_, DbPool>) -> Result<Option<RdStatusInfo>, String> {
    let token = match storage::load_rd_token(&pool).await? {
        Some(t) => t,
        None => return Ok(None),
    };

    match rd::get_user(&token.access_token).await {
        Ok(user) => Ok(Some(RdStatusInfo {
            status: "authenticated".to_string(),
            user: Some(RdUserInfo {
                username: user.username,
                email: user.email,
                premium: user.premium > 0,
                expiration: user.expiration,
            }),
        })),
        Err(_) => {
            let expires = chrono::DateTime::parse_from_rfc3339(&token.expires_at)
                .map_err(|e| e.to_string())?;
            if expires > chrono::Utc::now() {
                Ok(Some(RdStatusInfo {
                    status: "authenticated".to_string(),
                    user: None,
                }))
            } else {
                Ok(Some(RdStatusInfo {
                    status: "expired".to_string(),
                    user: None,
                }))
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RdUserInfo {
    pub username: String,
    pub email: String,
    pub premium: bool,
    pub expiration: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RdStatusInfo {
    pub status: String,
    pub user: Option<RdUserInfo>,
}
