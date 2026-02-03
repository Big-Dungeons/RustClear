use crate::dungeon::door::door::Door;
use crate::dungeon::dungeon::{Dungeon, DUNGEON_ORIGIN};
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::room::puzzles::quiz::QuizPuzzle;
use crate::dungeon::room::puzzles::three_weirdos::ThreeWeirdosPuzzle;
use crate::dungeon::room::room_data::RoomData;
use crate::dungeon::room::room_implementation::{MobRoom, RoomImplementation};
use glam::{dvec3, ivec3, usize, IVec3};
use server::block::rotatable::Rotate;
use server::block::Block;
use server::network::protocol::play::clientbound::Chat;
use server::types::aabb::AABB;
use server::types::chat_component::ChatComponent;
use server::types::direction::Direction;
use server::world::chunk::chunk_grid::ChunkGrid;
use server::{ClientId, Player, World};
use std::cell::{RefCell, UnsafeCell};
use std::cmp::{max, min};
use std::collections::HashMap;
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

pub enum RoomStatus {
    Undiscovered,
    Discovered,
    Complete,
    Failed,
}

pub struct Room {
    pub segments: Vec<RoomSegment>,
    pub room_bounds: Vec<RoomBounds>,
    pub rotation: Direction,
    pub data: RoomData,

    pub status: RoomStatus,

    pub players: HashMap<ClientId, Rc<UnsafeCell<Player<DungeonPlayer>>>>,
    pub implementation: UnsafeCell<Box<dyn RoomImplementation>>
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
            let max_y = (room_data.bottom + room_data.height) as f64 + 100.0;

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

        let implementation: UnsafeCell<Box<dyn RoomImplementation>> = match room_data.name.as_str() {
            "Three Weirdos" => UnsafeCell::new(Box::new(ThreeWeirdosPuzzle::default())),
            "Quiz" => UnsafeCell::new(Box::new(QuizPuzzle {})),
            _ => UnsafeCell::new(Box::new(MobRoom {})),
        };

        Self {
            segments,
            room_bounds,
            rotation,
            data: room_data,
            status: RoomStatus::Undiscovered,
            implementation,
            players: HashMap::new(),
        }
    }

    pub fn neighbours(&self) -> impl Iterator<Item = &RoomNeighbour> {
        self.segments.iter().flat_map(|seg| seg.neighbours.iter().flatten())
    }

    pub fn players(&mut self) -> impl Iterator<Item = &mut Player<DungeonPlayer>> {
        self.players.values().map(|it| unsafe { &mut *it.get() })
    }

    pub fn get_corner_pos(&self) -> IVec3 {
        Room::get_corner_pos_from(&self.segments, &self.rotation, &self.data)
    }

    pub fn get_corner_pos_from(
        segments: &[RoomSegment],
        rotation: &Direction,
        room_data: &RoomData
    ) -> IVec3 {
        let min_x = segments.iter().min_by(|a, b| a.x.cmp(&b.x)).unwrap().x;
        let min_z = segments.iter().min_by(|a, b| a.z.cmp(&b.z)).unwrap().z;

        let x = min_x as i32 * 32 + DUNGEON_ORIGIN.x;
        let y = 68;
        let z = min_z as i32 * 32 + DUNGEON_ORIGIN.y;

        match rotation {
            Direction::North => ivec3(x, y, z),
            Direction::East => ivec3(x + room_data.length - 1, y, z),
            Direction::South => ivec3(x + room_data.length - 1, y, z + room_data.width - 1),
            Direction::West => ivec3(x, y, z + room_data.width - 1),
        }
    }

    pub fn load_into_world(&self, chunk_grid: &mut ChunkGrid<Dungeon>) {
        let corner = self.get_corner_pos();

        for (index, block) in self.data.block_data.iter().enumerate() {
            if *block == Block::Air {
                continue;
            }

            let index = index as i32;
            let x = index % self.data.width;
            let z = (index / self.data.width) % self.data.length;
            let y = self.data.bottom + index / (self.data.width * self.data.length);

            let bp = ivec3(x, y, z).rotate(self.rotation);
            let block = block.rotate(self.rotation);

            chunk_grid.set_block_at(block, corner.x + bp.x, y, corner.z + bp.z);
        }
    }

    pub fn add_player_ref(&mut self, client_id: ClientId, player: Rc<UnsafeCell<Player<DungeonPlayer>>>/*segment: Option<usize>*/) {
        debug_assert!(!self.players.contains_key(&client_id), "player already in room");
        self.players.insert(client_id, player);
    }

    pub fn remove_player_ref(&mut self, client_id: ClientId) {
        debug_assert!(self.players.contains_key(&client_id), "player wasn't in the room");
        self.players.remove(&client_id);
    }

    pub fn get_world_block_position(&self, room_position: IVec3) -> IVec3 {
        let corner = self.get_corner_pos();
        let mut position = room_position.rotate(self.rotation);
        position.x += corner.x;
        position.z += corner.z;
        position
    }

    pub fn is_undiscovered(&self) -> bool {
        matches!(self.status, RoomStatus::Undiscovered)
    }

    // run essentially when it discovers a room...
    pub fn discover(&mut self, world: &mut World<Dungeon>) {
        self.status = RoomStatus::Discovered;
        let implementation = unsafe { &mut *self.implementation.get() };
        implementation.discover(self, world);

        world.map.draw_room(self)
    }

    pub fn tick(&mut self, world: &mut World<Dungeon>) {
        let implementation = unsafe { &mut *self.implementation.get() };
        implementation.tick(self, world)
    }

    pub fn interact_with_block(
        room_rc: &Rc<RefCell<Room>>,
        player: &mut Player<DungeonPlayer>,
        position: IVec3
    ) {
        if Self::try_open_door(room_rc, player, position) {
            return;
        }

        let mut room = room_rc.borrow_mut();
        let implementation = unsafe { &mut *room.implementation.get() };
        implementation.interact(&mut room, player, position);

        drop(room);

        // let room = room_rc.borrow();
        // let relative = {
        //     let corner = room.get_corner_pos();
        //     let p = IVec3 {
        //         x: position.x - corner.x,
        //         y: position.y,
        //         z: position.z - corner.z,
        //     };
        //     match room.rotation {
        //         Direction::North => p,
        //         Direction::East => IVec3 { x: p.z, y: p.y, z: -p.x },
        //         Direction::South => IVec3 { x: -p.x, y: p.y, z: -p.z },
        //         Direction::West => IVec3 { x: -p.z, y: p.y, z: p.x },
        //     }
        // };

        // player.write_packet(&Chat {
        //     component: ChatComponent::new(format!("relative position {}", relative)),
        //     chat_type: 0,
        // })
    }

    pub fn attack_block(room_rc: &Rc<RefCell<Room>>, player: &mut Player<DungeonPlayer>, position: IVec3) {
        if Self::try_open_door(room_rc, player, position) {
            return;
        }
        // player.write_packet(&Chat {
        //     component: ChatComponent::new("lc block"),
        //     chat_type: 0,
        // })
    }

    fn try_open_door(room_rc: &Rc<RefCell<Room>>, player: &mut Player<DungeonPlayer>, position: IVec3) -> bool {
        let room = room_rc.borrow();
        let world = player.world_mut();
        for neighbour in room.neighbours() {
            let mut door = neighbour.door.borrow_mut();
            if !door.contains(position) || door.is_open {
                continue;
            }
            if !door.can_open(world) {
                // todo: proper chat message and sound
                player.write_packet(&Chat {
                    component: ChatComponent::new("no key"),
                    chat_type: 0,
                });
                continue;
            }
            door.open(world);
            drop(door);

            neighbour.room.borrow_mut().discover(world);
            return true
        }
        false
    }
}

