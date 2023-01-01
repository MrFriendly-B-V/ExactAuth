use std::str::FromStr;
use actix_web::cookie::time;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::instrument;
use crate::exact_api::get_exact_url;

pub const TOKEN_PATH: &str = "/api/oauth2/token";

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Invalid grant: Either the token/code is invalid or has expired")]
    InvalidGrant,
    #[error("An OAuth2 error occurred: {0}")]
    Other(String)
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum OAuth2GrantType {
    AuthorizationCode,
    RefreshToken,
}

pub struct TokenPair {
    pub access: String,
    pub refresh: String,
    pub access_expiry: i64,
    pub refresh_expiry: i64
}

pub const REFRESH_VALID_FOR_SEC: i64 = 3600 * 24 * 30; //30 days

#[instrument(skip_all)]
pub async fn exchange_code_for_token(client_id: &str, client_secret: &str, redirect_uri: &str, code: &str) -> Result<TokenPair, TokenError> {
    token_exchange(
        client_id,
        client_secret,
        redirect_uri,
        Some(code),
        None,
        OAuth2GrantType::AuthorizationCode,
    ).await
}

#[instrument(skip_all)]
pub async fn refresh_tokens(client_id: &str, client_secret: &str, redirect_uri: &str, refresh_token: &str) -> Result<TokenPair, TokenError> {
    token_exchange(
        client_id,
        client_secret,
        redirect_uri,
        None,
        Some(refresh_token),
        OAuth2GrantType::RefreshToken,
    ).await
}

#[derive(Serialize)]
struct RequestForm<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<&'a str>,
    grant_type: OAuth2GrantType,
    client_id: &'a str,
    client_secret: &'a str,
    redirect_uri: &'a str,
}

#[derive(Deserialize)]
struct ResponseJson {
    access_token: String,
    expires_in: String, // Should be i64, but there's a bug in Exact's implementation
    refresh_token: String,
}

#[derive(Deserialize)]
struct ErrorResponseJson {
    error: String,
}

#[instrument(skip_all)]
async fn token_exchange(client_id: &str, client_secret: &str, redirect_uri: &str, code: Option<&str>, refresh_token: Option<&str>, grant_type: OAuth2GrantType) -> Result<TokenPair, TokenError> {
    let response = Client::new()
        .post(get_exact_url(TOKEN_PATH))
        .header("User-Agent", &format!("MrFriendly {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
        .form(&RequestForm {
            redirect_uri,
            grant_type,
            client_id,
            client_secret,
            refresh_token,
            code
        })
        .send()
        .await?;

    if !response.status().is_success() {
        let error: ErrorResponseJson = response.json().await?;
        return Err(if error.error.eq("invalid_grant") {
            TokenError::InvalidGrant
        } else {
            TokenError::Other(error.error)
        })
    }

    let response: ResponseJson = response.json().await?;

    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    let expires_in = i64::from_str(&response.expires_in)
        .map_err(|_| TokenError::Other(format!("Invalid value for 'expires_in', failed to parse '{}' to i64", response.expires_in)))?;
    Ok(TokenPair {
        access: response.access_token,
        refresh: response.refresh_token,
        access_expiry: now + expires_in,
        refresh_expiry: now + REFRESH_VALID_FOR_SEC,
    })
}