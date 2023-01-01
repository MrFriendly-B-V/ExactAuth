
use std::time::Duration;
use actix_web::cookie::time;
use thiserror::Error;
use tracing::{trace, warn};
use dal::{Mysql, User};
use crate::exact_api::TokenError;

const JOB_INTERVAL_SEC: u64 = 15;
const JOB_FAIL_INTERVAL_SEC: u64 = 5;

pub fn start_refresh_token_task(mysql: Mysql, client_id: &str, client_secret: &str, redirect_uri: &str) {
    let client_id = client_id.to_string();
    let client_secret = client_secret.to_string();
    let redirect_uri = redirect_uri.to_string();

    tokio::spawn(async move {
        loop {
            match refresh_tokens(mysql.clone(), &client_id, &client_secret, &redirect_uri).await {
                Ok(_) => {
                    trace!("All tokens that needed refreshing refreshed. Checking again in {JOB_INTERVAL_SEC} seconds");
                    tokio::time::sleep(Duration::from_secs(JOB_INTERVAL_SEC)).await;
                },
                Err(e) => {
                    warn!("Failed to refresh tokens: {e}. Retrying in {JOB_FAIL_INTERVAL_SEC} seconds");
                    tokio::time::sleep(Duration::from_secs(JOB_FAIL_INTERVAL_SEC)).await;
                }
            }
        }
    });
}

#[derive(Debug, Error)]
pub enum RefreshError {
    #[error("DAL error: {0}")]
    Dal(#[from] dal::Error),
    #[error("Token error: {0}")]
    Token(#[from] TokenError)
}

async fn refresh_tokens(mysql: Mysql, client_id: &str, client_secret: &str, redirect_uri: &str) -> Result<(), RefreshError> {
    let users = User::list_all(mysql)?;
    for user in users {
        let access_token = match user.get_access_token()? {
            Some(x) => x,
            None => continue,
        };

        let refresh_token = match user.get_refresh_token()? {
            Some(x) => x,
            None => continue,
        };

        // Check if the token is within 29 seconds of expiring
        // Reason for 29:
        // Exact tokens are valid for 10 minutes, but may only be refreshed after
        // they expire within 30 seconds. The 1 second difference is to provide a buffer
        let now = time::OffsetDateTime::now_utc();
        let expiry = time::OffsetDateTime::from_unix_timestamp(access_token.expiry)
            .expect("Unix timestamp was out of range");

        trace!("Access token for user {} expires at {}", user.id, access_token.expiry);
        if now > expiry || (expiry - now).whole_seconds() < 29 {
            trace!("Access token for user {} has expired, or must be refreshed", user.id);

            // Refresh the token
            let refreshed_pair = crate::exact_api::refresh_tokens(
                client_id,
                client_secret,
                redirect_uri,
                &refresh_token.token
            ).await?;

            if refresh_token.token.ne(&refreshed_pair.refresh) {
                user.set_refresh_token(&refreshed_pair.refresh, refreshed_pair.refresh_expiry)?;
            }

            user.set_access_token(&refreshed_pair.access, refreshed_pair.access_expiry)?;

            trace!("Refreshed tokens for user {}", user.id);
        } else {
            trace!("Access token for user {} is not yet expired", user.id);
        }
    }

    Ok(())
}