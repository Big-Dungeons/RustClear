use crate::core::block::Block;
use crate::core::entity::entity_metadata_serializable::EntityMetadataSerializable;
use crate::core::network::packets::packet_serializable::PacketSerializable;
use crate::core::player::inventory::item_stack::ItemStack;
use crate::core::types::entity_variant::ObjectVariant;
use macros::entity_metadata;

#[derive(Debug, Clone)]
pub enum ObjectMetadata {
    DroppedItem {
        item: ItemStack,
    },
     FallingBlock {
        block: Block
    },
    EnderPearl
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
            Self::DroppedItem { .. } => ObjectVariant::DroppedItem,
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
            _ => 0
        }
    }

    pub fn get_entity_metadata(&self) -> Option<ObjectEntityMetadata> {
        match self {
            Self::DroppedItem { item } => Some(DroppedItemMetadata { item: item.clone() }.into()),
            _ => None,
        }
    }
}