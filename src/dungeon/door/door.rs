use crate::dungeon::door::door_entity::{DoorEntityAppearance, DoorEntityExtension};
use crate::dungeon::dungeon::Dungeon;
use glam::{ivec3, DVec3, IVec3};
use rand::prelude::IndexedRandom;
use server::block::block_parameter::Axis;
use server::block::blocks::Blocks;
use server::block::rotatable::Rotatable;
use server::utils::seeded_rng::seeded_rng;
use server::world::chunk::chunk_grid::ChunkGrid;
use server::World;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq)]
pub enum DoorType {
    Normal,
    Entrance,
    Wither,
    Blood,
}

pub struct Door {
    pub x: i32,
    pub z: i32,
    pub axis: Axis,
    door_type: DoorType,
    
    inner_start: IVec3,
    inner_end: IVec3,

    pub is_open: bool,
}

impl Door {
    
    pub fn new(x: i32, z: i32, axis: Axis, door_type: DoorType) -> Self {
        Self {
            x,
            z,
            axis,
            is_open: door_type == DoorType::Normal,
            door_type,
            inner_start: ivec3(x - 1, 69, z - 1),
            inner_end: ivec3(x + 1, 72, z + 1),
        }
    }
    
    pub const fn get_type(&self) -> &DoorType {
        &self.door_type
    }

    const fn get_block(&self) -> Blocks {
        match self.door_type {
            DoorType::Normal => Blocks::Air,
            DoorType::Entrance => Blocks::SilverfishBlock { variant: 5 },
            DoorType::Wither => Blocks::CoalBlock,
            DoorType::Blood => Blocks::StainedHardenedClay { color: 14 }
        }
    }
    
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
            ivec3(self.x - dx, 67, self.z - dz),
            ivec3(self.x + dx, 66, self.z + dz),
        );

        // Might need to replace with a random palette of cobble, stone, gravel etc if we want to mimic hypixel FULLY, but this works fine.
        chunk_grid.fill_blocks(
            Blocks::Stone { variant: 0 },
            ivec3(self.x - (dz - 2) * 2, 68, self.z - (dx - 2) * 2),
            ivec3(self.x + (dz - 2) * 2, 68, self.z + (dx - 2) * 2),
        );

        chunk_grid.fill_blocks(
            Blocks::Air,
            ivec3(self.x - dx, 69, self.z - dz),
            ivec3(self.x + dx, 73, self.z + dz),
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

            let bp = ivec3(x - 2, y, z - 2).rotate(self_direction);

            let mut block_to_place = *block;
            block_to_place.rotate(self_direction);
            chunk_grid.set_block_at(block_to_place, self.x + bp.x, 69 + bp.y, self.z + bp.z);
        }

        chunk_grid.fill_blocks(
            self.get_block(),
            ivec3(self.x - 1, 69, self.z - 1),
            ivec3(self.x + 1, 72, self.z + 1),
        );
    }
    
    pub fn can_open(&self, world: &World<Dungeon>) -> bool {
        if self.is_open {
            return false
        }
        match self.door_type {
            DoorType::Wither => world.wither_key_count != 0,
            DoorType::Blood => world.blood_key_count != 0,
            _ => true
        }
    }
    
    pub fn open(&mut self, world: &mut World<Dungeon>) {
        assert!(!self.is_open, "door is already open");

        match self.door_type {
            DoorType::Wither => {
                assert_ne!(world.wither_key_count, 0, "opened a wither door with 0 keys");
                world.wither_key_count -= 1;
            }
            DoorType::Blood => {
                assert_ne!(world.blood_key_count, 0, "opened blood door with 0 keys");
                world.blood_key_count -= 1;
            }
            _ => {}
        }
        
        self.is_open = true;
        
        world.chunk_grid.fill_blocks(
            Blocks::Barrier,
            ivec3(self.x - 1, 69, self.z - 1),
            ivec3(self.x + 1, 72, self.z + 1),
        );
        // door entity gets rid of blocks when it disappears
        world.spawn_entity(
            DVec3::new(self.x as f64 - 1.0, 69.0, self.z as f64 - 1.0),
            0.0,
            0.0,
            DoorEntityAppearance { block: self.get_block() },
            DoorEntityExtension {}
        );
    }

    // inner bit of door, blocks abilities
    pub fn contains(&self, block_pos: &IVec3) -> bool {
        let (x ,y , z) = (block_pos.x, block_pos.y, block_pos.z);
        (x >= self.inner_start.x && x <= self.inner_end.x) &&
        (y >= self.inner_start.y && y <= self.inner_end.y) &&
        (z >= self.inner_start.z && z <= self.inner_end.z)
    }
}