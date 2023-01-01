use std::borrow::Cow;
use actix_cors::Cors;
use actix_web::{App, Error, HttpServer, ResponseError, web};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::{Method, StatusCode, Version};
use mrauth::MrAuthClient;
use tracing::{debug, info, Span};
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
        .wrap(tracing_actix_web::TracingLogger::<CleanRootSpanBuilder>::new())
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

#[derive(Clone, Copy)]
pub struct CleanRootSpanBuilder;

impl RootSpanBuilder for CleanRootSpanBuilder {
    #[allow(unused)] // They're not unused, CLion just doesnt know that
    fn on_request_start(request: &ServiceRequest) -> Span {
        let http_route: Cow<'static, str> = request
            .match_pattern()
            .map(Into::into)
            .unwrap_or_else(|| "default".into());
        let http_method = http_method_str(&request.method());
        let http_flavor = http_flavor(request.version());

        let span = ::tracing::span!(
                    ::tracing::Level::INFO,
                    "HTTP request",
                    http.method = %http_method,
                    http.route = %http_route,
                    http.flavor = %http_flavor,
                    http.status_code = ::tracing::field::Empty,
                    exception.message = ::tracing::field::Empty,
                    exception.details = ::tracing::field::Empty,
                );
        span
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        match &outcome {
            Ok(response) => {
                if let Some(error) = response.response().error() {
                    // use the status code already constructed for the outgoing HTTP response
                    handle_error(span, response.status(), error.as_response_error());
                } else {
                    let code: i32 = response.response().status().as_u16().into();
                    span.record("http.status_code", code);
                    span.record("otel.status_code", "OK");
                }
            }
            Err(error) => {
                let response_error = error.as_response_error();
                handle_error(span, response_error.status_code(), response_error);
            }
        };
    }
}

fn handle_error(span: Span, status_code: StatusCode, response_error: &dyn ResponseError) {
    // pre-formatting errors is a workaround for https://github.com/tokio-rs/tracing/issues/1565
    let display = format!("{response_error}");
    let debug = format!("{response_error:?}");
    span.record("exception.message", &tracing::field::display(display));
    span.record("exception.details", &tracing::field::display(debug));
    let code: i32 = status_code.as_u16().into();

    span.record("http.status_code", code);
}

fn http_flavor(version: Version) -> Cow<'static, str> {
    match version {
        Version::HTTP_09 => "0.9".into(),
        Version::HTTP_10 => "1.0".into(),
        Version::HTTP_11 => "1.1".into(),
        Version::HTTP_2 => "2.0".into(),
        Version::HTTP_3 => "3.0".into(),
        other => format!("{other:?}").into(),
    }
}

fn http_method_str(method: &Method) -> Cow<'static, str> {
    match method {
        &Method::OPTIONS => "OPTIONS".into(),
        &Method::GET => "GET".into(),
        &Method::POST => "POST".into(),
        &Method::PUT => "PUT".into(),
        &Method::DELETE => "DELETE".into(),
        &Method::HEAD => "HEAD".into(),
        &Method::TRACE => "TRACE".into(),
        &Method::CONNECT => "CONNECT".into(),
        &Method::PATCH => "PATCH".into(),
        other => other.to_string().into(),
    }
}