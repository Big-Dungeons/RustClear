use crate::net::packets::packet_buffer::PacketBuffer;
use crate::net::protocol::play::clientbound::{DestroyEntites, EntityAttach, SpawnObject};
use crate::net::var_int::VarInt;
use crate::server::block::block_parameter::Axis;
use crate::server::block::block_position::BlockPos;
use crate::server::block::blocks::Blocks;
use crate::server::entity::entity::{Entity, EntityImpl};
use crate::server::entity::entity_metadata::{EntityMetadata, EntityVariant};
use crate::server::world;
use crate::server::world::World;
use crate::utils::seeded_rng::seeded_rng;
use glam::DVec3;
use rand::prelude::IndexedRandom;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DoorType {
    NORMAL,
    ENTRANCE,
    WITHER,
    BLOOD,
}

impl DoorType {
    const fn get_block(&self) -> Blocks {
        match self {
            DoorType::NORMAL => Blocks::Air,
            DoorType::ENTRANCE => Blocks::SilverfishBlock { variant: 5 },
            DoorType::WITHER => Blocks::CoalBlock,
            DoorType::BLOOD => Blocks::StainedHardenedClay { color: 14 }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Door {
    pub x: i32,
    pub z: i32,

    pub direction: Axis,
    pub door_type: DoorType,
}

impl Door {

    pub fn load_into_world(
        &self,
        world: &mut World,
        // maybe somehow make this constant? idk
        door_blocks: &HashMap<DoorType, Vec<Vec<Blocks>>>,
    ) {
        // Area to fill with air
        let (dx, dz) = match self.direction {
            Axis::X => (3, 2),
            _ => (2, 3),
        };

        // Doors have a thick bedrock floor usually
        world.fill_blocks(
            Blocks::Bedrock,
            BlockPos { x: self.x - dx, y: 67, z: self.z - dz },
            BlockPos { x: self.x + dx, y: 66, z: self.z + dz },
        );

        // Might need to replace with a random palette of cobble, stone, gravel etc if we want to mimic hypixel FULLY, but this works fine.
        world.fill_blocks(
            Blocks::Stone { variant: 0 },
            BlockPos { x: self.x - (dz - 2) * 2, y: 68, z: self.z - (dx - 2) * 2 },
            BlockPos { x: self.x + (dz - 2) * 2, y: 68, z: self.z + (dx - 2) * 2 },
        );

        world.fill_blocks(
            Blocks::Air,
            BlockPos { x: self.x - dx, y: 69, z: self.z - dz },
            BlockPos { x: self.x + dx, y: 73, z: self.z + dz },
        );

        // Pretty much just to get a normal self from a wither one,
        // since wither doors are just normal doors with coal blocks.
        let door_type = match self.door_type {
            DoorType::BLOOD => DoorType::BLOOD,
            DoorType::ENTRANCE => DoorType::ENTRANCE,
            DoorType::WITHER | DoorType::NORMAL => DoorType::NORMAL,
        };

        let block_data = door_blocks.get(&door_type).unwrap();
        let chosen = block_data.choose(&mut seeded_rng()).unwrap();
        let self_direction = self.direction.get_direction();

        for (i, block) in chosen.iter().enumerate() {
            let x = (i % 5) as i32;
            let z = ((i / 5) % 5) as i32;
            let y = (i / (5 * 5)) as i32;

            let bp = BlockPos { x: x - 2, y, z: z - 2 }.rotate(self_direction);

            let mut block_to_place = block.clone();
            block_to_place.rotate(self_direction);
            world.set_block_at(block_to_place, self.x + bp.x, 69 + bp.y, self.z + bp.z);
        }

        world.fill_blocks(
            self.door_type.get_block(),
            BlockPos { x: self.x - 1, y: 69, z: self.z - 1 },
            BlockPos { x: self.x + 1, y: 72, z: self.z + 1 },
        );
    }


    pub fn open_door(&self, world: &mut World) {
        if cfg!(debug_assertions) {
            assert_ne!(self.door_type, DoorType::NORMAL);
        }

        let start = BlockPos { x: self.x - 1, y: 69, z: self.z - 1 };
        let end = BlockPos { x: self.x + 1, y: 72, z: self.z + 1 };

        let mut entities = Vec::new();
        world::iterate_blocks(start, end, |x,y, z| {
            world.set_block_at(Blocks::Barrier, x, y, z);
            world.interactable_blocks.remove(&BlockPos { x, y, z });

            let id = world.spawn_entity(
                DVec3::new(x as f64 + 0.5, y as f64 - DOOR_ENTITY_OFFSET, z as f64 + 0.5),
                EntityMetadata {
                    variant: EntityVariant::Bat { hanging: false },
                    is_invisible: true
                },
                DoorEntityImpl::new(self.door_type.get_block(), 5.0, 20),
            ).unwrap();
            entities.push(id);
        });

        world.server_mut().schedule(20, move |server| {
            world::iterate_blocks(start, end, |x,y, z| {
                server.world.set_block_at(Blocks::Air, x, y, z);
            });
        });

        // world.server_mut().dungeon.test.push(OpenDoorTask {
        //     ticks_left: 20,
        //     door_index: self.id,
        // });
    }
}

// this maybe could be used in places other than doors, ie when you flick lever 

/// this entity implementation is used for doors in dungeons to animate them.
///
/// it spawns a falling block entity that rides the entity.
/// every tick it lowers the y position, you must remove the entity to make it stop
#[derive(Debug)]
pub struct DoorEntityImpl {
    pub block: Blocks,
    distance_per_tick: f64,
    ticks_left: u32,
}

impl DoorEntityImpl {
    pub fn new(block: Blocks, distance: f64, ticks: u32) -> Self {
        Self {
            block,
            distance_per_tick: distance / ticks as f64,
            ticks_left: ticks,
        }
    }
}

/// offset so that the falling block riding the bat
pub const DOOR_ENTITY_OFFSET: f64 = 0.65;

impl EntityImpl for DoorEntityImpl {

    fn spawn(&mut self, entity: &mut Entity, buffer: &mut PacketBuffer) {
        let world = entity.world_mut();
        let entity_id = world.new_entity_id();

        let object_data = {
            let block_state_id = self.block.get_block_state_id() as i32;
            let block_id = block_state_id >> 4;
            let metadata = block_state_id & 0b1111;
            block_id | (metadata << 12)
        };

        buffer.write_packet(&SpawnObject {
            entity_id: VarInt(entity_id),
            entity_variant: 70,
            x: entity.position.x,
            y: entity.position.y + DOOR_ENTITY_OFFSET,
            z: entity.position.z,
            yaw: 0.0,
            pitch: 0.0,
            data: object_data,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
        });

        buffer.write_packet(&EntityAttach {
            entity_id,
            vehicle_id: entity.id,
            leash: false,
        });
    }

    fn despawn(&mut self, entity: &mut Entity, buffer: &mut PacketBuffer) {
        buffer.write_packet(&DestroyEntites {
            entities: vec![VarInt(entity.id + 1)],
        });
    }

    fn tick(&mut self, entity: &mut Entity, _: &mut PacketBuffer) {
        entity.position.y -= self.distance_per_tick;
        self.ticks_left -= 1;
        if self.ticks_left == 0 {
            entity.world_mut().despawn_entity(entity.id);
        }
    }
}