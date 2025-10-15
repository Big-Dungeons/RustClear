use crate::inventory::item_stack::ItemStack;
use crate::network::packets::packet_serialize::PacketSerializable;
use bytes::{BufMut, BytesMut};

#[derive(Debug, Clone)]
pub enum EntityVariant {
    NPC {
        npc_id: &'static str
    },
    DroppedItem {
        item: ItemStack,
    },
    ArmorStand,
    Zombie {
        is_child: bool,
        is_villager: bool
    },
    Bat {
        hanging: bool
    },
    FallingBlock,
}

impl EntityVariant {

    /// Returns the mc entity id of the variant 
    pub const fn get_id(&self) -> i8 {
        match self {
            // npcs need to be spawned with SpawnPlayer packet
            EntityVariant::NPC { .. } => unreachable!(),
            EntityVariant::DroppedItem { .. } => 2,
            EntityVariant::ArmorStand => 30,
            EntityVariant::Zombie { .. } => 54,
            EntityVariant::Bat { .. } => 65,
            EntityVariant::FallingBlock => 70,
        }
    }
    
    /// Returns if the variant is an object and needs to be spawned
    /// using Spawn Object packet instead of Spawn Mob
    pub const fn is_object(&self) -> bool {
        matches!(self, EntityVariant::DroppedItem { .. } | EntityVariant::FallingBlock)
    }
}

// maybe just re-do so its more aligned with mc,
// it wouldn't be hard and would make it not need to manually serialize everything

#[derive(Debug, Clone)]
pub struct EntityMetadata {
    // add more needed stuff here
    pub variant: EntityVariant,
    pub is_invisible: bool
}

impl EntityMetadata {
    pub fn new(variant: EntityVariant) -> Self {
        Self {
            variant,
            is_invisible: false,
        }
    }
}

const BYTE: u8 = 0;
const SHORT: u8 = 1;
const INT: u8 = 2;
const FLOAT: u8 = 3;
const STRING: u8 = 4;
const ITEM_STACK: u8 = 5;

fn write_data(buf: &mut BytesMut, data_type: u8, id: u8, data: impl PacketSerializable) {
    buf.put_u8(data_type << 5 | id & 31);
    data.write(buf);
}

impl PacketSerializable for EntityMetadata {
    fn write_size(&self) -> usize {
        const BYTE_SIZE: usize = const { size_of::<u8>() };
        let mut size = BYTE_SIZE * 3;
        match &self.variant {
            EntityVariant::NPC { .. } => {
                size += BYTE_SIZE + BYTE_SIZE;
            }
            EntityVariant::DroppedItem { item } => {
                size += item.write_size() + BYTE_SIZE;
            }
            EntityVariant::Zombie { is_child, is_villager } => {
                size += is_child.write_size() + is_villager.write_size() + BYTE_SIZE + BYTE_SIZE;
            }
            EntityVariant::Bat { hanging } => {
                size += hanging.write_size() + BYTE_SIZE
            }
            _ => {}
        }
        size
    }
    fn write(&self, buf: &mut BytesMut) {
        let mut flags: u8 = 0;

        if self.is_invisible {
            flags |= 0b00100000
        }

        write_data(buf, BYTE, 0, flags);

        match &self.variant {
            EntityVariant::NPC { .. } => {
                // all layers visible
                write_data(buf, BYTE, 10, u8::MAX)
            }
            EntityVariant::DroppedItem { item } => {
                write_data(buf, ITEM_STACK, 10, Some(item.clone()))
            }
            EntityVariant::Zombie { is_child, is_villager } => {
                write_data(buf, BYTE, 12, *is_child);
                write_data(buf, BYTE, 13, *is_villager);
            }
            EntityVariant::Bat { hanging } => {
                write_data(buf, BYTE, 16, *hanging);
            }
            _ => {}
        }
        buf.put_u8(127)
    }
}