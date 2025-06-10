use std::net::TcpStream;
use std::time::Duration;

use godot::classes::{INode, Node};
use godot::prelude::*;
use shared::game::game_state::GameState;
use shared::game::vector;
use shared::network::packet::Packet;
use shared::network::packet_tcp::PacketTCP;

use crate::player_object::PlayerObject;

const SERVER_PORT: &str = "8080";

#[derive(GodotClass)]
#[class(base=Node)]
pub struct NetworkNode {
    tcp_stream: Option<PacketTCP>,
    game_state: GameState,
    rolling_packet_id: i32,

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
            tcp_stream: None,
            game_state: GameState::new(),
            rolling_packet_id: 1,
            active: false,
            di: Vector2::ZERO,
            last_di: Vector2::ZERO,
            packets_sent: 0,
            packets_recv: 0,
        }
    }

    fn physics_process(&mut self, _delta: f64) {
        self.poll_active();
        self.sync_di();
    }

    fn process(&mut self, _delta: f64) {
        self.recv_packet();
    }
}

#[godot_api]
impl NetworkNode {
    #[func]
    fn connect_to_server(&mut self, server_ip: String, username: String) -> bool {
        match TcpStream::connect(format!("{}:{}", server_ip, SERVER_PORT)) {
            Ok(stream) => {
                stream
                    .set_read_timeout(Some(Duration::from_millis(10)))
                    .unwrap();
                self.tcp_stream = Some(PacketTCP::new(stream));

                self.send_packet(&Packet::new(
                    "1".into(),
                    2,
                    1,
                    Some(bitcode::encode::<String>(&username)),
                ));

                true
            }
            Err(_) => false,
        }
    }

    #[func]
    fn disconnect(&mut self) {
        if let Some(ref mut tcp_stream) = self.tcp_stream {
            tcp_stream
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
        if let Some(ref mut tcp_stream) = self.tcp_stream {
            self.active = tcp_stream.stream.peer_addr().is_ok();
        } else {
            self.active = false;
        }
    }

    fn recv_packet(&mut self) {
        if let Some(ref mut tcp_stream) = self.tcp_stream {
            let packet = tcp_stream.recv_packet();

            if packet.is_ok() {
                self.packets_recv += 1;
                self.process_packet(&packet.unwrap());
            }
        }
    }

    fn send_packet(&mut self, packet: &Packet) {
        if let Some(ref mut stream) = self.tcp_stream {
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

        match packet.packet_type {
            0 => {}
            1 => {}
            2 => {}
            3 => match packet.packet_subtype {
                0 => {
                    if let Some(payload) = packet.payload.clone() {
                        let new_state = bitcode::decode(&payload).unwrap();

                        if self.game_state != new_state {
                            self.game_state = new_state;
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }

    fn sync_di(&mut self) {
        if let Some(ref mut stream) = self.tcp_stream {
            if self.di != self.last_di {
                self.rolling_packet_id += 1;
                let id = self.rolling_packet_id.to_string();
                let payload = bitcode::encode(&vector::Vector2::new(self.di.x, self.di.y));
                let packet = Packet::new(id, 2, 2, Some(payload));
                self.send_packet(&packet);
                self.last_di.x = self.di.x;
                self.last_di.y = self.di.y;
            }
        }
    }
}
