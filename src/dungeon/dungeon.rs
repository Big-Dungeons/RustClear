use crate::dungeon::door::door::{Door, DoorType};
use crate::dungeon::door::door_positions::DOOR_POSITIONS;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::dungeon_items::DungeonItem;
use crate::dungeon::map::DungeonMap;
use crate::dungeon::room::room::{Room, RoomNeighbour, RoomSegment};
use crate::dungeon::room::room_data::{get_random_data_with_type, RoomData, RoomShape, RoomType};
use anyhow::bail;
use glam::{ivec3, DVec3, IVec2};
use server::block::block_parameter::Axis;
use server::block::rotatable::Rotate;
use server::constants::Gamemode;
use server::inventory::menu::OpenContainer;
use server::network::binary::var_int::VarInt;
use server::network::protocol::play::clientbound::{Chat, EntityProperties, PlayerAbilities};
use server::player::attribute::{Attribute, AttributeMap, AttributeModifier};
use server::player::sidebar::Sidebar;
use server::types::aabb::AABB;
use server::types::chat_component::ChatComponent;
use server::utils::hasher::deterministic_hasher::DeterministicHashMap;
use server::{ClientId, GameProfile, Player, World, WorldExtension};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

pub const DUNGEON_ORIGIN: IVec2 = IVec2::new(-200, -200);

pub enum DungeonState {
    NotStarted,
    Starting { starts_in_ticks: usize },
    Started { ticks: usize }
}

pub struct Dungeon {
    pub rooms: Vec<Rc<RefCell<Room>>>,
    pub doors: Vec<Rc<RefCell<Door>>>,
    room_index_grid: [Option<usize>; 36],
    entrance_room_index: usize,

    pub state: DungeonState,
    pub map: DungeonMap,

    // there can be only 1 blood key in hypixel,
    // but what if we ever want to do some fun stuff with more than 1
    pub blood_key_count: usize,
    pub wither_key_count: usize,
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
                    Dungeon::start_dungeon(world);
                } else if *tick % 20 == 0 {

                    let seconds_remaining = *tick / 20;
                    let s = if seconds_remaining == 1 { "" } else { "s" };
                    let str = format!("§aStarting in {} second{}.", seconds_remaining, s);

                    // todo: global packet buffer for these global stuff
                    for player in world.players_mut() {
                        player.write_packet(&Chat {
                            component: ChatComponent::new(str.clone()),
                            chat_type: 0,
                        });
                    }
                }
            }
            DungeonState::Started { ticks } => {
                *ticks += 1;

                for player_rc in world.players.iter_mut() {
                    let player = unsafe { &mut *player_rc.get() };

                    let Some((new_room, new_segment)) = world.extension.get_room(&player.position, player.collision_aabb()) else {
                        if let Some((old_room, _)) = &player.extension.current_room {
                            // was in a room, now in no room
                            old_room.borrow_mut().remove_player_ref(player.client_id);
                            player.extension.current_room = None
                        }
                        continue;
                    };
                    if let Some((old_room, old_segment)) = &mut player.extension.current_room {
                        if !Rc::ptr_eq(old_room, new_room) {
                            // was in a different room, moved to new room
                            old_room.borrow_mut().remove_player_ref(player.client_id);
                            new_room.borrow_mut().add_player_ref(player.client_id, player_rc.clone());
                            player.extension.current_room = Some((new_room.clone(), new_segment));
                        } else if *old_segment != new_segment {
                            // same room, different segment
                            // new_room.borrow_mut().update_player_segment(player.client_id, new_segment);
                            *old_segment = new_segment;
                        }
                    } else {
                        // no previous room
                        new_room.borrow_mut().add_player_ref(player.client_id, player_rc.clone());
                        player.extension.current_room = Some((new_room.clone(), new_segment));
                    }
                }

                // iterate over rooms and sections ,
                // if room has any players, tick it
                // and for sections if it has a player, try to spawn its neighbours

                // tick puzzles, etc

                for index in 0..world.rooms.len() {
                    let room_rc = world.rooms[index].clone();
                    let mut room = room_rc.borrow_mut();

                    if !room.players.is_empty() && !room.discovered {
                        let room_impl = unsafe { &mut *room.implementation.get() };
                        room_impl.discover(&mut room, world);

                        room.discovered = true;
                        world.extension.map.draw_room(&room);
                    }
                }

                if let Some(packet) = world.extension.map.get_packet() {
                    for player in world.players_mut() {
                        player.write_packet(&packet)
                    }
                }
            }
            _ => {}
        }
    }

    fn on_player_join(world: &mut World<Self>, profile: GameProfile, client_id: ClientId) {
        let entrance = world.extension.entrance_room().clone();
        let entrance = entrance.borrow();

        let mut position = entrance.get_world_block_position(ivec3(15, 72, 18)).as_dvec3();
        position.x += 0.5;
        position.z += 0.5;
        
        let player = world.spawn_player(
            position,
            180.0.rotate(entrance.rotation),
            0.0,
            profile,
            client_id,
            Gamemode::Survival,
            DungeonPlayer { 
                is_ready: false,
                sidebar: Sidebar::new(),
                current_room: None,
                cooldowns: HashMap::new(),
                active_abilities: Cell::new(Vec::new()),
            }
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

        player.extension.sidebar.write_init_packets(&mut player.packet_buffer);
        
        player.inventory.set_slot(37, Some(DungeonItem::Hyperion));
        player.inventory.set_slot(39, Some(DungeonItem::Pickaxe));
        player.inventory.set_slot(42, Some(DungeonItem::TacticalInsertion));
        player.inventory.set_slot(43, Some(DungeonItem::AspectOfTheVoid));
        player.inventory.set_slot(44, Some(DungeonItem::SkyblockMenu));
        player.sync_inventory();
        player.flush_packets()
    }

    fn on_player_leave(_: &mut World<Self>, player: &mut Player<Self::Player>) {
        if let Some((room_rc, _)) = &player.current_room {
            let mut room = room_rc.borrow_mut();
            room.remove_player_ref(player.client_id)
        }
    }
}

