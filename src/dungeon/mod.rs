use std::collections::HashMap;

use crate::dungeon::door::{Door, DoorType};
use crate::dungeon::room::Room;
use crate::dungeon::room_data::{get_random_data_with_type, RoomData, RoomShape, RoomType};
use crate::server::block::block_parameter::Axis;
use crate::server::block::blocks::Blocks;
use crate::server::player::Player;
use crate::server::world::World;

pub mod room;
pub mod door;
pub mod crushers;
pub mod room_data;

// The top leftmost corner of the dungeon
const DUNGEON_ORIGIN: (i32, i32) = (0, 0);

const DOOR_POSITIONS: [(i32, i32); 60] = [(DUNGEON_ORIGIN.0 + 31, DUNGEON_ORIGIN.1 + 15), (DUNGEON_ORIGIN.0 + 63, DUNGEON_ORIGIN.1 + 15), (DUNGEON_ORIGIN.0 + 95, DUNGEON_ORIGIN.1 + 15), (DUNGEON_ORIGIN.0 + 127, DUNGEON_ORIGIN.1 + 15), (DUNGEON_ORIGIN.0 + 159, DUNGEON_ORIGIN.1 + 15), (DUNGEON_ORIGIN.0 + 15, DUNGEON_ORIGIN.1 + 31), (DUNGEON_ORIGIN.0 + 47, DUNGEON_ORIGIN.1 + 31), (DUNGEON_ORIGIN.0 + 79, DUNGEON_ORIGIN.1 + 31), (DUNGEON_ORIGIN.0 + 111, DUNGEON_ORIGIN.1 + 31), (DUNGEON_ORIGIN.0 + 143, DUNGEON_ORIGIN.1 + 31), (DUNGEON_ORIGIN.0 + 175, DUNGEON_ORIGIN.1 + 31), (DUNGEON_ORIGIN.0 + 31, DUNGEON_ORIGIN.1 + 47), (DUNGEON_ORIGIN.0 + 63, DUNGEON_ORIGIN.1 + 47), (DUNGEON_ORIGIN.0 + 95, DUNGEON_ORIGIN.1 + 47), (DUNGEON_ORIGIN.0 + 127, DUNGEON_ORIGIN.1 + 47), (DUNGEON_ORIGIN.0 + 159, DUNGEON_ORIGIN.1 + 47), (DUNGEON_ORIGIN.0 + 15, DUNGEON_ORIGIN.1 + 63), (DUNGEON_ORIGIN.0 + 47, DUNGEON_ORIGIN.1 + 63), (DUNGEON_ORIGIN.0 + 79, DUNGEON_ORIGIN.1 + 63), (DUNGEON_ORIGIN.0 + 111, DUNGEON_ORIGIN.1 + 63), (DUNGEON_ORIGIN.0 + 143, DUNGEON_ORIGIN.1 + 63), (DUNGEON_ORIGIN.0 + 175, DUNGEON_ORIGIN.1 + 63), (DUNGEON_ORIGIN.0 + 31, DUNGEON_ORIGIN.1 + 79), (DUNGEON_ORIGIN.0 + 63, DUNGEON_ORIGIN.1 + 79), (DUNGEON_ORIGIN.0 + 95, DUNGEON_ORIGIN.1 + 79), (DUNGEON_ORIGIN.0 + 127, DUNGEON_ORIGIN.1 + 79), (DUNGEON_ORIGIN.0 + 159, DUNGEON_ORIGIN.1 + 79), (DUNGEON_ORIGIN.0 + 15, DUNGEON_ORIGIN.1 + 95), (DUNGEON_ORIGIN.0 + 47, DUNGEON_ORIGIN.1 + 95), (DUNGEON_ORIGIN.0 + 79, DUNGEON_ORIGIN.1 + 95), (DUNGEON_ORIGIN.0 + 111, DUNGEON_ORIGIN.1 + 95), (DUNGEON_ORIGIN.0 + 143, DUNGEON_ORIGIN.1 + 95), (DUNGEON_ORIGIN.0 + 175, DUNGEON_ORIGIN.1 + 95), (DUNGEON_ORIGIN.0 + 31, DUNGEON_ORIGIN.1 + 111), (DUNGEON_ORIGIN.0 + 63, DUNGEON_ORIGIN.1 + 111), (DUNGEON_ORIGIN.0 + 95, DUNGEON_ORIGIN.1 + 111), (DUNGEON_ORIGIN.0 + 127, DUNGEON_ORIGIN.1 + 111), (DUNGEON_ORIGIN.0 + 159, DUNGEON_ORIGIN.1 + 111), (DUNGEON_ORIGIN.0 + 15, DUNGEON_ORIGIN.1 + 127), (DUNGEON_ORIGIN.0 + 47, DUNGEON_ORIGIN.1 + 127), (DUNGEON_ORIGIN.0 + 79, DUNGEON_ORIGIN.1 + 127), (DUNGEON_ORIGIN.0 + 111, DUNGEON_ORIGIN.1 + 127), (DUNGEON_ORIGIN.0 + 143, DUNGEON_ORIGIN.1 + 127), (DUNGEON_ORIGIN.0 + 175, DUNGEON_ORIGIN.1 + 127), (DUNGEON_ORIGIN.0 + 31, DUNGEON_ORIGIN.1 + 143), (DUNGEON_ORIGIN.0 + 63, DUNGEON_ORIGIN.1 + 143), (DUNGEON_ORIGIN.0 + 95, DUNGEON_ORIGIN.1 + 143), (DUNGEON_ORIGIN.0 + 127, DUNGEON_ORIGIN.1 + 143), (DUNGEON_ORIGIN.0 + 159, DUNGEON_ORIGIN.1 + 143), (DUNGEON_ORIGIN.0 + 15, DUNGEON_ORIGIN.1 + 159), (DUNGEON_ORIGIN.0 + 47, DUNGEON_ORIGIN.1 + 159), (DUNGEON_ORIGIN.0 + 79, DUNGEON_ORIGIN.1 + 159), (DUNGEON_ORIGIN.0 + 111, DUNGEON_ORIGIN.1 + 159), (DUNGEON_ORIGIN.0 + 143, DUNGEON_ORIGIN.1 + 159), (DUNGEON_ORIGIN.0 + 175, DUNGEON_ORIGIN.1 + 159), (DUNGEON_ORIGIN.0 + 31, DUNGEON_ORIGIN.1 + 175), (DUNGEON_ORIGIN.0 + 63, DUNGEON_ORIGIN.1 + 175), (DUNGEON_ORIGIN.0 + 95, DUNGEON_ORIGIN.1 + 175), (DUNGEON_ORIGIN.0 + 127, DUNGEON_ORIGIN.1 + 175), (DUNGEON_ORIGIN.0 + 159, DUNGEON_ORIGIN.1 + 175)];

