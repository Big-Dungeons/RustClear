use crate::dungeon::door::door::Door;
use crate::dungeon::dungeon::DUNGEON_ORIGIN;
use crate::dungeon::room::room_data::RoomData;
use glam::{dvec3, ivec3, IVec3};
use server::block::Block;
use server::block::rotatable::Rotatable;
use server::types::aabb::AABB;
use server::types::direction::Direction3D;
use server::world::chunk::chunk_grid::ChunkGrid;
use server::ClientId;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct RoomSegment {
    pub x: usize,
    pub z: usize,
    pub neighbours: [Option<RoomNeighbour>; 4],
    pub player_ref_count: usize,
}

pub struct RoomNeighbour {
    pub room: Rc<RefCell<Room>>,
    pub door: Rc<RefCell<Door>>
}

pub struct RoomBounds {
    pub aabb: AABB,
    pub segment_index: Option<usize>
}

pub struct Room {
    pub segments: Vec<RoomSegment>,
    pub room_bounds: Vec<RoomBounds>,
    
    pub rotation: Direction3D,
    pub data: RoomData,

    pub discovered: bool,
    // idk if a bool value,
    pub completed: bool,

    // usize is index of the section they're in
    pub players: HashMap<ClientId, Option<usize>>
}

impl Room {
    
    pub fn new(
        segments: Vec<RoomSegment>,
        room_data: RoomData,
    ) -> Self {
        let rotation = get_rotation_from_segments(&segments);
        let mut room_bounds: Vec<RoomBounds> = Vec::new();

        for (index, segment) in segments.iter().enumerate() {
            let x = (segment.x as i32 * 32 + DUNGEON_ORIGIN.x) as f64;
            let y = room_data.bottom as f64;
            let z = (segment.z as i32 * 32 + DUNGEON_ORIGIN.y) as f64;
            let max_y = (room_data.bottom + room_data.height) as f64;

            room_bounds.push(RoomBounds {
                aabb: AABB::new(dvec3(x, y, z), dvec3(x + 31.0, max_y, z + 31.0)),
                segment_index: Some(index),
            });
            if segments.iter().any(|seg| seg.x == segment.x + 1 && seg.z == segment.z) {
                let x = x + 31.0;
                room_bounds.push(RoomBounds {
                    aabb: AABB::new(dvec3(x, y, z), dvec3(x + 1.0, max_y, z + 31.0)),
                    segment_index: None,
                });
            }
            if segments.iter().any(|seg| seg.x == segment.x && seg.z == segment.z + 1) {
                let z = z + 31.0;
                room_bounds.push(RoomBounds {
                    aabb: AABB::new(dvec3(x, y, z), dvec3(x + 31.0, max_y, z + 1.0)),
                    segment_index: None,
                });
            }
            if segments.iter().any(|seg| seg.x == segment.x + 1 && seg.z == segment.z + 1) {
                let x = x + 31.0;
                let z = z + 31.0;
                room_bounds.push(RoomBounds {
                    aabb: AABB::new(dvec3(x, y, z), dvec3(x + 1.0, max_y, z + 1.0)),
                    segment_index: None,
                });
            }
        }
        
        Self {
            segments,
            room_bounds,
            rotation,
            data: room_data,
            discovered: false,
            completed: false,
            players: HashMap::new(),
        }
    }

    pub fn neighbours(&self) -> impl Iterator<Item = &RoomNeighbour> {
        self.segments.iter().flat_map(|seg| seg.neighbours.iter().flatten())
    }

    pub fn get_corner_pos(&self) -> IVec3 {
        Room::get_corner_pos_from(&self.segments, &self.rotation, &self.data)
    }

    pub fn get_corner_pos_from(
        segments: &[RoomSegment],
        rotation: &Direction3D,
        room_data: &RoomData
    ) -> IVec3 {
        let min_x = segments.iter().min_by(|a, b| a.x.cmp(&b.x)).unwrap().x;
        let min_z = segments.iter().min_by(|a, b| a.z.cmp(&b.z)).unwrap().z;

        let x = min_x as i32 * 32 + DUNGEON_ORIGIN.x;
        let y = 68;
        let z = min_z as i32 * 32 + DUNGEON_ORIGIN.y;

        match rotation {
            Direction3D::North => ivec3(x, y, z),
            Direction3D::East => ivec3(x + room_data.length - 1, y, z),
            Direction3D::South => ivec3(x + room_data.length - 1, y, z + room_data.width - 1),
            Direction3D::West => ivec3(x, y, z + room_data.width - 1),
            _ => unreachable!(),
        }
    }
    
