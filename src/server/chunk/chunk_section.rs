use crate::server::block::blocks::Blocks;

/// ChunkSection represents a 16x16x16 cube of blocks.
///
/// (This is based on ExtendedBlockStorage in 1.8.9, only difference being there is no lighting).
pub struct ChunkSection {
    pub data: [u16; 4096],
    pub solid_block_amount: u16,
}

impl ChunkSection {

    /// Creates a new empty ChunkSection.
    /// Blocks must be added later.
    pub fn new() -> ChunkSection {
        Self {
            data: [0; 4096],
            solid_block_amount: 0,
        }
    }

    pub fn get_block(&self, index: usize) -> Blocks {
        Blocks::from(self.data[index])
    }
    
    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> Blocks {
        let index = (y << 8) | (z << 4) | x;
        Blocks::from(self.data[index as usize])
    }
    

    pub fn set_block(&mut self, block: Blocks, index: usize) {
        if self.data[index] != 0 {
            self.solid_block_amount -= 1;
        }
        if block != Blocks::Air {
            self.solid_block_amount += 1;
        }
        let block_state_id = block.get_block_state_id();
        self.data[index] = block_state_id;
    }

    pub fn set_block_at(&mut self, block: Blocks, x: i32, y: i32, z: i32) {
        let index = (y << 8) | (z << 4) | x;
        self.set_block(block, index as usize)
    }

    pub fn is_empty(&self) -> bool {
        self.solid_block_amount == 0
    }
}