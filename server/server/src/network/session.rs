use std::io::ErrorKind;
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

use shared::game::game_state::GameState;
use shared::network::packet::Packet;
use shared::network::{packet_tcp::PacketTCP, packet_udp::PacketUDP};
use tokio::sync::oneshot::error::TryRecvError;
use tokio::sync::{oneshot, watch};
use tracing::{info, warn};

use crate::game::command::{Command, CommandHandlerClient};

pub struct Session {
    session_id: u32,
    tcp_stream: PacketTCP,
    udp_socket: PacketUDP,
    origin: String,
    player_id: Option<String>,
    state_watch_rx: watch::Receiver<GameState>,
    cmd_tx: mpsc::Sender<Command>,
    pending_cmds: Vec<(String, i16, oneshot::Receiver<Result<Option<Vec<u8>>, i16>>)>,
    command_handler_client: CommandHandlerClient,
}

impl Session {
    pub fn new(
        session_id: u32,
        tcp_stream: TcpStream,
        udp_socket: Arc<UdpSocket>,
        origin: SocketAddr,
        state_watch_rx: watch::Receiver<GameState>,
        cmd_tx: mpsc::Sender<Command>,
    ) -> Self {
        tcp_stream.set_nonblocking(true).unwrap();

        Session {
            session_id,
            tcp_stream: PacketTCP::new(tcp_stream),
            udp_socket: PacketUDP::new(udp_socket),
            origin: origin.to_string(),
            player_id: None,
            state_watch_rx,
            cmd_tx: cmd_tx.clone(),
            pending_cmds: vec![],
            command_handler_client: CommandHandlerClient::new(cmd_tx.clone()),
        }
    }
}

impl Session {
    pub fn start(&mut self) {
        info!("Connected: {} - Session: {}", self.origin, self.session_id);

        loop {
            match self.tcp_stream.recv_packet() {
                Ok(packet) => self.handle_client_packet(&packet),
                Err(ref e) if e.kind() == ErrorKind::TimedOut => {}
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
                _ => {}
            }

            self.sync_client_state();
            self.handle_pending_cmds();

            thread::sleep(Duration::from_millis(1));
        }

        info!("Disconnected: {}", self.origin);
    }

    fn handle_client_packet(&mut self, packet: &Packet) {
        match packet.packet_type {
            0 => match packet.packet_subtype {
                0 => {
                    self.tcp_stream
                        .send_packet(&Packet::new(packet.id.clone(), 0, 0, None))
                        .unwrap();
                }
                _ => {}
            },
            1 => {}
            2 => {
                match packet.packet_subtype {
                    1 => {
                        match self
                            .command_handler_client
                            .create_character(&packet.id, packet.payload.clone())
                        {
                            Ok(player_id) => {
                                self.command_handler_client.set_player_id(&player_id);
                                self.player_id = Some(player_id);
                            }
                            Err(_) => {
                                warn!("Failed to create a character!");
                            }
                        };
                    }
                    2 => {
                        match self
                            .command_handler_client
                            .set_character_bearing(&packet.id, packet.payload.clone())
                        {
                            Ok(_) => {
                                let response = Packet::respond_to(&packet, true, None);
                                let _ = self.tcp_stream.send_packet(&response);
                            }
                            Err(_) => {
                                let response = Packet::respond_to(&packet, false, None);
                                let _ = self.tcp_stream.send_packet(&response);
                            }
                        };
                    }
                    _ => {}
                };
            }
            3 => {}
            _ => {}
        }
    }

    fn sync_client_state(&mut self) {
        if self.state_watch_rx.has_changed().unwrap() {
            let mut new_state = self.state_watch_rx.borrow_and_update().clone();

            if let Some(player_id) = self.player_id.clone() {
                let self_position = match new_state.players.get_mut(&player_id) {
                    Some(player) => {
                        player.id = "0".into();
                        Some(player.position)
                    }
                    None => None,
                };

                if let Some(self_pos) = self_position {
                    use std::collections::HashMap;

                    let mut players_with_distance: Vec<_> = new_state
                        .players
                        .iter()
                        .map(|(id, player)| {
                            let distance = player.position.distance(&self_pos);
                            (id.clone(), player.clone(), distance)
                        })
                        .collect();

                    players_with_distance
                        .sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

                    let trimmed_players: HashMap<_, _> = players_with_distance
                        .into_iter()
                        .take(1000)
                        .map(|(id, player, _)| (id, player))
                        .collect();

                    new_state.players = trimmed_players;
                }

                let packet = Packet::new("".into(), 3, 0, Some(bitcode::encode(&new_state)));
                let _ = self.tcp_stream.send_packet(&packet);
            }
        }
    }

    fn handle_pending_cmds(&mut self) {
        self.pending_cmds.retain_mut(|(id, cmd_type, rx)| {
            match rx.try_recv() {
                Ok(res) => {
                    match res {
                        Ok(_) => {
                            self.tcp_stream
                                .send_packet(&Packet::new(id.clone(), 2, 0, None))
                                .unwrap();
                        }
                        Err(e) => {
                            self.tcp_stream
                                .send_packet(&Packet::new(
                                    id.clone(),
                                    2,
                                    -1,
                                    Some(bitcode::encode(&e)),
                                ))
                                .unwrap();
                        }
                    }

                    return false;
                }
                Err(e) => match e {
                    TryRecvError::Empty => {
                        return true;
                    }
                    TryRecvError::Closed => {
                        return false;
                    }
                },
            };
        });
    }
}
