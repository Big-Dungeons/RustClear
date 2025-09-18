use std::sync::LazyLock;

use crate::net::packets::packet::IdentifiedPacket;
use crate::net::packets::packet_serialize::PacketSerializable;
use crate::register_packets;
use base64::engine::general_purpose;
use base64::Engine;
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

impl Default for StatusResponse<'_> {
    fn default() -> Self {
        Self {
            status: &STATUS_RESPONSE_JSON
        }
    }
}

packet_serializable! {
    pub struct StatusPong {
        pub client_time: i64,
    }
}

// not real sure where to put this, but here should be fine for now.
const FAVICON_BYTES: &[u8] = include_bytes!("../../../assets/favicon.png");

pub static STATUS_RESPONSE_JSON: LazyLock<String> = LazyLock::new(|| {
    let encoded_image = general_purpose::STANDARD.encode(FAVICON_BYTES);
    let version = env!("CARGO_PKG_VERSION");

    format!(r#"{{
        "version": {{ "name": "1.8.9", "protocol": 47 }},
        "players": {{ "max": 1, "online": 0 }},
        "description": {{ "text": "RustClear", "color": "gold", "extra": [{{ "text": " version ", "color": "gray" }}, {{ "text": "{version}", "color": "green" }}] }},
        "favicon": "data:image/png;base64,{encoded_image}"
    }}"#)
});