use std::sync::LazyLock;

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

// todo:
// make this be defined outside of the crate 

pub static STATUS_RESPONSE_JSON: LazyLock<String> = LazyLock::new(|| {
    // let encoded_image = &get_assets().icon_data;
    let version = env!("CARGO_PKG_VERSION"); // this is (probably) gonna grab the version of the server crate. Might be fine, but the build scripts arent and it will be desyned.

    format!(r#"{{
        "version": {{ "name": "1.8.9", "protocol": 47 }},
        "players": {{ "max": 1, "online": 0 }},
        "description": {{ "text": "RustClear", "color": "gold", "extra": [{{ "text": " version ", "color": "gray" }}, {{ "text": "{version}", "color": "green" }}] }},
        "favicon": "data:image/png;base64,<data>"
    }}"#)
});
