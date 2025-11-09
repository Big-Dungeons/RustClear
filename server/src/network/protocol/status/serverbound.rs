use macros::packet_deserializable;

// 0x00
// unused since it doesn't do any reading,
// but im keeping it to remind myself that this packet exists
packet_deserializable! {
    pub struct StatusRequest;
}

// 0x01
packet_deserializable! {
    pub struct StatusPing {
        pub client_time: i64,
    }
}

