use crate::network::packets::packet_serialize::PacketSerializable;
use bytes::BytesMut;

#[repr(i8)]
#[derive(Copy, Clone)]
pub enum Gamemode {
    Survival,
    Creative,
}

impl PacketSerializable for Gamemode {
    fn write_size(&self) -> usize {
        (*self as i8).write_size()
    }
    fn write(&self, buf: &mut BytesMut) {
        (*self as i8).write(buf)
    }
}