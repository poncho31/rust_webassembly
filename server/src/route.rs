use actix_web::web;
use crate::routes::ping_route;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ping").service(ping_route::get)
    );
}
