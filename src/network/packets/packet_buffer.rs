use crate::network::binary::var_int::{var_int_size, write_var_int};
use crate::network::internal_packets::NetworkThreadMessage;
use crate::network::packets::packet::IdentifiedPacket;
use crate::network::packets::packet_serialize::PacketSerializable;
use crate::player::player::ClientId;
use bytes::BytesMut;

#[derive(Debug)]
pub struct PacketBuffer {
    buffer: BytesMut,
}

impl PacketBuffer {
    
    pub fn new() -> Self {
        Self {
            buffer: BytesMut::new()
        }
    }

    pub fn write_packet<P : IdentifiedPacket + PacketSerializable>(&mut self, packet: &P) {
        let write_size = (var_int_size(P::PACKET_ID) + packet.write_size()) as i32;
        self.buffer.reserve(write_size as usize + var_int_size(write_size));
        
        write_var_int(&mut self.buffer, write_size);
        write_var_int(&mut self.buffer, P::PACKET_ID);
        packet.write(&mut self.buffer);
    }

    /// gets a message for network thread to send the packets inside the buffer to the client.
    pub fn get_packet_message(&mut self, client_id: ClientId) -> NetworkThreadMessage {
        let buffer = self.buffer.split().freeze();
        NetworkThreadMessage::SendPackets { client_id, buffer }
    }

    #[inline]
    pub fn copy_from(&mut self, buf: &PacketBuffer) {
        self.buffer.extend_from_slice(&buf.buffer)
    }
    
    #[inline]
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
    
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}