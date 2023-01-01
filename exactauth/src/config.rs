use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub mysql_host: String,
    pub mysql_user: String,
    pub mysql_password: String,
    pub mysql_db: String,
    pub exact_client_id: String,
    pub exact_client_secret: String,
    pub redirect_uri: String,
    pub mrauth_url: String,
}