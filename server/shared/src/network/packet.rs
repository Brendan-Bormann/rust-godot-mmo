use bitcode::{Decode, Encode};

use crate::game::vector::Vector2;

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub struct Packet {
    pub id: String,
    pub packet_type: PacketType,
    pub payload: PacketPayload,
}

impl Packet {
    pub fn new(id: String, packet_type: PacketType, payload: PacketPayload) -> Packet {
        Packet {
            id,
            packet_type,
            payload,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum PacketType {
    Unknown,
    Heartbeat,
    AuthToken,
    InputFailure,
    InputSuccess,
    InputDirection,
    InputRotation,
    StateFull,
    StateFullDiff,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq)]
pub enum PacketPayload {
    None,
    Token(String),
    InputDirection(Vector2),
    InputRotation(f32),
    StateFull(Vec<u8>),
    StateFullDiff(Vec<u8>),
    Error(String),
}
