use crate::core::network::packets::packet_deserializable::PacketDeserializable;
use crate::core::network::packets::IdentifiedPacket;
use crate::core::network::Bytes;
use macros::{identified_packet, PacketDeserializable};

// 0x00 StatusRequest

#[identified_packet(id=0x01)]
#[derive(PacketDeserializable)]
pub struct StatusPing {
    pub client_time: i64
}