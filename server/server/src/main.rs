mod game;
mod network;
mod storage;

use game::game_manager::GameManager;
use network::{api, client::ClientManager};
use tracing::info;

use std::{
    net::{TcpListener, UdpSocket},
    sync::Arc,
    thread,
};
use tracing_subscriber;

use crate::storage::mem_db::MemDB;

const NETWORK_PORT: &str = "8080";
const API_PORT: &str = "8081";
const MEMDB_ADDR: &str = "redis://127.0.0.1/";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .with_timer(tracing_subscriber::fmt::time::time())
        .compact()
        .init();

    let mem_db_client = redis::Client::open(MEMDB_ADDR).unwrap();
    let pool = r2d2::Pool::builder().build(mem_db_client).unwrap();
    let mem_db = MemDB::new(pool);

    let (mut game_manager, state_watch_rx, cmd_tx) = GameManager::new();

    let _game = thread::spawn(move || {
        let _ = game_manager.start();
    });

    let client_udp = Arc::new(UdpSocket::bind(format!("0.0.0.0:{NETWORK_PORT}")).unwrap());
    let client_tcp = TcpListener::bind(format!("0.0.0.0:{NETWORK_PORT}")).unwrap();

    let _network = thread::spawn(move || {
        let mut client_manager =
            ClientManager::new(client_tcp, client_udp.clone(), state_watch_rx, cmd_tx);
        let _ = client_manager.start();
    });

    let api_mem_db = mem_db.clone();
    let _api = tokio::spawn(async move {
        api::start_api(format!("0.0.0.0:{API_PORT}"), api_mem_db).await;
    });

    loop {}
}
