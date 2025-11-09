use crate::network::packets::packet::IdentifiedPacket;
use crate::network::packets::packet_serialize::PacketSerializable;
use crate::register_packets;
use bytes::BytesMut;
use fstr::FString;
use macros::packet_serializable;

register_packets! {
    // LoginDisconnect = 0x00;
    // EncryptionRequest = 0x01;
    LoginSuccess = 0x02;
    // EnableCompression = 0x03;
}

packet_serializable! {
    pub struct LoginSuccess {
        pub uuid: String,
        pub name: FString,
    }
}