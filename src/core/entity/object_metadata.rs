use crate::core::block::Block;
use crate::core::block::block_rotation::Rotation;
use crate::core::entity::entity_metadata::MobFlags;
use crate::core::entity::entity_metadata_serializable::EntityMetadataSerializable;
use crate::core::network::packets::packet_serializable::PacketSerializable;
use crate::core::network::protocol::sized_string::SizedString;
use crate::core::player::inventory::item_stack::ItemStack;
use crate::core::types::entity_variant::ObjectVariant;
use enumset::EnumSet;
use macros::entity_metadata;

// add more if they become needed
#[derive(Debug, Clone)]
pub enum PaintingVariant {
    Wanderer,
    Graham,
}

impl PaintingVariant {
    pub fn get_string(&self) -> SizedString<16> {
        match self {
            PaintingVariant::Wanderer => SizedString::new("Wanderer"),
            PaintingVariant::Graham => SizedString::new("Graham"),
        }
    }
}

// objects are super scuffed compared to normal entities, cuz of 1.8.9 code... :(
#[derive(Debug, Clone)]
pub enum ObjectMetadata {
    Painting {
        painting: PaintingVariant,
        rotation: Rotation,
    },
    DroppedItem {
        item: ItemStack,
    },
    Minecart,
    FallingBlock {
        block: Block,
    },
    EnderPearl,
}

entity_metadata! {
    // only some have actual metadata
    #[derive(Debug, Clone)]
    pub enum ObjectEntityMetadata {
        DroppedItem {
            10 => pub item: ItemStack = ItemStack::new(),
        }
    }
}

impl ObjectMetadata {
    pub fn get_variant(&self) -> ObjectVariant {
        match self {
            Self::Painting { .. } => unreachable!(), // painting has its own packet
            Self::DroppedItem { .. } => ObjectVariant::DroppedItem,
            Self::Minecart => ObjectVariant::Minecart,
            Self::FallingBlock { .. } => ObjectVariant::FallingBlock,
            Self::EnderPearl => ObjectVariant::EnderPearl,
        }
    }

    pub fn get_data(&self) -> i32 {
        match self {
            Self::FallingBlock { block } => {
                let block_state_id = block.get_blockstate_id() as i32;
                let block_id = block_state_id >> 4;
                let metadata = block_state_id & 0b1111;
                block_id | (metadata << 12)
            }
            _ => 0,
        }
    }

    pub fn get_entity_metadata(&self) -> Option<ObjectEntityMetadata> {
        match self {
            Self::DroppedItem { item } => {
                Some(DroppedItemMetadata::new().item(item.clone()).into())
            }
            _ => None,
        }
    }
}
