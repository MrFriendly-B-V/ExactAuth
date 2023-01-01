use actix_web::web;
use actix_web::web::ServiceConfig;
use crate::routable::Routable;

mod access_token;
mod logged_in;
mod login;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.service(web::scope("/v1")
            .route("/login", web::get().to(login::login))
            .route("/logged-in", web::get().to(logged_in::logged_in))
            .route("/access-token", web::get().to(access_token::access_token))
        );
    }
}