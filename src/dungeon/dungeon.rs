use crate::block::block_parameter::Axis;
use crate::block::rotatable::Rotatable;
use crate::dungeon::door::door::{Door, DoorType};
use crate::dungeon::door::door_positions::DOOR_POSITIONS;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::dungeon_items::DungeonItem;
use crate::dungeon::room::room::{Room, RoomNeighbour, RoomSegment};
use crate::dungeon::room::room_data::{get_random_data_with_type, RoomData, RoomShape, RoomType};
use crate::inventory::menu::OpenContainer;
use crate::network::binary::var_int::VarInt;
use crate::network::protocol::play::clientbound::{Chat, EntityProperties, PlayerAbilities};
use crate::player::attribute::{Attribute, AttributeMap, AttributeModifier};
use crate::player::player::{ClientId, GameProfile, Player};
use crate::types::block_position::BlockPos;
use crate::types::chat_component::ChatComponent;
use crate::utils::hasher::deterministic_hasher::DeterministicHashMap;
use crate::world::world::{World, WorldExtension};
use anyhow::bail;
use glam::IVec2;
use std::collections::HashMap;
use uuid::Uuid;

pub const DUNGEON_ORIGIN: IVec2 = IVec2::new(-200, -200);

pub enum DungeonState {
    NotStarted,
    Starting { starts_in_ticks: usize },
    Started { ticks: usize }
}

pub struct Dungeon {
    pub rooms: Vec<Room>,
    pub doors: Vec<Door>,
    room_index_grid: [Option<usize>; 36],
    entrance_room_index: usize,

    pub state: DungeonState
}

impl WorldExtension for Dungeon {

    type Player = DungeonPlayer;

    fn tick(world: &mut World<Self>) {
        let dungeon = &mut world.extension;

        match &mut dungeon.state {
            DungeonState::Starting { starts_in_ticks: tick } => {
                *tick -= 1;
                if *tick == 0 {
                    dungeon.state = DungeonState::Started { ticks: 0 };
                    world.start_dungeon();
                } else if *tick % 20 == 0 {

                    let seconds_remaining = *tick / 20;
                    let s = if seconds_remaining == 1 { "" } else { "s" };
                    let str = format!("§aStarting in {} second{}.", seconds_remaining, s);

                    for player in world.players.iter_mut() {
                        player.write_packet(&Chat {
                            component: ChatComponent::new(str.clone()),
                            chat_type: 0,
                        });
                    }
                }
            }
            DungeonState::Started { ticks } => {
                *ticks += 1;
            }
            _ => {}
        }
    }

    fn on_player_join(world: &mut World<Self>, profile: GameProfile, client_id: ClientId) {
        let entrance = world.extension.entrance_room();
        let position = entrance.get_world_block_pos(&BlockPos::new(15, 72, 18)).as_dvec3_centered();
        
        let player = world.spawn_player(
            position,
            180.0.rotate(entrance.rotation),
            0.0,
            profile,
            client_id,
            DungeonPlayer { is_ready: false }
        );

        let speed: f32 = 500.0 * 0.001;

        let mut attributes = AttributeMap::new();
        attributes.insert(Attribute::MovementSpeed, speed as f64);
        attributes.add_modify(Attribute::MovementSpeed, AttributeModifier {
            id: Uuid::parse_str("662a6b8d-da3e-4c1c-8813-96ea6097278d").unwrap(),
            amount: 0.3, // this is always 0.3 for hypixels speed stuff
            operation: 2,
        });

        player.write_packet(&EntityProperties {
            entity_id: VarInt(player.entity_id),
            properties: attributes, // this gets sent every time you sprint for some reason
        });
        player.write_packet(&PlayerAbilities {
            invulnerable: false,
            flying: false,
            allow_flying: false,
            creative_mode: false,
            fly_speed: 0.0,
            walk_speed: speed,
        });

        player.inventory.set_slot(43, Some(DungeonItem::AspectOfTheVoid));
        player.inventory.set_slot(37, Some(DungeonItem::Pickaxe));
        player.inventory.set_slot(44, Some(DungeonItem::SkyblockMenu));
        player.sync_inventory();
        player.flush_packets()
    }
}

impl World<Dungeon> {
    
    pub fn start_dungeon(&mut self) {
        for player in self.players.iter_mut() {
            if let OpenContainer::Menu(_) = player.open_container {
                player.open_container(OpenContainer::None)
            } 
        }
        
        // might be bad idea
        for door in unsafe { self.extension_mut() }.doors.iter_mut() {
            if door.door_type == DoorType::Entrance {
                door.open(self);
                break
            }
        }
    }
    
    pub const fn has_started(&self) -> bool {
        matches!(self.extension.state, DungeonState::Started { .. })
    }
    
    pub fn update_ready_status(&mut self, player: &mut Player<DungeonPlayer>) {
        assert!(!matches!(self.state, DungeonState::Started { .. }), "tried to ready up when dungeon has already started");

        let is_ready = player.extension.is_ready;
        let message = format!("§7{} {}!", player.profile.username, if is_ready { "§ais now ready" } else { "§cis no longer ready" });
        
        let packet = Chat {
            component: ChatComponent::new(message),
            chat_type: 0,
        };
        
        for player in self.players.iter_mut() {
            player.write_packet(&packet)
        }
        
        if is_ready {
            let mut should_start = true;

            for player in self.players.iter() {
                if !player.extension.is_ready {
                    should_start = false
                }
            }
            if should_start {
                self.state = DungeonState::Starting { starts_in_ticks: 100 }
            }
        } else {
            self.state = DungeonState::NotStarted
        }
    }
}

impl Dungeon {

