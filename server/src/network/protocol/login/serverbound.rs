use crate::types::sized_string::SizedString;
use blocks::packet_deserializable;

// 0x00
packet_deserializable! {
    pub struct LoginStart {
        pub username: SizedString<16>
    }
}