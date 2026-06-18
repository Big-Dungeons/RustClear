use crate::core::network::packets::packet_deserializable::PacketDeserializable;
use crate::core::network::packets::IdentifiedPacket;
use crate::core::network::protocol::sized_string::SizedString;
use crate::core::network::Bytes;
use macros::{identified_packet, PacketDeserializable};

#[identified_packet(id=0x00)]
#[derive(PacketDeserializable)]
pub struct LoginStart {
    pub username: SizedString<16>
}