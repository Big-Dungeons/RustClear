use crate::core::block::Block;
use crate::core::types::entity_variant::ObjectVariant;

#[derive(Debug, Copy, Clone)]
pub enum ObjectMetadata {
    FallingBlock {
        block: Block
    },
    EnderPearl
}

impl ObjectMetadata {
    pub fn get_variant(&self) -> ObjectVariant {
        match self {
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
}