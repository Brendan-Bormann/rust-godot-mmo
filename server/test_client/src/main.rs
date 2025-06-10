use std::{
    io::ErrorKind,
    net::{TcpStream, UdpSocket},
    sync::Arc,
    thread,
    time::Duration,
};

use rand::Rng;

use shared::{
    game::{game_state::GameState, vector::Vector2},
    network::{packet::Packet, packet_tcp::PacketTCP, packet_udp::PacketUDP},
};

// scratch pad for scripting players
const PLAYER_COUNT: i16 = 100;

fn main() {
    println!("Running bots...");

    for i in 1..PLAYER_COUNT {
        thread::spawn(move || {
            println!(" - starting bot {}", i);
            start_bot(i);
        });
    }

    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}

fn start_bot(id: i16) {
    let mut packet_id = 0;
    let mut local_state = GameState::new();
    let username = format!("Bot{}", id);

    let tcp = TcpStream::connect("127.0.0.1:8080").unwrap();
    let s = tcp.try_clone().unwrap();
    let addr = tcp.local_addr().unwrap().clone();
    tcp.set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();

    let mut p_tcp = PacketTCP::new(tcp);

    let udp = UdpSocket::bind(addr).unwrap();
    udp.set_read_timeout(Some(Duration::from_millis(10)))
        .unwrap();
    let mut p_udp = PacketUDP::new(Arc::new(udp));

    packet_id += 1;
    let packet = Packet::new(packet_id.to_string(), 0, 0, None);
    p_tcp.send_packet(&packet).unwrap();

    packet_id += 1;
    let packet = Packet::new(
        packet_id.to_string(),
        2,
        1,
        Some(bitcode::encode::<String>(&username)),
    );
    p_tcp.send_packet(&packet).unwrap();

    let mut p_tcp2 = PacketTCP::new(s);

    thread::spawn(move || {
        let mut packet_id = 1;

        let packet2 = Packet::new(packet_id.to_string(), 2, 2, None);
        let mut rng = rand::rng();

        loop {
            let mut new_packet = packet2.clone();
            packet_id += 1;
            new_packet.id = format!("{}", packet_id);

            let x: f32 = rng.random_range(-1.0..1.0);
            let y: f32 = rng.random_range(-1.0..1.0);

            let random_di = Vector2::new(x, y);

            new_packet.payload = Some(bitcode::encode::<Vector2>(&random_di));
            p_tcp2.send_packet(&new_packet).unwrap();
            thread::sleep(Duration::from_millis(rng.random_range(200..2000)));
        }
    });

    loop {
        match p_tcp.recv_packet() {
            Ok(packet) => {
                // println!(
                //     "tcp:{}, id: {}, t: {}, s: {}",
                //     p_tcp.stream.peer_addr().unwrap(),
                //     if packet.id == "" { "_" } else { &packet.id },
                //     packet.packet_type,
                //     packet.packet_subtype
                // );

                if packet.packet_type == 3 {
                    let data = packet.payload.unwrap();
                    let new_state: GameState = bitcode::decode(&data).unwrap();
                    local_state = new_state;
                }
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {}
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {}
            _ => {}
        }

        match p_udp.recv_packet() {
            Ok((_packet, addr)) => {}
            _ => {}
        }
    }
}
