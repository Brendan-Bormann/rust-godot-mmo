mod game;
mod network;
mod storage;

use game::game_manager::GameManager;
use network::{api, client::ClientManager};
use tracing::info;

use std::{net::TcpListener, thread};
use tracing_subscriber;

use crate::storage::{Storage, mem_db::MemDB, sql_db::SQLDB};

const NETWORK_PORT: &str = "8080";
const API_PORT: &str = "8081";
const MEMDB_ADDR: &str = "redis://127.0.0.1/";
const PSQL_ADDR: &str = "postgres://admin:password@127.0.0.1:5432/game";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .with_timer(tracing_subscriber::fmt::time::time())
        .compact()
        .init();

    info!("--  Server Started  --");

    let client_tcp = TcpListener::bind(format!("0.0.0.0:{NETWORK_PORT}")).unwrap();

    let mem_db = MemDB::new(format!("{}", MEMDB_ADDR));
    let sql_db = SQLDB::new(format!("{}", PSQL_ADDR)).await;
    let storage = Storage::new(mem_db, sql_db);

    let (mut game_manager, state_watch_rx, cmd_tx) = GameManager::new();

    // let game_storage = storage.clone();
    let _game = thread::spawn(move || {
        let _ = game_manager.start();
    });

    let network_storage = storage.clone();
    let _network = thread::spawn(move || {
        let mut client_manager =
            ClientManager::new(client_tcp, state_watch_rx, cmd_tx, network_storage);
        let _ = client_manager.start();
    });

    let api_storage = storage.clone();
    let _api = tokio::spawn(async move {
        api::start_api(format!("0.0.0.0:{API_PORT}"), api_storage).await;
    });

    loop {}
}
