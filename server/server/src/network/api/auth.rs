use std::net::SocketAddr;

use crate::network::api::AppState;
use axum::{
    Json,
    extract::{ConnectInfo, State},
};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct LoginInfo {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateAccountResponse {
    pub success: bool,
    pub message: String,
}

pub async fn create_account(
    State(AppState { db }): State<AppState>,
    ConnectInfo(_addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginInfo>,
) -> Json<CreateAccountResponse> {
    let email = &payload.email;
    let password = &payload.password;

    match db.sql.create_account(email, password).await {
        Ok(true) => Json(CreateAccountResponse {
            success: true,
            message: "Account created.".into(),
        }),
        Ok(false) => Json(CreateAccountResponse {
            success: false,
            message: "Email is already in use.".into(),
        }),
        Err(_) => Json(CreateAccountResponse {
            success: false,
            message: "Failed to create account.".into(),
        }),
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub auth_token: String,
}

pub async fn login(
    State(AppState { mut db }): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginInfo>,
) -> Json<LoginResponse> {
    let email = &payload.email;
    let password = &payload.password;

    match db.sql.verify_password(email, password).await {
        Ok((id, pw_match)) => {
            if !pw_match {
                return Json(LoginResponse {
                    success: false,
                    message: "Failed to log in.".into(),
                    auth_token: "".into(),
                });
            } else {
                let auth_token = Uuid::new_v4().to_string();

                db.mem
                    .open_session(
                        &addr
                            .to_string()
                            .split(':')
                            .next()
                            .unwrap_or(&addr.to_string()),
                        &auth_token,
                        id,
                    )
                    .unwrap();
                Json(LoginResponse {
                    success: true,
                    message: "Successfully logged in.".into(),
                    auth_token,
                })
            }
        }
        Err(_) => Json(LoginResponse {
            success: false,
            message: "Failed to log in.".into(),
            auth_token: "".into(),
        }),
    }
}
