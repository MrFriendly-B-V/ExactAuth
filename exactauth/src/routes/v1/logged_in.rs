use actix_web::web;
use serde::Deserialize;
use tracing::instrument;
use dal::User;
use crate::{ConfigData, MysqlData};
use crate::error::{Error, WebResult};
use crate::exact_api::exchange_code_for_token;
use crate::routes::redirect::Redirect;

#[derive(Deserialize)]
pub struct Query {
    code: String,
    state: String,
}

#[instrument(skip(mysql, config, query))]
pub async fn logged_in(mysql: MysqlData, config: ConfigData, query: web::Query<Query>) -> WebResult<Redirect> {
    let auth_start = User::get_by_authorization_start_id(mysql.as_ref().clone(), &query.state)?
        .ok_or(Error::Forbidden("Unknown state".into()))?;

    let token_pair = exchange_code_for_token(
        &config.exact_client_id,
        &config.exact_client_secret,
        &config.redirect_uri,
        &query.code,
    ).await?;

    let user = auth_start.user;
    user.set_access_token(&token_pair.access, token_pair.access_expiry)?;
    user.set_refresh_token(&token_pair.refresh, token_pair.refresh_expiry)?;

    Ok(Redirect::new(auth_start.caller))
}