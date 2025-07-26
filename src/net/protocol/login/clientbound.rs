use crate::net::packets::packet::IdentifiedPacket;
use crate::net::packets::packet_serialize::PacketSerializable;
use crate::register_packets;
use blocks::packet_serializable;

register_packets! {
    // LoginDisconnect = 0x00;
    // EncryptionRequest = 0x01;
    LoginSuccess = 0x02;
    // EnableCompression = 0x03;
}

packet_serializable! {
    pub struct LoginSuccess {
        pub uuid: String,
        pub name: String,
    }
}