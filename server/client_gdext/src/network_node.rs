use rustls::pki_types::ServerName;
use rustls::{ClientConnection, RootCertStore, StreamOwned};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use godot::classes::{INode, Node};
use godot::prelude::*;
use shared::game::game_state::GameState;
use shared::game::vector;
use shared::network::packet::{Packet, PacketType};
use shared::network::packet_tcp::PacketTCP;

use crate::player_object::PlayerObject;

const SERVER_PORT: &str = "8080";
const SERVER_API_PORT: &str = "8081";

#[derive(GodotClass)]
#[class(base=Node)]
pub struct NetworkNode {
    packet_stream: Option<PacketTCP<TcpStream>>,
    game_state: GameState,
    rolling_packet_id: i32,
    auth_token: Option<String>,

    #[var]
    #[export]
    active: bool,

    #[var]
    pub di: Vector2,
    pub last_di: Vector2,

    #[var]
    pub packets_sent: i32,
    #[var]
    pub packets_recv: i32,
}

#[godot_api]
impl INode for NetworkNode {
    fn init(_base: Base<Node>) -> Self {
        godot_print!("CLIENT_RUST: Network Node Initialized.");

        Self {
            packet_stream: None,
            game_state: GameState::new(),
            rolling_packet_id: 1,
            auth_token: None,
            active: false,
            di: Vector2::ZERO,
            last_di: Vector2::ZERO,
            packets_sent: 0,
            packets_recv: 0,
        }
    }

    // fn physics_process(&mut self, _delta: f64) {
    //     self.poll_active();
    //     self.sync_di();
    // }

    fn process(&mut self, _delta: f64) {
        self.recv_packet();
        self.poll_active();
        self.sync_di();
    }
}

#[godot_api]
impl NetworkNode {
    #[func]
    fn login(&mut self, server_ip: String, email: String, password: String) -> bool {
        self.auth_token = None;

        let response = login(server_ip, email, password);

        if response.success {
            self.auth_token = Some(response.auth_token);
        }

        godot_print!("Login success: {} - {}", response.success, response.message);

        response.success
    }

    #[func]
    fn connect_to_server(&mut self, server_ip: String) -> bool {
        if self.auth_token.is_none() {
            return false;
        }

        // let root_store = RootCertStore {
        //     roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        // };
        // let config = rustls::ClientConfig::builder()
        //     .with_root_certificates(root_store)
        //     .with_no_client_auth();

        let server_addr = format!("{}:{}", server_ip, SERVER_PORT);

        // let server_name = "localhost".try_into().unwrap();
        // let conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();

        match TcpStream::connect(server_addr) {
            Ok(stream) => {
                stream
                    .set_read_timeout(Some(Duration::from_millis(10)))
                    .unwrap();

                // let tls = rustls::StreamOwned::new(conn, stream);
                // let mut stream = PacketTCP::new(tls);

                let mut stream = PacketTCP::new(stream);

                match &self.auth_token {
                    Some(token) => {
                        let r = stream.send_auth_token("0".into(), token.clone());

                        match r {
                            Ok(e) => {}
                            Err(e) => {
                                godot_print!("error: {}", e)
                            }
                        }

                        self.packet_stream = Some(stream);
                        godot_print!("Authenticated connection to server established.");
                        return true;
                    }
                    None => {
                        let _ = stream.stream.shutdown(std::net::Shutdown::Both);
                        godot_error!("Authenticated connection to server failed!");
                        return false;
                    }
                };
            }
            Err(_) => false,
        }
    }

    #[func]
    fn disconnect(&mut self) {
        if let Some(ref mut packet_stream) = self.packet_stream {
            packet_stream
                .stream
                .shutdown(std::net::Shutdown::Both)
                .unwrap();
        }
    }

    #[func]
    fn get_player_ids(&mut self) -> PackedStringArray {
        let mut ids: Vec<GString> = vec![];
        self.game_state
            .players
            .iter()
            .for_each(|(id, _)| ids.push(id.clone().into()));

        ids.into()
    }

    #[func]
    fn sync_player(&mut self, &mut godot_player: Gd<PlayerObject>) {
        let id: String = godot_player.get_name().to_string();

        if let Some(state_player) = self.game_state.players.get(&id) {
            let mut gdp = godot_player.bind_mut();
            gdp.network_set_player(&state_player);
        }
    }
}

impl NetworkNode {
    fn poll_active(&mut self) {
        if let Some(ref mut packet_stream) = self.packet_stream {
            self.active = packet_stream.stream.peer_addr().is_ok();
        } else {
            self.active = false;
        }
    }

    fn recv_packet(&mut self) {
        if let Some(ref mut packet_stream) = self.packet_stream {
            let packet = packet_stream.recv_packet();

            if packet.is_ok() {
                self.packets_recv += 1;
                self.process_packet(&packet.unwrap());
            }
        }
    }

    fn send_packet(&mut self, packet: &Packet) {
        if let Some(ref mut stream) = self.packet_stream {
            self.packets_sent += 1;
            let _ = stream.send_packet(&packet);
        }
    }

    fn process_packet(&mut self, packet: &Packet) {
        // godot_print!(
        //     "got a packet! t:{} s:{}",
        //     packet.packet_type,
        //     packet.packet_subtype
        // );

        // match packet.packet_type {
        //     0 => {}
        //     1 => {}
        //     2 => {}
        //     3 => match packet.packet_subtype {
        //         0 => {
        //             if let Some(payload) = packet.payload.clone() {
        //                 let new_state = bitcode::decode(&payload).unwrap();

        //                 if self.game_state != new_state {
        //                     self.game_state = new_state;
        //                 }
        //             }
        //         }
        //         _ => {}
        //     },
        //     _ => {}
        // };
    }

    fn sync_di(&mut self) {
        // if let Some(ref mut stream) = self.packet_stream {
        //     if self.di != self.last_di {
        //         self.rolling_packet_id += 1;
        //         let id = self.rolling_packet_id.to_string();
        //         let payload = bitcode::encode(&vector::Vector2::new(self.di.x, self.di.y));
        //         let packet = Packet::new(id, 2, 2, Some(payload));
        //         self.send_packet(&packet);
        //         self.last_di.x = self.di.x;
        //         self.last_di.y = self.di.y;
        //     }
        // }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub auth_token: String,
}

fn login(server_ip: String, email: String, password: String) -> LoginResponse {
    let client = reqwest::blocking::Client::new();

    let res = client
        .post(format!("http://{}:{}/login", server_ip, SERVER_API_PORT))
        .json(&LoginRequest { email, password })
        .send()
        .unwrap();

    if res.status().is_success() {
        return res.json().unwrap();
    } else {
        return LoginResponse {
            success: false,
            message: "Could not login.".into(),
            auth_token: "".into(),
        };
    }
}