    pub fn from_string(
        layout_str: &str,
        room_data_storage: &DeterministicHashMap<usize, RoomData>,
    ) -> anyhow::Result<Dungeon> {
        
        let mut rooms: Vec<Room> = Vec::new();
        let mut doors: Vec<Door> = Vec::new();

        for (index, position) in DOOR_POSITIONS.into_iter().enumerate() {
            let Some(type_string) = layout_str.get(index + 72..index+73) else {
                bail!("Failed to parse door type.");
            };
            let door_type = match type_string {
                "0" => Some(DoorType::Normal),
                "1" => Some(DoorType::Wither),
                "2" => Some(DoorType::Blood),
                "3" => Some(DoorType::Entrance),
                _ => None,
            };
            if let Some(door_type) = door_type {
                let axis = match ((position.x - DUNGEON_ORIGIN.x) / 16) % 2 == 0 {
                    true => Axis::Z,
                    false => Axis::X,
                };
                doors.push(Door::new(position.x, position.y, axis, door_type));
            }
        }
        
        let mut room_id_map: HashMap<usize, Vec<RoomSegment>> = HashMap::new();
        
        for index in 0..36 {
            let x = index % 6;
            let z = index / 6;

            let Some(id) = layout_str.get(index * 2..index * 2 + 2) else {
                bail!("Failed to parse dungeon string: too small.")
            };
            let Ok(id) = id.parse::<usize>() else {
                bail!("Failed to parse dungeon string: invalid number.")
            };

            // no room here
            if id == 0 {
                continue;
            }

            let mut segment = RoomSegment {
                x,
                z,
                neighbours: [const { None }; 4],
            };

            // find neighbouring doors
            let center_x = x as i32 * 32 + 15 + DUNGEON_ORIGIN.x;
            let center_z = z as i32 * 32 + 15 + DUNGEON_ORIGIN.y;

            let door_options = [
                (center_x, center_z - 16),
                (center_x + 16, center_z),
                (center_x, center_z + 16),
                (center_x - 16, center_z)
            ];
            for (index, (door_x, door_z)) in door_options.into_iter().enumerate() {
                let door = doors.iter().enumerate().find(|(_, door)| {
                    door.x == door_x && door.z == door_z
                });

                if let Some((door_index, _)) = door {
                    segment.neighbours[index] = Some(RoomNeighbour {
                        room_index: 0, // will be populated later? if at all
                        door_index,
                    });
                }
            }
            
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

                // Fairy can have a varying number of doors, all other special rooms are fixed to just one.
                let shape = match room_type {
                    RoomType::Fairy => RoomShape::OneByOne,
                    _ => RoomShape::OneByOneEnd,
                };

                let mut room_data = get_random_data_with_type(
                    room_type,
                    shape,
                    room_data_storage,
                    &rooms
                );
                room_data.room_type = room_type;
                rooms.push(Room::new(vec![segment], room_data));
                continue;
            }
            
            let entry = room_id_map.entry(id).or_default(); 
            entry.push(segment)
        }
        
        rooms.reserve(room_id_map.len());
        
        for segments in room_id_map.into_values() {
            let shape = RoomShape::from_segments(&segments);
            let data = get_random_data_with_type(RoomType::Normal, shape, &room_data_storage, &rooms);
            rooms.push(Room::new(segments, data))
        }

        // populate room index grid
        let mut room_grid: [Option<usize>; 36] = [const { None }; 36];
        let mut grid_max_x = 0;
        let mut grid_max_y = 0;
        let mut entrance_room_index = 0;

        for (index, room) in rooms.iter().enumerate() {
            if room.data.room_type == RoomType::Entrance {
                entrance_room_index = index;
            }
            for segment in room.segments.iter() {
                let x = segment.x;
                let z = segment.z;
                let segment_index = x + z * 6;

                if segment_index > room_grid.len() - 1 {
                    bail!("Segment index for {},{} out of bounds: {}", x, z, segment_index);
                }
                if room_grid[segment_index].is_some() {
                    bail!("Segment at {x}, {z} is already occupied by {:?}!", room_grid[segment_index]);
                }
                room_grid[segment_index] = Some(index);

                if x > grid_max_x {
                    grid_max_x = x;
                }
                if z > grid_max_y {
                    grid_max_y = z;
                }
            }
        }


        Ok(Dungeon {
            rooms,
            doors,
            room_index_grid: room_grid,
            entrance_room_index,
            state: DungeonState::NotStarted,
        })
    }

    pub fn entrance_room(&self) -> &Room {
        &self.rooms[self.entrance_room_index]
    }
    
    // maybe this also considers room bounds? im not sure
    pub fn get_room_at(&self, x: i32, z: i32) -> Option<usize> {
        if x < DUNGEON_ORIGIN.x || z < DUNGEON_ORIGIN.y { 
            return None;
        }
        let grid_x = ((x - DUNGEON_ORIGIN.x) / 32) as usize;
        let grid_z = ((z - DUNGEON_ORIGIN.y) / 32) as usize;
        *self.room_index_grid.get(grid_x + (grid_z * 6)).unwrap_or_else(|| &None)
    }
    
    pub fn get_player_room(&self, player: &Player<DungeonPlayer>) -> Option<(usize, Option<usize>)> {
        let room_index = self.get_room_at(
            player.position.x as i32,
            player.position.z as i32
        );
        if let Some(index) = room_index {
            let player_aabb = player.collision_aabb();

            for room_bounds in self.rooms[index].room_bounds.iter() {
                if player_aabb.intersects(&room_bounds.aabb) {
                    return Some((index, room_bounds.segment_index))
                }
            }
        }
        None
    }
}