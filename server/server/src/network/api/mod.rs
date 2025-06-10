pub mod auth;

use axum::{
    Json, Router,
    routing::{get, post},
};
use std::net::SocketAddr;
use tracing::info;

use crate::storage::mem_db::MemDB;

#[derive(Clone)]
pub struct AppState {
    pub mem_db: MemDB,
}

pub async fn start_api(address: String, mem_db: MemDB) {
    info!("API started");

    let state = AppState { mem_db };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/login", post(auth::login))
        .route("/create-account", post(auth::create_account))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn root_handler() -> &'static str {
    "test"
}
