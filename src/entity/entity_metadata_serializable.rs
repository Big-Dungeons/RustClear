use crate::network::packets::packet_serializable::PacketSerializable;
use crate::player::inventory::item_stack::ItemStack;
use enumset::{EnumSet, EnumSetType};
use glam::IVec3;

pub(super) trait EntityMetadataSerializable: PacketSerializable {
    const ID: u8;
}

impl EntityMetadataSerializable for bool {
    const ID: u8 = 0;
}

impl EntityMetadataSerializable for u8 {
    const ID: u8 = 0;
}

impl EntityMetadataSerializable for i8 {
    const ID: u8 = 0;
}

impl EntityMetadataSerializable for i16 {
    const ID: u8 = 1;
}

impl EntityMetadataSerializable for i32 {
    const ID: u8 = 2;
}

impl EntityMetadataSerializable for f32 {
    const ID: u8 = 3;
}

impl EntityMetadataSerializable for &str {
    const ID: u8 = 4;
}

impl EntityMetadataSerializable for String {
    const ID: u8 = 4;
}

impl EntityMetadataSerializable for ItemStack {
    const ID: u8 = 5;
}

impl EntityMetadataSerializable for Option<ItemStack> {
    const ID: u8 = 5;
}

impl EntityMetadataSerializable for IVec3 {
    const ID: u8 = 6;
}

impl<E : EnumSetType> EntityMetadataSerializable for EnumSet<E> {
    const ID: u8 = 0;
}