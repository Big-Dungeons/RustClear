use crate::network::binary::var_int::{write_var_int, VarInt};
use crate::network::internal_packets::NetworkThreadMessage;
use crate::network::packets::packet::IdentifiedPacket;
use crate::network::packets::packet_serialize::PacketSerializable;
use crate::player::player::ClientId;
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
    pub fn get_packet_message(&mut self, client_id: ClientId) -> NetworkThreadMessage {
        let msg = NetworkThreadMessage::SendPackets {
            client_id,
            buffer: self.buffer.clone().freeze(),
        };
        self.buffer.clear();
        msg
    }
    
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}