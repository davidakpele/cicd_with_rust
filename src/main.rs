use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tokio::net::TcpListener;

mod config;
mod connection;
mod controllers;
mod services;
mod repository;
mod models;
mod utils;
mod router;
mod responses;
mod middleware;
mod ws;
mod payloads;
mod enums;
mod bootstrap;

use config::settings::Settings;
use connection::connection::establish_connection;
use crate::{
    services::user_service, ws::{ws_channel::WsBroadcaster, ws_server::start_ws_server}
};

#[tokio::main]
async fn main() {
    let _settings = Settings::new();

    let pool = match establish_connection().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("❌ Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };


    let broadcaster = Arc::new(WsBroadcaster::new());
    let user_service = Arc::new(user_service::UserService::new(repository::user_repository::UserRepository { db: pool.clone() }));
  

    tokio::spawn({
        let broadcaster = broadcaster.clone();
        let user_service = user_service.clone();

        async move {
            start_ws_server(
                "0.0.0.0:9001",
                broadcaster,
                user_service,
            ).await;
        }
    });

    let app = router::web::create_routes(pool, broadcaster.clone())
    .layer(CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any));

    let listener = TcpListener::bind("0.0.0.0:8022").await.unwrap();
    println!("🚀 Server started on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}