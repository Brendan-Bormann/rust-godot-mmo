use bitcode::{decode, encode};
use std::io::{self, Read, Write};

use crate::{
    game::vector::Vector2,
    network::packet::{Packet, PacketPayload, PacketType},
};

pub struct PacketTCP<S> {
    pub stream: S,
}

impl<S: Read + Write> PacketTCP<S> {
    pub fn new(stream: S) -> Self {
        Self { stream }
    }

    fn read(&mut self) -> io::Result<Vec<u8>> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        self.stream.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.stream.write_all(&(data.len() as u32).to_be_bytes())?;
        self.stream.write_all(data)?;
        self.stream.flush()
    }

    pub fn recv_packet(&mut self) -> io::Result<Packet> {
        let buf = self.read()?;
        decode::<Packet>(&buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    pub fn send_packet(&mut self, packet: &Packet) -> io::Result<()> {
        let data = encode(packet);
        self.write(&data)
    }

    // --- high-level senders ---

    pub fn send_heartbeat(&mut self, id: String) -> io::Result<()> {
        self.send_packet(&Packet::new(id, PacketType::Heartbeat, PacketPayload::None))
    }

    pub fn send_auth_token(&mut self, id: String, auth_token: String) -> io::Result<()> {
        self.send_packet(&Packet::new(
            id,
            PacketType::AuthToken,
            PacketPayload::Token(auth_token),
        ))
    }

    pub fn send_input_failure(&mut self, id: String) -> io::Result<()> {
        self.send_packet(&Packet::new(
            id,
            PacketType::InputFailure,
            PacketPayload::None,
        ))
    }

    pub fn send_input_success(&mut self, id: String) -> io::Result<()> {
        self.send_packet(&Packet::new(
            id,
            PacketType::InputSuccess,
            PacketPayload::None,
        ))
    }

    pub fn send_input_direction(&mut self, id: String, dir: Vector2) -> io::Result<()> {
        self.send_packet(&Packet::new(
            id,
            PacketType::InputDirection,
            PacketPayload::InputDirection(dir),
        ))
    }

    pub fn send_input_rotation(&mut self, id: String, rot: f32) -> io::Result<()> {
        self.send_packet(&Packet::new(
            id,
            PacketType::InputRotation,
            PacketPayload::InputRotation(rot),
        ))
    }
}
