use crate::core::network::packets::packet_serializable::PacketSerializable;
use crate::core::network::packets::IdentifiedPacket;
use bytes::BytesMut;
use macros::{identified_packet, PacketSerializable};

// LoginDisconnect = 0x00;
// EncryptionRequest = 0x01;
// EnableCompression = 0x03;

#[identified_packet(id=0x02)]
#[derive(Debug, PacketSerializable)]
pub struct LoginSuccess {
    pub uuid: String,
    pub name: String,
}