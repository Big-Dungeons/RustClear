use crate::core::network::packets::packet_serializable::PacketSerializable;
use bytes::BytesMut;

#[repr(i8)]
#[derive(Debug, Copy, Clone)]
pub enum EntityVariant {
    ArmorStand = 30,
    Zombie = 54,
    Bat = 65,
}

#[repr(i8)]
#[derive(Debug, Copy, Clone)]
pub enum ObjectVariant {
    DroppedItem = 2,
    EnderPearl = 65,
    FallingBlock = 70
}

impl PacketSerializable for EntityVariant {
    fn write_size(&self) -> usize {
        size_of::<u8>()
    }
    fn write(&self, buf: &mut BytesMut) {
        (*self as i8).write(buf)
    }
}

impl PacketSerializable for ObjectVariant {
    fn write_size(&self) -> usize {
        size_of::<u8>()
    }
    fn write(&self, buf: &mut BytesMut) {
        (*self as i8).write(buf)
    }
}