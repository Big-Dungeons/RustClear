use crate::network::binary::var_int::VarInt;
use crate::network::client::Client;
use crate::network::connection_state::ConnectionState;
use crate::network::packets::packet::{ProcessContext, ProcessPacket};
use crate::register_serverbound_packets;
use crate::types::sized_string::SizedString;
use blocks::packet_deserializable;

register_serverbound_packets! {
    HandshakePacket;
    Handshake = 0x00;
}

packet_deserializable! {
    pub struct Handshake {
        pub protocol_version: VarInt,
        pub server_address: SizedString<255>,
        pub server_port: u16,
        pub next_state: VarInt,
    }
}

impl ProcessPacket for Handshake {
    async fn process<'a>(&self, client: &mut Client, _: ProcessContext<'a>) -> anyhow::Result<()> {
        client.connection_state = ConnectionState::from_id(self.next_state.0)?;
        Ok(())
    }
}