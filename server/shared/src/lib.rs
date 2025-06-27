pub mod game;
pub mod network;
pub mod util;

#[cfg(test)]
mod tests {
    use bitcode::{decode, encode};

    use crate::{
        game::vector::Vector2,
        network::packet::{Packet, PacketPayload, PacketType},
    };

    #[test]
    fn test_encoding() {
        let packet = Packet::new(
            "0".into(),
            PacketType::InputDirection,
            PacketPayload::InputDirection(Vector2::zero()),
        );

        let data = encode(&packet);
        let decoded = decode::<Packet>(&data).unwrap();
        assert_eq!(packet, decoded); // confirm round-trip works
    }
}
