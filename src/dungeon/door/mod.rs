use crate::block::block_parameters::BlockColor;
use crate::block::block_rotation::{Rotate, Rotation};
use crate::block::Block;
use crate::chunk::chunk_grid::{iterate_blocks, ChunkGrid};
use crate::entity::components::riding::Riding;
use crate::entity::components::transform::Transform;
use crate::entity::entity_metadata::BatMetadata;
use crate::entity::object_metadata::ObjectMetadata;
use crate::entity::Mob;
use bevy::prelude::{Commands, Component, Deref, DerefMut, Entity, EntityEvent, Local, On, Query, ResMut, Resource};
use glam::{dvec3, ivec3, IVec2, IVec3};
use rand::prelude::IndexedRandom;
use rand::rng;
use std::collections::HashMap;

pub mod door_positions;

#[derive(Default, Resource, Deref, DerefMut)]
pub struct DoorLookup(HashMap<IVec2, Entity>);

#[derive(Hash, Eq, PartialEq)]
pub enum DoorType {
    Normal,
    Entrance,
    Wither,
    Blood,
}

pub enum DoorAxis {
    Z,
    X
}

#[derive(Component)]
pub struct Door {
    pub position: IVec2,
    pub axis: DoorAxis,
    pub door_type: DoorType,
}

impl Door {
    pub const fn get_block(&self) -> Block {
        match self.door_type {
            DoorType::Normal => Block::Air,
            DoorType::Entrance => Block::SilverfishChiseledStoneBrick,
            DoorType::Wither => Block::CoalBlock,
            DoorType::Blood => Block::StainedHardenedClay { color: BlockColor::Red }
        }
    }
}

#[derive(EntityEvent)]
pub struct OpenDoorEvent {
    pub entity: Entity
}

pub fn open_door(
    event: On<OpenDoorEvent>,
    door_query: Query<&Door>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    let door = door_query
        .get(event.entity)
        .unwrap();

    iterate_blocks(
        ivec3(door.position.x - 1, 69, door.position.y - 1),
        ivec3(door.position.x + 1, 72, door.position.y + 1),
        |IVec3 { x, y, z}| {
            let bat = commands.spawn((
                Mob::new(BatMetadata {
                    flags: 0,
                    hanging: false,
                }),
                Transform {
                    position: dvec3(x as f64 + 0.5, y as f64 - 0.655, z as f64 + 0.5),
                    yaw: 0.0,
                    pitch: 0.0,
                }
            )).id();

            commands.spawn((
                Mob::new_object(ObjectMetadata::FallingBlock {
                    block: door.get_block(),
                }),
                Transform {
                    position: dvec3(x as f64 + 0.5, y as f64, z as f64 + 0.5),
                    yaw: 0.0,
                    pitch: 0.0,
                },
                Riding(bat)
            ));

            chunks.set_block_at(Block::Air, (x, y, z))
        }
    );
}

pub fn open_entrance_doors(
    door_query: Query<(Entity, &Door)>,
    mut commands: Commands,
) {
    for (entity, door) in door_query.iter() {
        if let DoorType::Entrance = door.door_type {
            commands.trigger(OpenDoorEvent { entity })
        }
    }
}

pub fn load_doors_into_world(
    query: Query<&Door>,
    mut chunks: ResMut<ChunkGrid>,
    door_blocks: Local<DoorBlocks>
) {
    for door in query.iter() {
        let IVec2 { x, y } = door.position;

        let (dx, dz) = match door.axis {
            DoorAxis::X => (3, 2),
            _ => (2, 3),
        };

        // Doors have a thick bedrock floor usually
        chunks.fill_blocks(
            Block::Bedrock,
            ivec3(x - dx, 67, y - dz),
            ivec3(x + dx, 66, y + dz),
        );

        // Might need to replace with a random palette of cobble, stone, gravel etc if we want to mimic hypixel FULLY, but this works fine.
        chunks.fill_blocks(
            Block::Stone,
            ivec3(x - (dz - 2) * 2, 68, y - (dx - 2) * 2),
            ivec3(x + (dz - 2) * 2, 68, y + (dx - 2) * 2),
        );

        chunks.fill_blocks(
            Block::Air,
            ivec3(x - dx, 69, y - dz),
            ivec3(x + dx, 73, y + dz),
        );

        // Pretty much just to get a normal self from a wither one,
        // since wither doors are just normal doors with coal blocks.
        let door_type = match door.door_type {
            DoorType::Blood => DoorType::Blood,
            DoorType::Entrance => DoorType::Entrance,
            DoorType::Wither | DoorType::Normal => DoorType::Normal,
        };

        let block_data = door_blocks.get(&door_type).unwrap();
        // todo: seeded
        let chosen = block_data.choose(&mut rng()).unwrap();

        let rotation = match door.axis {
            DoorAxis::Z => Rotation::None,
            DoorAxis::X => Rotation::CounterClockwise90,
        };

        for (index, block) in chosen.iter().enumerate() {
            let x = (index % 5) as i32;
            let y = (index / (5 * 5)) as i32;
            let z = ((index / 5) % 5) as i32;
            let bp = ivec3(x - 2, y, z - 2).rotate(rotation);

            let block_to_place = block.rotate(rotation);
            chunks.set_block_at(block_to_place, (door.position.x + bp.x, 69 + bp.y, door.position.y + bp.z));
        }

        chunks.fill_blocks(
            door.get_block(),
            ivec3(x - 1, 69, y - 1),
            ivec3(x + 1, 72, y + 1),
        );
    }
}

// would be nice somehow better format so it's not hardcoded
#[derive(Resource, Deref)]
pub struct DoorBlocks(HashMap<DoorType, Vec<Vec<Block>>>);

impl Default for DoorBlocks {
    fn default() -> Self {
        let door_data: Vec<Vec<Block>> = include_str!("../../../DungeonData/door_data/doors.txt")
            .split("\n")
            .map(|line| {
                let mut blocks: Vec<Block> = Vec::new();

                for i in (0..line.len() - 1).step_by(4) {
                    let substr = line.get(i..i + 4).unwrap();
                    let state = u16::from_str_radix(substr, 16).unwrap();

                    blocks.push(Block::from(state));
                }

                blocks
            })
            .collect();

        DoorBlocks(HashMap::from_iter(
            vec![
                (DoorType::Blood, vec![door_data[0].clone()]),
                (DoorType::Entrance, vec![door_data[1].clone()]),
                (
                    DoorType::Normal,
                    vec![
                        door_data[1].clone(),
                        door_data[2].clone(),
                        door_data[3].clone(),
                        door_data[4].clone(),
                        door_data[5].clone(),
                        door_data[6].clone(),
                        door_data[7].clone(),
                    ],
                ),
            ],
        ))
    }
}