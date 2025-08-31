use crate::net::internal_packets::NetworkThreadMessage;
use crate::net::packets::packet::IdentifiedPacket;
use crate::net::packets::packet_serialize::PacketSerializable;
use crate::net::var_int::{write_var_int, VarInt};
use crate::server::player::player::ClientId;
use bytes::BytesMut;

#[derive(Debug)]
pub struct PacketBuffer {
    pub buffer: BytesMut,
}

impl PacketBuffer {
    
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new()
        }
    }

    pub fn write_packet<P : IdentifiedPacket + PacketSerializable>(&mut self, packet: &P) {
        let id = VarInt(P::PACKET_ID);
        write_var_int(&mut self.buffer, (id.write_size() + packet.write_size()) as i32);
        id.write(&mut self.buffer);
        packet.write(&mut self.buffer);
        
    }

    pub fn copy_from(&mut self, buf: &PacketBuffer) {
        self.buffer.extend_from_slice(&buf.buffer)
    }

    /// gets a message for network thread to send the packets inside the buffer to the client.
    pub fn get_packet_message(&mut self, client_id: &ClientId) -> NetworkThreadMessage {
        let msg = NetworkThreadMessage::SendPackets {
            client_id: *client_id,
            buffer: self.buffer.clone().freeze(),
        };
        self.buffer.clear();
        msg
    }
}