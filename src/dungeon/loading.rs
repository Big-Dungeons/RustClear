use crate::core::block::block_rotation::Rotate;
use crate::core::entity::components::interactable::Interactable;
use crate::core::entity::components::transform::Transform;
use crate::core::entity::entity_metadata::ZombieMetadata;
use crate::core::entity::Mob;
use crate::core::player::inventory::menu::{Menu, OpenMenu};
use crate::dungeon::door::door_positions::DOOR_POSITIONS;
use crate::dungeon::door::{Door, DoorAxis, DoorLookup, DoorType};
use crate::dungeon::entities::npc::NPCBehaviour;
use crate::dungeon::menus::MortMenu;
use crate::dungeon::rng::{DHashMap, DHashSet, SeededRng};
use crate::dungeon::rooms::room_data::{random_room_data, RoomData, RoomDataLookup, RoomShape, RoomType};
use crate::dungeon::rooms::{Room, RoomGridLookup, RoomSegment};
use crate::dungeon::{door, rooms, DungeonState, EntranceRoom, DUNGEON_ORIGIN};
use bevy::app::App;
use bevy::prelude::{default, ChildOf, Commands, Deref, Entity, IntoScheduleConfigs, Plugin, PostStartup, Query, Res, ResMut, Resource, Startup, State};
use glam::{ivec2, ivec3};
use rand::prelude::IndexedRandom;

pub(super) struct DungeonLoadingPlugin;

impl Plugin for DungeonLoadingPlugin {
    fn build(&self, app: &mut App) {
        let dungeon_layouts = include_str!("../../DungeonData/dungeon_layouts.txt")
            .split("\n")
            .collect::<Vec<&str>>();

        let mut rng = app.world_mut().resource_mut::<SeededRng>();

        let layout = dungeon_layouts
            .choose(&mut rng)
            .unwrap()
            .to_string();

        app
            .insert_resource(Layout(layout))
            .insert_resource(RoomDataLookup::default())
            .insert_resource(RoomGridLookup::default())
            .insert_resource(DoorLookup::default())
            .add_systems(
                Startup,
                (
                    create_rooms_and_doors,
                    populate_room_grid,
                    set_entrance_room_resource,
                ).chain()
            )
            .add_systems(
                PostStartup,
                (
                    rooms::load_rooms_into_world,
                    door::load_doors_into_world
                ).chain(),
            );
    }
}

// stores layout string
#[derive(Resource, Deref)]
struct Layout(String);

