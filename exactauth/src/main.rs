use actix_cors::Cors;
use actix_web::{App, HttpServer, ResponseError, web};
use actix_web::body::MessageBody;
use mrauth::MrAuthClient;
use noiseless_tracing_actix_web::NoiselessRootSpanBuilder;
use tracing::{debug, info};
use tracing_actix_web::RootSpanBuilder;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use dal::Mysql;
use crate::config::Config;
use crate::routable::Routable;

mod config;
mod routes;
mod exact_api;
mod error;
mod tasks;
mod routable;

pub type MysqlData = web::Data<Mysql>;
pub type ConfigData = web::Data<Config>;
pub type AuthData = web::Data<MrAuthClient>;

#[cfg(not(debug_assertions))]
const BIND_PORT: u16 = 8080;
#[cfg(debug_assertions)]
const BIND_PORT: u16 = 8081;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    setup_tracing();

    info!("Starting server");
    debug!("Reading config");
    let config: Config = envy::from_env().expect("Reading config");
    let mysql = Mysql::new(&config.mysql_user, &config.mysql_password, &config.mysql_host, &config.mysql_db).expect("Setting up DB");

    tasks::refresh_tokens::start_refresh_token_task(
        mysql.clone(),
        &config.exact_client_id,
        &config.exact_client_secret,
        &config.redirect_uri
    );

    let authclient = MrAuthClient::new(
        &format!("MrFriendly Exactauth v{}", env!("CARGO_PKG_VERSION")),
        config.mrauth_url.clone(),
    );

    HttpServer::new(move || App::new()
        .wrap(Cors::permissive())
        .wrap(tracing_actix_web::TracingLogger::<NoiselessRootSpanBuilder>::new())
        .app_data(web::Data::new(mysql.clone()))
        .app_data(web::Data::new(config.clone()))
        .app_data(web::Data::new(authclient.clone()))
        .configure(routes::Router::configure)
    ).bind(&format!("0.0.0.0:{BIND_PORT}"))?.run().await

}

fn setup_tracing() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO")
    }

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(layer().compact())
        .init();
}