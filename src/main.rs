pub mod config;
pub mod db;
pub mod models;
pub mod request_models;
pub mod routes;
pub mod sessions;

use anyhow::Result;
use std::net::SocketAddr;

use crate::{
    config::CONFIG,
    db::Database,
    routes::github::{github_oauth_callback, github_oauth_redirect},
};
use axum::{routing::get, Router};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub redis: Arc<redis::Client>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .pretty()
        .init();

    let config = CONFIG.clone();
    let redis = Arc::new(redis::Client::open(config.redis)?);
    let database = Database::new(config.database).await?;
    database.migrate().await?;

    let app_state = AppState { database, redis };

    let oauth_routes = Router::new()
        .route("/login", get(github_oauth_redirect))
        .route("/callback", get(github_oauth_callback));

    let app = Router::new()
        .nest("/oauth", oauth_routes)
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    println!("Starting on: {addr:?}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;
    println!("Started");
    Ok(())
}
