use actix_web::web;
use actix_web::web::ServiceConfig;
use crate::routable::Routable;

mod v1;
mod redirect;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.service(web::scope("/api")
            .configure(v1::Router::configure)
        );
    }
}