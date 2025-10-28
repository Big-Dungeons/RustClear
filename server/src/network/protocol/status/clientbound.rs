use crate::network::packets::packet::IdentifiedPacket;
use crate::network::packets::packet_serialize::PacketSerializable;
use crate::register_packets;
use blocks::packet_serializable;
use bytes::BytesMut;

register_packets! {
    StatusResponse<'_> = 0x00;
    StatusPong = 0x01;
}

packet_serializable! {
    pub struct StatusResponse<'a> {
        pub status: &'a str,
    }
}

packet_serializable! {
    pub struct StatusPong {
        pub client_time: i64,
    }
}
