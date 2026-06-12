use crate::network::packets::packet_serializable::PacketSerializable;
use crate::network::packets::IdentifiedPacket;
use bytes::BytesMut;
use macros::{identified_packet, PacketSerializable};

#[identified_packet(id=0x00)]
#[derive(Debug, PacketSerializable)]
pub struct StatusResponse<'a> {
    pub status: &'a str
}

#[identified_packet(id=0x01)]
#[derive(Debug, PacketSerializable)]
pub struct StatusPong {
    pub client_time: i64
}