// contains a vec of rooms,
// also contains a grid, containing indices pointing towards the rooms,
//
// contains a vec of doors (for generation)
pub struct Dungeon {
    pub rooms: Vec<Room>,
    pub doors: Vec<Door>,
    // maybe dont use 36 in case of a smaller map ?
    pub index_grid: [usize; 36],
}

impl Dungeon {

    pub fn with_rooms_and_doors(rooms: Vec<Room>, doors: Vec<Door>) -> Dungeon {
        let mut index_grid = [0; 36];

        // populate index grid
        for (room_id, room) in rooms.iter().enumerate() {
            for (x, z) in room.segments.iter() {
                let segment_index = x + z * 6;

                index_grid[segment_index] = room_id + 1;
            }
        }

        // println!("grid {:?}", &index_grid);

        Dungeon {
            rooms,
            doors,
            index_grid,
        }
    }

    // Layout String:
    // 36 x room ids, two digits long each. 00 = no room, 01 -> 06 are special rooms like spawn, puzzles etc
    // 07 -> ... are normal rooms, with unique ids to differentiate them and preserve layout
    // Doors are 60x single digit numbers in the order left -> right top -> down for every spot they can possibly spawn
    pub fn from_string(layout_str: &str, room_data_storage: &HashMap<usize, RoomData>) -> Dungeon {
        let mut rooms: Vec<Room> = Vec::new();
        // For normal rooms which can be larger than 1x1, store their segments and make the whole room in one go later
        let mut room_id_map: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();

        let mut doors: Vec<Door> = Vec::new();

        for i in 0..60usize {
            let type_str = layout_str.get(i + 72..i+73).unwrap();
            let (x, z) = DOOR_POSITIONS[i];

            let door_type = match type_str {
                "0" => Some(DoorType::NORMAL),
                "1" => Some(DoorType::WITHER),
                "2" => Some(DoorType::BLOOD),
                "3" => Some(DoorType::ENTRANCE),
                _ => None,
            };

            if !door_type.is_none() {
                let direction = match x % 2 {
                    0 => Axis::Z,
                    1 => Axis::X,
                    _ => unreachable!(),
                };

                doors.push(Door {
                    x,
                    z,
                    direction,
                    door_type: door_type.unwrap()
                })
            }
        }

        for i in 0..36 {
            let substr = layout_str.get(i*2..i*2+2);
            let x = i % 6;
            let z = i / 6;

            // Shouldn't happen if data is not corrupted
            if substr.is_none() {
                panic!("Failed to parse dungeon string: too small.")
            }

            let id = substr.unwrap().parse::<usize>().unwrap();

            // No room here
            if id == 0 {
                continue;
            }

            // Special rooms
            if id <= 6 {
                let room_type = match id {
                    1 => RoomType::Entrance,
                    2 => RoomType::Fairy,
                    3 => RoomType::Blood,
                    4 => RoomType::Puzzle,
                    5 => RoomType::Trap,
                    6 => RoomType::Yellow,
                    _ => unreachable!()
                };

                let mut room_data = get_random_data_with_type(
                    room_type,
                    RoomShape::OneByOne,
                    room_data_storage
                );

                room_data.room_type = room_type;

                rooms.push(Room::new(
                    vec![(x, z)],
                    &doors,
                    room_data
                ));

                continue
            }

            // Normal rooms, add segments to this specific room id
            let entry = room_id_map.entry(id).or_default();
            entry.push((x, z));
        }

        // Make the normal rooms
        for (_, segments) in room_id_map {
            let shape = RoomShape::from_segments(&segments);

            rooms.push(Room::new(
                segments,
                &doors,
                get_random_data_with_type(
                    RoomType::Normal,
                    shape,
                    room_data_storage
                )
            ));
        }

        Dungeon::with_rooms_and_doors(rooms, doors)
    }

    pub fn get_room_at(&self, x: i32, z: i32) -> Option<&Room> {
        let grid_x = ((x as i32 - DUNGEON_ORIGIN.0) / 32) as usize;
        let grid_z = ((z as i32 - DUNGEON_ORIGIN.1) / 32) as usize;

        let entry = self.index_grid.get(grid_x + (grid_z * 6));

        if entry.is_none_or(|index| *index == 0) {
            return None;
        }

        self.rooms.get(*entry.unwrap() - 1)
    }

    pub fn get_player_room(&self, player: &Player) -> Option<&Room> {
        let server = player.server_mut();
        let entity = player.get_entity(&server.world).unwrap();

        self.get_room_at(entity.pos.x as i32, entity.pos.z as i32)
    }

    pub fn load_door(&self, door: &Door, world: &mut World) {
        for dx in -2..=2 {
            for dz in -2..=2 {
                world.set_block_at(
                    Blocks::Bedrock,
                    door.x + dx,
                    68,
                    door.z + dz
                );
            }
        }
    }

}