    pub fn load_into_world(&self, chunk_grid: &mut ChunkGrid) {
        let corner = self.get_corner_pos();

        for (index, block) in self.data.block_data.iter().enumerate() {
            if *block == Block::Air {
                continue;
            }
            // not sure if editing room data might ruin something,
            // so to be safe im just cloning it
            let mut block = *block;
            // block.rotate(self.rotation);

            let index = index as i32;

            let x = index % self.data.width;
            let z = (index / self.data.width) % self.data.length;
            let y = self.data.bottom + index / (self.data.width * self.data.length);

            let bp = ivec3(x, y, z).rotate(self.rotation);

            chunk_grid.set_block_at(block, corner.x + bp.x, y, corner.z + bp.z);
        }
    }

    pub fn add_player_ref(&mut self, client_id: ClientId, segment: Option<usize>) {
        debug_assert!(!self.players.contains_key(&client_id), "player already in room");
        self.players.insert(client_id, segment);
        if let Some(segment) = segment {
            self.segments[segment].player_ref_count += 1;
        }
    }

    pub fn remove_player_ref(&mut self, client_id: ClientId) {
        debug_assert!(self.players.contains_key(&client_id), "player wasn't in the room");
        let segment = self.players.remove(&client_id).unwrap();

        if let Some(segment) = segment {
            self.segments[segment].player_ref_count -= 1;
        }
    }

    pub fn update_player_segment(&mut self, client_id: ClientId, new: Option<usize>) {
        let old = self.players.get_mut(&client_id).unwrap();
        debug_assert!(*old != new, "tried updated player section, when section hasn't changed");

        if let Some(segment) = *old {
            self.segments[segment].player_ref_count -= 1;
        };
        if let Some(segment) = new {
            self.segments[segment].player_ref_count += 1;
        }
        *old = new;
    }

    pub fn get_world_block_position(&self, room_position: IVec3) -> IVec3 {
        let corner = self.get_corner_pos();
        let mut position = room_position.rotate(self.rotation);
        position.x += corner.x;
        position.z += corner.z;
        position
    }
}

fn get_rotation_from_segments(segments: &[RoomSegment]) -> Direction3D {
    let unique_x = segments.iter()
        .map(|segment| segment.x)
        .collect::<HashSet<usize>>();
    let unique_z = segments.iter()
        .map(|segment| segment.z)
        .collect::<HashSet<usize>>();

    let not_long = unique_x.len() > 1 && unique_z.len() > 1;

    match segments.len() {
        1 => {
            let segment = &segments[0];
            let mut bitmask: u8 = 0;
            for index in 0..4 {
                bitmask <<= 1;
                bitmask |= segment.neighbours[index].is_some() as u8
            }
            match bitmask {
                // Doors on all sides, never changes
                0b1111 => Direction3D::North,
                // Dead end 1x1
                0b1000 => Direction3D::North,
                0b0100 => Direction3D::East,
                0b0010 => Direction3D::South,
                0b0001 => Direction3D::West,
                // Opposite doors
                0b0101 => Direction3D::North,
                0b1010 => Direction3D::East,
                // L bend
                0b0011 => Direction3D::North,
                0b1001 => Direction3D::East,
                0b1100 => Direction3D::South,
                0b0110 => Direction3D::West,
                // Triple door
                0b1011 => Direction3D::North,
                0b1101 => Direction3D::East,
                0b1110 => Direction3D::South,
                0b0111 => Direction3D::West,
                _ => Direction3D::North,
            }
        }
        2 => match unique_z.len() == 1 {
            true => Direction3D::North,
            false => Direction3D::East,
        },
        3 => {
            // L room
            if not_long {
                let corner_value = segments.iter().find(|x| {
                    segments.iter().all(|y| {
                        x.x.abs_diff(y.x) + x.z.abs_diff(y.z) <= 1
                    })
                }).expect("Invalid L room: Segments:");

                let min_x = segments.iter().min_by(|a, b| a.x.cmp(&b.x)).unwrap().x;
                let min_z = segments.iter().min_by(|a, b| a.z.cmp(&b.z)).unwrap().z;
                let max_x = segments.iter().max_by(|a, b| a.x.cmp(&b.x)).unwrap().x;
                let max_z = segments.iter().max_by(|a, b| a.z.cmp(&b.z)).unwrap().z;

                if corner_value.x == min_x && corner_value.z == min_z {
                    return Direction3D::East
                }
                if corner_value.x == max_x && corner_value.z == min_z {
                    return Direction3D::South
                }
                if corner_value.x == max_x && corner_value.z == max_z {
                    return Direction3D::West
                }
                return Direction3D::North
            }

            match unique_z.len() == 1 {
                true => Direction3D::North,
                false => Direction3D::East,
            }
        },
        4 => {
            if unique_x.len() == 2 && unique_z.len() == 2 {
                return Direction3D::North
            }

            match unique_z.len() == 1 {
                true => Direction3D::North,
                false => Direction3D::East,
            }
        },
        _ => unreachable!(),
    }
}