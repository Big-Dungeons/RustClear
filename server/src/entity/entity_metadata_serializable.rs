use crate::inventory::item_stack::ItemStack;
use crate::network::packets::packet_serialize::PacketSerializable;
use bytes::BytesMut;
use enumset::{EnumSet, EnumSetType};
use glam::IVec3;

// used on types that can be represented in an entities metadata
pub trait MetadataSerializable: PacketSerializable {
    const ID: u8;
}

impl MetadataSerializable for bool {
    const ID: u8 = 0;
}

impl MetadataSerializable for u8 {
    const ID: u8 = 0;
}

impl MetadataSerializable for i8 {
    const ID: u8 = 0;
}

impl MetadataSerializable for i16 {
    const ID: u8 = 1;
}

impl MetadataSerializable for i32 {
    const ID: u8 = 2;
}

impl MetadataSerializable for f32 {
    const ID: u8 = 3;
}

impl MetadataSerializable for &str {
    const ID: u8 = 4;
}

impl MetadataSerializable for String {
    const ID: u8 = 4;
}

impl MetadataSerializable for ItemStack {
    const ID: u8 = 5;
}

impl MetadataSerializable for Option<ItemStack> {
    const ID: u8 = 5;
}

impl MetadataSerializable for IVec3 {
    const ID: u8 = 6;
}

impl<E : EnumSetType> PacketSerializable for EnumSet<E> {
    fn write_size(&self) -> usize {
        u8::write_size(&self.as_u8())
    }
    fn write(&self, buf: &mut BytesMut) {
        u8::write(&self.as_u8(), buf)
    }
}

impl<E : EnumSetType> MetadataSerializable for EnumSet<E> {
    const ID: u8 = 0;
}