use actix_web;
use server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    server::run_server().await
}