impl Dungeon {

    pub fn has_started(&self) -> bool {
        matches!(self.state, DungeonState::Started { .. })
    }

    pub fn start_dungeon(world: &mut World<Self>) {
        for player in world.players_mut() {
            if let OpenContainer::Menu(_) = player.get_container() {
                player.open_container(OpenContainer::None)
            }
            player.inventory.set_slot(44, Some(DungeonItem::MagicalMap));
            player.sync_inventory();
        }

        {
            let entrance_room = world.entrance_room();
            entrance_room.borrow_mut().discovered = true;
            world.map.draw_room(&entrance_room.borrow());
        }

        for neighbour in world.entrance_room().borrow().neighbours() {
            neighbour.door.borrow_mut().open(world);
            neighbour.room.borrow_mut().discovered = true;
            world.map.draw_room(&neighbour.room.borrow());
        }
    }

    pub fn update_ready_status(world: &mut World<Self>, player: &mut Player<DungeonPlayer>) {
        assert!(!matches!(world.state, DungeonState::Started { .. }), "tried to ready up when dungeon has already started");

        let is_ready = player.extension.is_ready;
        let message = format!("§7{} {}!", player.profile.username, if is_ready { "§ais now ready" } else { "§cis no longer ready" });

        let packet = Chat {
            component: ChatComponent::new(message),
            chat_type: 0,
        };

        for player in world.players_mut() {
            player.write_packet(&packet)
        }

        if is_ready {
            let mut should_start = true;

            for player in world.players_mut() {
                if !player.extension.is_ready {
                    should_start = false
                }
            }
            if should_start {
                world.state = DungeonState::Starting { starts_in_ticks: 100 }
            }
        } else {
            world.state = DungeonState::NotStarted
        }
    }

