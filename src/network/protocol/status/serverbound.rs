use crate::network::client::Client;
use crate::network::packets::packet::{ProcessContext, ProcessPacket};
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::protocol::status::clientbound::{StatusPong, StatusResponse};
use crate::register_serverbound_packets;
use blocks::packet_deserializable;

register_serverbound_packets! {
    Status;
    StatusRequest = 0x00;
    StatusPing = 0x01;
}

packet_deserializable! {
    pub struct StatusRequest;
}

impl ProcessPacket for StatusRequest {
    async fn process<'a>(&self, client: &mut Client, context: ProcessContext<'a>) -> anyhow::Result<()> {
        let mut packet_buffer = PacketBuffer::new();
        packet_buffer.write_packet(&StatusResponse::default());
        context.network_thread_tx.send(packet_buffer.get_packet_message(client.client_id))?;
        Ok(())
    }
}

packet_deserializable! {
    pub struct StatusPing {
        pub client_time: i64,
    }
}

impl ProcessPacket for StatusPing {
    async fn process<'a>(&self, client: &mut Client, context: ProcessContext<'a>) -> anyhow::Result<()> {
        let mut packet_buffer = PacketBuffer::new();
        packet_buffer.write_packet(&StatusPong {
            client_time: self.client_time,
        });
        context.network_thread_tx.send(packet_buffer.get_packet_message(client.client_id))?;
        Ok(())
    }
}

