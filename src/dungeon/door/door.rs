use crate::block::block_parameter::Axis;
use crate::block::blocks::Blocks;
use crate::types::block_position::BlockPos;
use crate::utils::seeded_rng::seeded_rng;
use crate::world::chunk::chunk_grid::ChunkGrid;
use rand::prelude::IndexedRandom;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq)]
pub enum DoorType {
    Normal,
    Entrance,
    Wither,
    Blood,
}

impl DoorType {
    const fn get_block(&self) -> Blocks {
        match self {
            DoorType::Normal => Blocks::Air,
            DoorType::Entrance => Blocks::SilverfishBlock { variant: 5 },
            DoorType::Wither => Blocks::CoalBlock,
            DoorType::Blood => Blocks::StainedHardenedClay { color: 14 }
        }
    }
}

pub struct Door {
    pub x: i32,
    pub z: i32,
    pub axis: Axis,
    pub door_type: DoorType
}

impl Door {
    
    pub fn load_into_world(
        &self,
        chunk_grid: &mut ChunkGrid,
        door_blocks: &HashMap<DoorType, Vec<Vec<Blocks>>>
    ) {
        // Area to fill with air
        let (dx, dz) = match self.axis {
            Axis::X => (3, 2),
            _ => (2, 3),
        };

        // Doors have a thick bedrock floor usually
        chunk_grid.fill_blocks(
            Blocks::Bedrock,
            BlockPos::new(self.x - dx, 67, self.z - dz),
            BlockPos::new(self.x + dx, 66, self.z + dz),
        );

        // Might need to replace with a random palette of cobble, stone, gravel etc if we want to mimic hypixel FULLY, but this works fine.
        chunk_grid.fill_blocks(
            Blocks::Stone { variant: 0 },
            BlockPos::new(self.x - (dz - 2) * 2, 68, self.z - (dx - 2) * 2),
            BlockPos::new(self.x + (dz - 2) * 2, 68, self.z + (dx - 2) * 2),
        );

        chunk_grid.fill_blocks(
            Blocks::Air,
            BlockPos::new(self.x - dx, 69, self.z - dz),
            BlockPos::new(self.x + dx, 73, self.z + dz),
        );

        // Pretty much just to get a normal self from a wither one,
        // since wither doors are just normal doors with coal blocks.
        let door_type = match self.door_type {
            DoorType::Blood => DoorType::Blood,
            DoorType::Entrance => DoorType::Entrance,
            DoorType::Wither | DoorType::Normal => DoorType::Normal,
        };

        let block_data = door_blocks.get(&door_type).unwrap();
        let chosen = block_data.choose(&mut seeded_rng()).unwrap();
        let self_direction = self.axis.get_direction();

        for (index, block) in chosen.iter().enumerate() {
            let x = (index % 5) as i32;
            let y = (index / (5 * 5)) as i32;
            let z = ((index / 5) % 5) as i32;

            let bp = BlockPos::new(x - 2, y, z - 2).rotate(self_direction);

            let mut block_to_place = block.clone();
            block_to_place.rotate(self_direction);
            chunk_grid.set_block_at(block_to_place, self.x + bp.x, 69 + bp.y, self.z + bp.z);
        }

        chunk_grid.fill_blocks(
            self.door_type.get_block(),
            BlockPos::new(self.x - 1, 69, self.z - 1),
            BlockPos::new(self.x + 1, 72, self.z + 1),
        );
    }
    
}