    pub fn from_string(
        layout_str: &str,
        room_data_storage: &DeterministicHashMap<usize, RoomData>,
    ) -> anyhow::Result<Dungeon> {
        
        let mut rooms: Vec<Rc<RefCell<Room>>> = Vec::new();
        let mut doors: Vec<Rc<RefCell<Door>>> = Vec::new();

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
                doors.push(Rc::new(RefCell::new(Door::new(position.x, position.y, axis, door_type))));
            }
        }

        // used in room neighbours as a temporary value,
        // if the reference count of this != 1 after finishing initializing it will fail
        let placeholder_neighbour = Rc::new(RefCell::new(Room::new(vec![RoomSegment {
            x: 0,
            z: 0,
            neighbours: [const { None }; 4],
            player_ref_count: 0,
        }], RoomData::dummy())));

        let mut room_id_map: DeterministicHashMap<usize, Vec<RoomSegment>> = DeterministicHashMap::default();
        
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
                player_ref_count: 0,
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
                    let door = door.borrow();
                    door.x == door_x && door.z == door_z
                });
                if let Some((_, door)) = door {
                    segment.neighbours[index] = Some(RoomNeighbour {
                        room: placeholder_neighbour.clone(),
                        door: door.clone(),
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
                rooms.push(Rc::new(RefCell::new(Room::new(vec![segment], room_data))));
                continue;
            }
            
            let entry = room_id_map.entry(id).or_default(); 
            entry.push(segment)
        }
        
        rooms.reserve(room_id_map.len());
        
        for segments in room_id_map.into_values() {
            let shape = RoomShape::from_segments(&segments);
            let data = get_random_data_with_type(RoomType::Normal, shape, room_data_storage, &rooms);
            rooms.push(Rc::new(RefCell::new(Room::new(segments, data))))
        }

        // populate room index grid
        let mut room_grid: [Option<usize>; 36] = [const { None }; 36];
        let mut grid_max_x = 0;
        let mut grid_max_y = 0;
        let mut entrance_room_index = 0;

        for (index, room) in rooms.iter().enumerate() {
            let room = room.borrow();
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

        // replace placeholder_neighbour with its actual neighbours
        for room_rc in rooms.iter() {
            for segment in room_rc.borrow_mut().segments.iter_mut() {
                for (index, neighbour) in segment.neighbours.iter_mut().enumerate() {
                    if let Some(neighbour) = neighbour {
                        // shouldn't be possible to error unless its an invalid dungeon
                        let neighbour_position = match index {
                            0 => segment.x + ((segment.z - 1) * 6),
                            1 => (segment.x + 1) + (segment.z * 6),
                            2 => segment.x + ((segment.z + 1) * 6),
                            3 => (segment.x - 1) + (segment.z * 6),
                            _ => unreachable!()
                        };
                        neighbour.room = rooms[room_grid[neighbour_position].unwrap()].clone()
                    }
                }
            }
        }

        if Rc::strong_count(&placeholder_neighbour) > 1 {
            bail!("Placeholder neighbour is leaked, likely caused by invalid room layout")
        }

        // temp
        let mut wither_key_count = 0;

        for door in doors.iter() {
            let door = door.borrow();
            if let DoorType::Wither = door.get_type() {
                wither_key_count += 1;
            }
        }

        let map_offset_x = (128 - (grid_max_x + 1) * 20) / 2;
        let map_offset_y = (128 - (grid_max_y + 1) * 20) / 2;

        Ok(Dungeon {
            rooms,
            doors,
            room_index_grid: room_grid,
            entrance_room_index,
            state: DungeonState::NotStarted,
            map: DungeonMap::new(map_offset_x, map_offset_y),
            wither_key_count,
            blood_key_count: 1,

        })
    }

    pub fn entrance_room(&self) -> Rc<RefCell<Room>> {
        self.rooms[self.entrance_room_index].clone()
    }

    pub fn get_room(&self, position: &DVec3, aabb: AABB) -> Option<(&Rc<RefCell<Room>>, Option<usize>)> {
        let grid_index = grid_position(position.x as i32, position.z as i32)?;
        let room_index = *self.room_index_grid.get(grid_index)?.as_ref()?;
        let room_rc = &self.rooms[room_index];

        for room_aabb in room_rc.borrow().room_bounds.iter() {
            if room_aabb.aabb.intersects(&aabb) {
                return Some((room_rc, room_aabb.segment_index))
            }
        }
        None
    }
}

fn grid_position(x: i32, z: i32) -> Option<usize> {
    if x < DUNGEON_ORIGIN.x || z < DUNGEON_ORIGIN.y {
        return None;
    }
    let grid_x = ((x - DUNGEON_ORIGIN.x) / 32) as usize;
    let grid_z = ((z - DUNGEON_ORIGIN.y) / 32) as usize;
    Some(grid_x + (grid_z * 6))
}