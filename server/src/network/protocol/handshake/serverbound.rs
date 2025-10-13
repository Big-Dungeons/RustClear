use crate::network::binary::var_int::VarInt;
use crate::types::sized_string::SizedString;
use blocks::packet_deserializable;

// 0x00
packet_deserializable! {
    #[derive(Debug)]
    pub struct Handshake {
        pub protocol_version: VarInt,
        pub server_address: SizedString<255>,
        pub server_port: u16,
        pub next_state: VarInt,
    }
}