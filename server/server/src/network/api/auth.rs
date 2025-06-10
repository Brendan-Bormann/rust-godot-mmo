use std::net::SocketAddr;

use axum::{
    Json,
    extract::{ConnectInfo, State},
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::network::api::AppState;

#[derive(Serialize, Deserialize)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
}

pub async fn create_account(
    State(AppState { mut mem_db }): State<AppState>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginInfo>,
) -> Json<Response> {
    let username = &payload.username;
    let password = &payload.password;

    match mem_db.create_account(username, password) {
        Ok(_) => Json(Response { success: true }),
        Err(_) => Json(Response { success: false }),
    }
}

pub async fn login(
    State(AppState { mut mem_db }): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginInfo>,
) -> Json<Response> {
    let username = &payload.username;
    let password = &payload.password;

    match mem_db.login(username, password) {
        Ok(_) => {
            info!("{} logged in!", username);
            mem_db.create_session(&addr.to_string(), username).unwrap();
            Json(Response { success: true })
        }
        Err(_) => {
            info!("{} failed to log in", username);
            Json(Response { success: false })
        }
    }
}