fn get_rotation_from_segments(segments: &[RoomSegment]) -> Direction {
    let mut min_x = usize::MAX;
    let mut min_z = usize::MAX;
    let mut max_x = usize::MIN;
    let mut max_z = usize::MIN;

    for segment in segments {
        min_x = min(min_x, segment.x);
        min_z = min(min_z, segment.z);
        max_x = max(max_x, segment.x);
        max_z = max(max_z, segment.z);
    }

    let width = (max_x - min_x) + 1;
    let length = (max_z - min_z) + 1;

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
                0b1111 => Direction::North,
                // Opposite doors
                0b0101 => Direction::North,
                0b1010 => Direction::East,
                // Dead end | L Bend | Triple Door
                0b1000 | 0b0011 | 0b1011 => Direction::North,
                0b0100 | 0b1001 | 0b1101 => Direction::East,
                0b0010 | 0b1100 | 0b1110 => Direction::South,
                0b0001 | 0b0110 | 0b0111 => Direction::West,
                _ => Direction::North,
            }
        }
        2 => match length == 1 {
            true => Direction::North,
            false => Direction::East,
        },
        3 => {
            // L room
            if width == 2 && length == 2 {
                let corner = segments.iter().find(|a| {
                    segments.iter().all(|b| {
                        a.x.abs_diff(b.x) + a.z.abs_diff(b.z) <= 1
                    })
                }).expect("Invalid L room: Segments:");

                match (corner.x, corner.z) {
                    (x, z) if x == min_x && z == min_z => Direction::East,
                    (x, z) if x == max_x && z == min_z => Direction::South,
                    (x, z) if x == max_x && z == max_z => Direction::West,
                    _ => Direction::North,
                }
            } else {
                match length == 1 {
                    true => Direction::North,
                    false => Direction::East,
                }
            }
        },
        4 => {
            if width == 2 && length == 2 {
                Direction::North
            } else {
                match length == 1 {
                    true => Direction::North,
                    false => Direction::East,
                }
            }
        },
        _ => unreachable!(),
    }
}