use actix_web::web;
use crate::routes::ping_route;

// This module defines the route configuration for the Actix web server. TODO 
//
#[allow(dead_code)]
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ping").service(ping_route::get)
    );
}
