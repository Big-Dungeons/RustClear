use crate::core::network::packets::packet_deserializable::PacketDeserializable;
use crate::core::network::packets::IdentifiedPacket;
use crate::core::network::protocol::var_int::VarInt;
use bytes::Bytes;
use macros::{identified_packet, PacketDeserializable};

#[identified_packet(id=0x00)]
#[derive(Debug, PacketDeserializable)]
pub struct Handshake {
    pub _protocol_version: VarInt,
    pub _server_address: String,
    pub _server_port: u16,
    pub next_state: VarInt,
}