fn create_rooms_and_doors(
    layout: Res<Layout>,
    mut commands: Commands,
    room_data_lookup: Res<RoomDataLookup>,
    mut door_lookup: ResMut<DoorLookup>,
    mut rng: ResMut<SeededRng>
) {
    // rooms
    let mut room_segments: DHashMap<usize, Vec<(RoomSegment, u8)>> = default();
    let mut existing_rooms: DHashSet<String> = default();

    // maybe have a neighbour bitmask lookup, that is filled here
    for (index, position) in DOOR_POSITIONS.into_iter().enumerate() {
        let Some(type_string) = layout.get(index + 72..index + 73) else {
            panic!("Failed to parse door type.");
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
                true => DoorAxis::Z,
                false => DoorAxis::X,
            };
            let door_entity = commands.spawn(Door {
                position,
                axis,
                door_type,
            });
            door_lookup.insert(position, door_entity.id());
        }
    }

    for index in 0..36 {
        let x = index % 6;
        let z = index / 6;

        let Some(id) = layout.get(index * 2..index * 2 + 2) else {
            // panic, since if it fails to parse it would be pointless to try to continue.
            panic!("Failed to parse dungeon string: too small.")
        };
        let Ok(id) = id.parse::<usize>() else {
            panic!("Failed to parse dungeon string: invalid number.")
        };

        if id == 0 {
            continue;
        }

        let segment = RoomSegment { x, z };
        let mut neighbour_bitmask: u8 = 0;

        let cx = x as i32 * 32 + 15 + DUNGEON_ORIGIN.x;
        let cz = z as i32 * 32 + 15 + DUNGEON_ORIGIN.y;

        let door_options = [
            ivec2(cx, cz - 16), // north
            ivec2(cx + 16, cz), // east
            ivec2(cx, cz + 16), // south
            ivec2(cx - 16, cz), // west
        ];

        for position in door_options {
            neighbour_bitmask <<= 1;
            neighbour_bitmask |= door_lookup.contains_key(&position) as u8;
        }

        if id <= 6 {
            let room_type = match id {
                1 => RoomType::Entrance,
                2 => RoomType::Fairy,
                3 => RoomType::Blood,
                4 => RoomType::Puzzle,
                5 => RoomType::Trap,
                6 => RoomType::Yellow,
                _ => unreachable!(),
            };

            // Fairy can have a varying number of doors, all other special rooms are fixed to just one.
            let shape = match room_type {
                RoomType::Fairy => RoomShape::OneByOne,
                _ => RoomShape::OneByOneEnd,
            };

            let mut room_data = random_room_data(&room_data_lookup, room_type, shape, &existing_rooms, &mut rng);
            room_data.room_type = room_type;

            existing_rooms.insert(room_data.name.clone());

            commands
                .spawn((
                    Room::new(&[(segment, neighbour_bitmask)], &room_data),
                    room_data,
                ))
                .with_child(segment);
            continue;
        }

        let entry = room_segments.entry(id).or_default();
        entry.push((segment, neighbour_bitmask));
    }

    for segments in room_segments.into_values() {
        let shape = RoomShape::from_segments(&segments);
        let data = random_room_data(&room_data_lookup, RoomType::Normal, shape, &existing_rooms, &mut rng);
        existing_rooms.insert(data.name.clone());
        
        let mut sorted = segments;
        sorted.sort_by(|(a, _), (b, _)| a.z.cmp(&b.z).then(a.x.cmp(&b.x)));

        commands
            .spawn((Room::new(&sorted, &data), data))
            .with_children(|parent| {
                for (segment, _) in sorted {
                    parent.spawn(segment);
                }
            });
    }
}

fn populate_room_grid(
    query: Query<(&RoomSegment, &ChildOf)>,
    mut room_grid: ResMut<RoomGridLookup>,
) {
    for (segment, parent) in &query {
        room_grid.set((segment.x, segment.z).into(), parent.parent());
    }
}

fn set_entrance_room_resource(
    query: Query<(Entity, &Room, &RoomData)>,
    mut commands: Commands,
) {
    for (entity, room, room_data) in query.iter() {
        if let RoomType::Entrance = room_data.room_type {
            commands.insert_resource(EntranceRoom { entity });

            // mort
            let mut position = room.relative_to_world(ivec3(15, 69, 4)).as_dvec3();
            position.x += 0.5;
            position.z += 0.5;
            let yaw = 0.0.rotate(room.rotation);

            commands.spawn((
                Mob::new(ZombieMetadata {
                    is_baby: false,
                    is_villager: false,
                }),
                Transform {
                    position,
                    yaw,
                    pitch: 0.0,
                },
                NPCBehaviour {
                    default_yaw: yaw,
                    default_pitch: 0.0,
                },
                Interactable::new(|world, player, _| {
                    if let DungeonState::Started { .. } = world.resource::<State<DungeonState>>().get() {
                        return;
                    }
                    let menu_entity = world.spawn((
                        Menu {
                            title: "Ready Up".to_string(),
                            items: vec![const { None }; 54],
                        },
                        MortMenu,
                        ChildOf(player),
                    )).id();
                    world.trigger(OpenMenu { menu_entity, entity: player });
                })
            ));

            // should only be one entrance per map
            break;
        }
    }
}

// todo: add populate neighbours, so segments can have ref to entity of neighbour room