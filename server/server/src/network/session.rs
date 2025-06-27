use std::io::ErrorKind;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use rustls::{ServerConnection, StreamOwned};
use shared::game::game_state::GameState;
use shared::network::packet::{Packet, PacketPayload, PacketType};
use shared::network::packet_tcp::PacketTCP;
use tokio::stream;
use tokio::sync::oneshot::error::TryRecvError;
use tokio::sync::{oneshot, watch};
use tracing::{info, warn};

use crate::game::command::{Command, CommandHandlerClient};
use crate::storage::Storage;

pub struct Session {
    packet_stream: PacketTCP<TcpStream>,
    origin: String,
    player_id: Option<String>,
    state_watch_rx: watch::Receiver<GameState>,
    pending_cmds: Vec<(String, i16, oneshot::Receiver<Result<Option<Vec<u8>>, i16>>)>,
    command_handler_client: CommandHandlerClient,
    auth_token: String,
    account_id: String,
    db: Storage,
}

impl Session {
    pub fn new(
        tcp_stream: TcpStream,
        origin: SocketAddr,
        state_watch_rx: watch::Receiver<GameState>,
        cmd_tx: mpsc::Sender<Command>,
        auth_token: String,
        account_id: String,
        db: Storage,
    ) -> Self {
        Session {
            packet_stream: PacketTCP::new(tcp_stream),
            origin: origin.to_string(),
            player_id: None,
            state_watch_rx,
            pending_cmds: vec![],
            command_handler_client: CommandHandlerClient::new(cmd_tx.clone()),
            auth_token,
            account_id,
            db,
        }
    }
}

impl Session {
    pub fn start(&mut self) {
        info!("Session started: {}", self.origin);

        loop {
            match self.packet_stream.recv_packet() {
                Ok(packet) => {
                    let success = self.authenticate(&packet);

                    if success {
                        info!("Session authenticated: {}", &self.origin);
                        let _ = self
                            .db
                            .mem
                            .consume_session(&self.origin.to_string().split(":").next().unwrap());
                        break;
                    } else {
                        self.stop_stream();
                        return;
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::TimedOut => {}
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                    self.stop_stream();
                    return;
                }
                _ => {}
            }

            thread::sleep(Duration::from_millis(1000));
        }

        // TODO: Control flow ie:
        // check if user has character id
        // - if not, allow createing or setting a character
        // - if so, prevent creating or swapping character
        // next if the character id is set, we allow game inputs

        loop {
            match self.packet_stream.recv_packet() {
                Ok(packet) => self.handle_client_packet(&packet),
                Err(ref e) if e.kind() == ErrorKind::TimedOut => {}
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                    self.stop_stream();
                    return;
                }
                _ => {}
            }

            self.sync_client_state();
            self.handle_pending_cmds();

            thread::sleep(Duration::from_millis(1));
        }
    }

    fn stop_stream(&mut self) {
        let peer = self.packet_stream.stream.peer_addr();

        if peer.is_ok() {
            self.packet_stream
                .stream
                .shutdown(std::net::Shutdown::Both)
                .unwrap();
        }

        let _ = self
            .db
            .mem
            .close_session(&self.origin.to_string().split(":").next().unwrap());

        info!("Disconnected: {}", &self.origin);
    }

    // client gets one chance to send a valid auth token
    fn authenticate(&mut self, packet: &Packet) -> bool {
        match packet.packet_type {
            PacketType::AuthToken => match &packet.payload {
                PacketPayload::Token(token) => {
                    if token == &self.auth_token {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn handle_client_packet(&mut self, packet: &Packet) {
        // match packet.get_type() {
        //     PacketType::Heartbeat => {
        //         self.packet_stream
        //             .send_packet(&Packet::new(&packet.id, PacketType::Heartbeat, None))
        //             .unwrap();
        //     }
        //     PacketType::InputDirection => {
        //         match self
        //             .command_handler_client
        //             .set_character_bearing(&packet.id, packet.payload.clone())
        //         {
        //             Ok(_) => {
        //                 self.packet_stream
        //                     .send_packet(&Packet::new(&packet.id, PacketType::InputSuccess, None))
        //                     .unwrap();
        //             }
        //             Err(_) => {
        //                 self.packet_stream
        //                     .send_packet(&Packet::new(&packet.id, PacketType::InputFailure, None))
        //                     .unwrap();
        //             }
        //         };
        //     }
        //     PacketType::InputRotation => {
        //         match self
        //             .command_handler_client
        //             .set_character_bearing(&packet.id, packet.payload.clone())
        //         {
        //             Ok(_) => {
        //                 self.packet_stream
        //                     .send_packet(&Packet::new(&packet.id, PacketType::InputSuccess, None))
        //                     .unwrap();
        //             }
        //             Err(_) => {
        //                 self.packet_stream
        //                     .send_packet(&Packet::new(&packet.id, PacketType::InputFailure, None))
        //                     .unwrap();
        //             }
        //         };
        //     }
        //     _ => {}
        // }
    }

    fn sync_client_state(&mut self) {
        // if self.state_watch_rx.has_changed().unwrap() {
        //     let mut new_state = self.state_watch_rx.borrow_and_update().clone();

        //     if let Some(player_id) = self.player_id.clone() {
        //         let self_position = match new_state.players.get_mut(&player_id) {
        //             Some(player) => {
        //                 player.id = "0".into();
        //                 Some(player.position)
        //             }
        //             None => None,
        //         };

        //         if let Some(self_pos) = self_position {
        //             use std::collections::HashMap;

        //             let mut players_with_distance: Vec<_> = new_state
        //                 .players
        //                 .iter()
        //                 .map(|(id, player)| {
        //                     let distance = player.position.distance(&self_pos);
        //                     (id.clone(), player.clone(), distance)
        //                 })
        //                 .collect();

        //             players_with_distance
        //                 .sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        //             let trimmed_players: HashMap<_, _> = players_with_distance
        //                 .into_iter()
        //                 .take(100)
        //                 .map(|(id, player, _)| (id, player))
        //                 .collect();

        //             new_state.players = trimmed_players;
        //         }

        //         let packet = Packet::new(
        //             "".into(),
        //             PacketType::StateFull,
        //             Some(bitcode::encode(&new_state)),
        //         );

        //         let _ = self.packet_stream.send_packet(&packet);
        //     }
        // }
    }

    fn handle_pending_cmds(&mut self) {
        // self.pending_cmds.retain_mut(|(id, cmd_type, rx)| {
        //     match rx.try_recv() {
        //         Ok(res) => {
        //             match res {
        //                 Ok(_) => {
        //                     self.packet_stream
        //                         .send_packet(&Packet::new(id, PacketType::InputSuccess, None))
        //                         .unwrap();
        //                 }
        //                 Err(e) => {
        //                     self.packet_stream
        //                         .send_packet(&Packet::new(
        //                             id,
        //                             PacketType::InputFailure,
        //                             Some(bitcode::encode(&e)),
        //                         ))
        //                         .unwrap();
        //                 }
        //             }

        //             return false;
        //         }
        //         Err(e) => match e {
        //             TryRecvError::Empty => {
        //                 return true;
        //             }
        //             TryRecvError::Closed => {
        //                 return false;
        //             }
        //         },
        //     };
        // });
    }
}
