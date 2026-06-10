use crate::network::packets::packet_deserializable::PacketDeserializable;
use crate::network::packets::IdentifiedPacket;
use crate::network::protocol::sized_string::SizedString;
use crate::network::Bytes;
use macros::{identified_packet, PacketDeserializable};

#[identified_packet(id=0x00)]
#[derive(PacketDeserializable)]
pub struct LoginStart {
    pub username: SizedString<16>
}