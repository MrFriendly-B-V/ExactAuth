use actix_web::web;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use crate::{AuthData, ConfigData, MysqlData};
use crate::error::WebResult;
use crate::exact_api::get_exact_url;
use crate::routes::redirect::Redirect;

#[derive(Deserialize)]
pub struct Query {
    bearer: String,
    scopes: String,
    caller: String,
}

#[derive(Serialize)]
struct OAuth2Query<'a> {
    client_id: &'a str,
    redirect_uri: &'a str,
    state: &'a str,
    response_type: &'static str,
    force_login: u32,
    scopes: &'a str,
}

const EXACT_OAUTH2_LOGIN_URI: &str = "/api/oauth2/auth";
const SCOPE: &str = "nl.mrfriendly.exact";

#[instrument(skip(mysql, config, auth, query))]
pub async fn login(mysql: MysqlData, config: ConfigData, auth: AuthData, query: web::Query<Query>) -> WebResult<Redirect> {
    let auth_user = mrauth::User::get_user(&auth, &query.bearer, SCOPE).await?;
    let user = match dal::User::get_by_id(mysql.as_ref().clone(), &auth_user.id)? {
        Some(x) => x,
        None => dal::User::create(mysql.as_ref().clone(), &auth_user.id)?
    };

    let auth_start = user.start_authorization(&query.scopes, &query.caller)?;
    let query = serde_qs::to_string(&OAuth2Query {
        client_id: &config.exact_client_id,
        redirect_uri: &config.redirect_uri,
        state: &auth_start.id,
        response_type: "code",
        force_login: 1,
        scopes: &query.scopes,
    }).unwrap();

    let url = format!("{}?{query}", get_exact_url(EXACT_OAUTH2_LOGIN_URI));
    Ok(Redirect::new(url))
}