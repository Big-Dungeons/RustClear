use crate::core::block::block_rotation::{Rotate, Rotation};
use crate::core::block::Block;
use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::dungeon::rooms::room_data::RoomData;
use crate::dungeon::DUNGEON_ORIGIN;
use bevy::prelude::{Component, Entity, Query, ResMut, Resource};
use glam::{ivec3, DVec3, IVec3, USizeVec2, Vec3Swizzles};
use std::cmp::{max, min};
use bevy::ecs::entity::EntityHashSet;

pub mod room_data;

#[derive(Component)]
pub struct Room {
    pub rotation: Rotation,
    pub corner: IVec3,

    pub players: EntityHashSet,
}

impl Room {

    pub fn new(segments: &[(RoomSegment, u8)], data: &RoomData) -> Self {
        let rotation = rotation_from_segments(segments);
        let segments: &[&RoomSegment] = &segments.iter().map(|(s, _)| s).collect::<Vec<_>>();
        let corner = corner_position(segments, data, rotation);
        Self {
            rotation,
            corner,
            players: EntityHashSet::new(),
        }
    }

    pub fn relative_to_world(&self, position: IVec3) -> IVec3 {
        let mut position = position.rotate(self.rotation);
        position.x += self.corner.x;
        position.z += self.corner.z;
        position
    }

    pub fn insert_player(&mut self, entity: Entity) {
        debug_assert!(!self.players.contains(&entity), "player already in this room");
        self.players.insert(entity);
    }

    pub fn remove_player(&mut self, entity: Entity) {
        debug_assert!(self.players.contains(&entity), "player was never in this room");
        self.players.remove(&entity);
    }
}

#[derive(Component, Copy, Clone)]
pub struct RoomSegment {
    pub x: usize,
    pub z: usize,
}

#[derive(Component)]
pub struct RoomNeighbours {
    pub room: Entity,
    pub door: Entity,
}

#[derive(Resource)]
pub struct RoomGridLookup([Option<Entity>; 36]);

impl Default for RoomGridLookup {
    fn default() -> Self {
        Self([const { None }; 36])
    }
}

impl RoomGridLookup {
    pub fn get(&self, grid_position: USizeVec2) -> Option<Entity> {
        let index = grid_position.x + grid_position.y * 6;
        if index >= 36 { return None; }
        self.0[index]
    }

    pub fn set(&mut self, grid_position: USizeVec2, entity: Entity) {
        let index = grid_position.x + grid_position.y * 6;
        if index >= 36 { return; }
        self.0[index] = Some(entity);
    }

    // maybe make a custom system param,
    // that has room grid lookup and room aabb query and gets rooms
    // from world position given that it should be inside a room
    pub fn get_from_world(&self, position: DVec3) -> Option<Entity> {
        let position = position.as_ivec3();
        if position.x < DUNGEON_ORIGIN.x || position.z < DUNGEON_ORIGIN.y {
            return None;
        }

        let grid_position = ((position.xz() - DUNGEON_ORIGIN) / 32).as_usizevec2();
        self.get(grid_position)
    }
}

// since y-axis with room segment positions is flipped (y+ is south)
// cw 90 and ccw 90 are flipped
pub fn rotation_from_segments(segments: &[(RoomSegment, u8)]) -> Rotation {
    let mut min_x = usize::MAX;
    let mut min_z = usize::MAX;
    let mut max_x = usize::MIN;
    let mut max_z = usize::MIN;

    for (segment, _) in segments {
        min_x = min(min_x, segment.x);
        min_z = min(min_z, segment.z);
        max_x = max(max_x, segment.x);
        max_z = max(max_z, segment.z);
    }

    let width = (max_x - min_x) + 1;
    let length = (max_z - min_z) + 1;

    match segments.len() {
        1 => {
            let bitmask = &segments[0].1;
            match bitmask {
                // Doors on all sides, never changes
                0b1111 => Rotation::None,
                // Opposite doors
                0b0101 => Rotation::None,
                0b1010 => Rotation::Clockwise90,
                // Dead end | L Bend | Triple Door
                0b1000 | 0b0011 | 0b1011 => Rotation::None,
                0b0100 | 0b1001 | 0b1101 => Rotation::Clockwise90,
                0b0010 | 0b1100 | 0b1110 => Rotation::Clockwise180,
                0b0001 | 0b0110 | 0b0111 => Rotation::CounterClockwise90,
                _ => Rotation::None,
            }
        }
        2 => match length == 1 {
            true => Rotation::None,
            false => Rotation::Clockwise90,
        },
        3 => {
            // L room
            if width == 2 && length == 2 {
                let (corner, _) = segments.iter().find(|(a, _)| {
                    segments.iter().all(|(b, _)| {
                        a.x.abs_diff(b.x) + a.z.abs_diff(b.z) <= 1
                    })
                }).expect("Invalid L room: Segments:");

                match (corner.x, corner.z) {
                    (x, z) if x == min_x && z == min_z => Rotation::Clockwise90,
                    (x, z) if x == max_x && z == min_z => Rotation::Clockwise180,
                    (x, z) if x == max_x && z == max_z => Rotation::CounterClockwise90,
                    _ => Rotation::None,
                }
            } else {
                match length == 1 {
                    true => Rotation::None,
                    false => Rotation::Clockwise90,
                }
            }
        },
        4 => {
            if width == 2 && length == 2 {
                Rotation::None
            } else {
                match length == 1 {
                    true => Rotation::None,
                    false => Rotation::Clockwise90,
                }
            }
        },
        _ => unreachable!(),
    }
}

fn corner_position(segments: &[&RoomSegment], data: &RoomData, rotation: Rotation) -> IVec3 {
    // due to segments being flipped
    let min_x = segments.iter().min_by(|a, b| a.x.cmp(&b.x)).unwrap().x;
    let min_z = segments.iter().min_by(|a, b| a.z.cmp(&b.z)).unwrap().z;
    let x = min_x as i32 * 32 + DUNGEON_ORIGIN.x;
    let y = 68;
    let z = min_z as i32 * 32 + DUNGEON_ORIGIN.y;
    match rotation {
        Rotation::None => ivec3(x, y, z),
        Rotation::Clockwise90 => ivec3(x + data.length - 1, y, z),
        Rotation::Clockwise180 => ivec3(x + data.length - 1, y, z + data.width - 1),
        Rotation::CounterClockwise90 => ivec3(x, y, z + data.width - 1),
    }
}

// might have issue where rooms overlap, but im pretty sure its fixed
pub fn load_rooms_into_world(
    query: Query<(&Room, &RoomData)>,
    mut chunks: ResMut<ChunkGrid>
) {
    for (room, data) in query.iter() {
        for (index, block) in data.block_data.iter().enumerate() {
            if *block == Block::Air {
                continue;
            }

            let index = index as i32;
            let x = index % data.width;
            let z = (index / data.width) % data.length;
            let y = data.bottom + index / (data.width * data.length);

            let bp = ivec3(x, y, z).rotate(room.rotation);
            let block = block.rotate(room.rotation);

            chunks.set_block_at(block, (room.corner.x + bp.x, y, room.corner.z + bp.z));
        }
    }
}