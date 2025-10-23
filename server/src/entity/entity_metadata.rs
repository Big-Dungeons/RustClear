use crate::inventory::item_stack::ItemStack;
use crate::network::packets::packet_serialize::PacketSerializable;
use blocks::entity_metadata_serializable;
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

entity_metadata_serializable! {
    pub struct PlayerMetadata {
        10 => pub layers: u8,
    }
}

entity_metadata_serializable! {
    #[derive(Debug, Copy, Clone)]
    pub enum EntityMetadata {
        Zombie {
            12 => pub is_baby: bool = false,
            13 => pub is_villager: bool = false,
        },
        Bat {
            16 => pub hanging: bool = false,
        }
    }
}


// entity_metadata_serializable! {
//     pub enum ObjectMetadata {
//         FallingBlock {
//
//         }
//     }
// }