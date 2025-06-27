use shared::game::game_state::GameState;
use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
    sync::{Arc, mpsc},
    thread,
};
use tokio::sync::watch;
use tracing::{info, warn};

use crate::{game::command::Command, storage::Storage};

use super::session::Session;

// use rustls::{
//     ServerConnection,
//     pki_types::{CertificateDer, PrivateKeyDer},
// };
// use rustls::{StreamOwned, pki_types::pem::PemObject};

pub struct ClientManager {
    tcp_listener: TcpListener,
    state_watch_rx: watch::Receiver<GameState>,
    cmd_tx: mpsc::Sender<Command>,
    db: Storage,
}

impl ClientManager {
    pub fn new(
        tcp_listener: TcpListener,
        state_watch_rx: watch::Receiver<GameState>,
        cmd_tx: mpsc::Sender<Command>,
        db: Storage,
    ) -> Self {
        ClientManager {
            tcp_listener,
            state_watch_rx,
            cmd_tx,
            db,
        }
    }
}

impl ClientManager {
    pub fn start(&mut self) {
        info!("- Network started");

        // let certs = load_certs(Path::new("certs/cert.pem"));
        // let privkey = load_private_key(Path::new("certs/key.pem"));

        // let config = rustls::ServerConfig::builder()
        //     .with_no_client_auth()
        //     .with_single_cert(certs, privkey)
        //     .expect("Failed to config TLS");

        loop {
            match self.tcp_listener.accept() {
                Ok((stream, origin)) => {
                    // let conn = rustls::ServerConnection::new(Arc::new(config.clone())).unwrap();
                    // let stream = rustls::StreamOwned::new(conn, stream);

                    info!("New stream started - peer: {}", origin);

                    let session = self
                        .db
                        .mem
                        .find_session(&origin.to_string().split(":").next().unwrap())
                        .unwrap_or(None);

                    if let Some(session) = session {
                        info!("Found matching session, starting session...");
                        self.accept_connection(
                            stream,
                            origin,
                            session.auth_token,
                            session.account_id,
                            self.db.clone(),
                        );
                    } else {
                        stream.shutdown(std::net::Shutdown::Both).unwrap();
                        warn!("Failed to verify session - stream shutdown");
                    }
                }
                Err(e) => {
                    warn!("Error while accepting tcp stream: {e}")
                }
            }
        }
    }

    pub fn accept_connection(
        &mut self,
        tcp_stream: TcpStream,
        origin: SocketAddr,
        auth_token: String,
        account_id: String,
        db: Storage,
    ) {
        let mut session = Session::new(
            tcp_stream,
            origin,
            self.state_watch_rx.clone(),
            self.cmd_tx.clone(),
            auth_token,
            account_id,
            db,
        );
        thread::spawn(move || {
            let _ = session.start();
        });
    }
}

// fn load_certs(filename: &Path) -> Vec<CertificateDer<'static>> {
//     CertificateDer::pem_file_iter(filename)
//         .expect("cannot open certificate file")
//         .map(|result| result.unwrap())
//         .collect()
// }

// fn load_private_key(filename: &Path) -> PrivateKeyDer<'static> {
//     PrivateKeyDer::from_pem_file(filename).expect("cannot read private key file")
// }
