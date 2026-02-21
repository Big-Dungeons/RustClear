#![allow(clippy::collapsible_if, clippy::too_many_arguments, clippy::new_without_default)]

use crate::dungeon::door::door::DoorType;
use crate::dungeon::door::door_entity::DoorBehaviour;
use crate::dungeon::dungeon::{Dungeon, DungeonState};
use crate::dungeon::entities::npc::NPCBehaviour;
use crate::dungeon::menus::MortMenu;
use crate::dungeon::room::room_data::RoomData;
use crate::dungeon::seeded_rng::{seeded_rng, SeededRng};
use anyhow::bail;
use bevy_ecs::component::Component;
use glam::ivec3;
use include_dir::include_dir;
use rand::prelude::IndexedRandom;
use server::block::rotatable::Rotate;
use server::block::Block;
use server::entity::components::entity_appearance::PlayerAppearance;
use server::entity::components::{EntityBehaviour, Interactable};
use server::entity::entity::MinecraftEntity;
use server::inventory::menu::OpenContainer;
use server::network::internal_packets::NetworkThreadMessage;
use server::network::network::start_network;
use server::types::chat_component::{ChatComponent, MCColors};
use server::types::status::Status;
use server::utils::hasher::deterministic_hasher::DeterministicHashMap;
use server::world::world::World;
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::UnboundedSender as Sender;

mod dungeon;

pub fn initialize_world(tx: Sender<NetworkThreadMessage>) -> anyhow::Result<World<Dungeon>> {
    let rng_seed: u64 = rand::random();
    SeededRng::set_seed(rng_seed);

    // tp maze, ice fill, boulder seed 18158556563918935308
    // three weirdo seed 16795237019042391353

    println!("seed {rng_seed}");

    let dungeon_layouts = include_str!("../DungeonData/dungeon_layouts.txt")
        .split("\n")
        .collect::<Vec<&str>>();

    let layout = dungeon_layouts.choose(&mut seeded_rng()).unwrap();

    // todo: fix room heights from moody's room data
    let room_data_storage = &room_data();
    let door_type_blocks = &door_block_data();

    let dungeon = Dungeon::from_string(layout, room_data_storage)?;
    // if you do anything with entities or anything that has a pointer to world.
    // once world moves out of this functions scope
    // it will move in the stack causing those pointers to be invalid,
    // this can be fixed by using Box<T> if it is required
    let mut world = World::new(tx, dungeon);

    for room in world.extension.rooms.iter() {
        room.borrow().load_into_world(&mut world.chunk_grid);
    }
    for door in world.extension.doors.iter() {
        door.borrow().load_into_world(&mut world.chunk_grid, door_type_blocks)
    }

    Ok(world)
}

// test
#[derive(Component)]
struct JumpBehaviour;

impl EntityBehaviour<Dungeon> for JumpBehaviour {
    fn tick(entity: &mut MinecraftEntity<Dungeon>, _: &mut Self) {
        if entity.ticks_existed % 10 >= 5 {
            entity.position.y -= 0.25
        } else {
            entity.position.y += 0.25
        }
    }
}

pub fn spawn_mort(world: &mut World<Dungeon>) {
    let entrance = world.extension.entrance_room();
    let entrance = entrance.borrow();
    let mut position = entrance.get_world_block_position(ivec3(15, 69, 4)).as_dvec3();

    position.x += 0.5;
    position.z += 0.5;

    let yaw = 0.0.rotate(entrance.rotation);

    world.spawn_entity(
        position,
        yaw,
        0.0,
        PlayerAppearance::new(
            "Mort",
            Default::default(),
            "ewogICJ0aW1lc3RhbXAiIDogMTYxODc4MTA4Mzk0NywKICAicHJvZmlsZUlkIiA6ICJhNzdkNmQ2YmFjOWE0NzY3YTFhNzU1NjYxOTllYmY5MiIsCiAgInByb2ZpbGVOYW1lIiA6ICIwOEJFRDUiLAogICJzaWduYXR1cmVSZXF1aXJlZCIgOiB0cnVlLAogICJ0ZXh0dXJlcyIgOiB7CiAgICAiU0tJTiIgOiB7CiAgICAgICJ1cmwiIDogImh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvOWI1Njg5NWI5NjU5ODk2YWQ2NDdmNTg1OTkyMzhhZjUzMmQ0NmRiOWMxYjAzODliOGJiZWI3MDk5OWRhYjMzZCIsCiAgICAgICJtZXRhZGF0YSIgOiB7CiAgICAgICAgIm1vZGVsIiA6ICJzbGltIgogICAgICB9CiAgICB9CiAgfQp9",
            "aNIhT2Tj20v1lONBOK3fIwBqJwWnjErq20h663Gb+PVmR9Iweh1h2ZEJ2pwDDnM4Af1XFDA5hS1Z9yOc8EdVTKyyi1yj9EIvMwQz/Q4N2sBsjWGZtCe8/Zy+X82iv0APB4cumE2gkgDbPjxCFNbpVKmV3U1WzwY/GKOMHofhWS1ULedQ1TszuMmDuHPLEzWaXigZ+xt5zChXvE8QoLTfBvgb8wtqVpyxAKf/o8xQduKiNE7t+de1CwOhLqbVTGh7DU0vLC5stDuqN+nC9dS7c2CG0ori6gFoGMvP4oIss6zm1nb0laMrZidJTgmuXk2Pv4NGDBXdYcAzhfWcSWGsBVMWrJfccgFheG+YcGYaYj6V2nBp0YTqqhN4wDt3ltyTNEMOr/JKyBTLzq/F7IL6rrdyMw+MbAgCa1FhfXxtzdQE2KsL55pbr2DZ8J4DYf+/OC1pWCJ4vvA/A1qGHyi3Zwtj9lCl1Jq5Qm2P9BgWxpk0ikJefRPMg4qWOEcYnjqwXuEp+IgTJi1xr+j/+g28aS1TsF8ijaJjSbEN4urrf3RYL+PZBcggzX9VaPB0NPdioOXznIotY+S6ZW7FnSh6UnrGAKadQBVLey5zmVWMfXlBUq9JMh0csuNd4dDQCLNK8oGORhMgksOMHhVaBie4otUgJ7ThR/WPjOAKiG2TNU0=",
        ),
        (
            NPCBehaviour {
                default_yaw: yaw,
                default_pitch: 0.0,
            },
            Interactable::<Dungeon> {
                callback: |_, player| {
                    if let DungeonState::Started { .. } = player.world().state {
                        return;
                    }
                    player.open_container(OpenContainer::Menu(Box::new(MortMenu {})))
                }
            },
            JumpBehaviour {}
        ),
    );
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let text = ChatComponent::new("RustClear").color(MCColors::Gold)
        .append(ChatComponent::new(" version ").color(MCColors::Gray))
        .append(ChatComponent::new(env!("CARGO_PKG_VERSION")).color(MCColors::Green));

    let status = Status::new(0, 1, text, "");
    let (tx, mut rx) = start_network("127.0.0.1:4972", status);

    let mut world = initialize_world(tx)?;
    spawn_mort(&mut world);

    world.entities.register_behaviour::<NPCBehaviour>();
    world.entities.register_behaviour::<JumpBehaviour>();
    world.entities.register_behaviour::<DoorBehaviour>();

    // for x in -200..0 {
    //     for z in -200..0 {
    //         world.chunk_grid.set_block_at(Block::Stone, x, 68, z)
    //     }
    // }
    //
    // for x in -200..0 {
    //     for z in -200..0 {
    //         world.spawn_entity(
    //             dvec3(x as f64 + 0.5, 69.0, z as f64 + 0.5),
    //             0.0,
    //             0.0,
    //             MobAppearance {
    //                 variant: EntityVariant::Zombie,
    //                 metadata: EntityMetadata::Zombie(ZombieMetadata {
    //                     is_baby: false,
    //                     is_villager: false,
    //                 })
    //             },
    //             (
    //                 NPCBehaviour {
    //                     default_yaw: 0.0,
    //                     default_pitch: 0.0,
    //                 },
    //                 Interactable::<Dungeon> {
    //                     callback: |_, player| {
    //                         if let DungeonState::Started { .. } = player.world().state {
    //                             return;
    //                         }
    //                         player.open_container(OpenContainer::Menu(Box::new(MortMenu {})))
    //                     }
    //                 },
    //                 JumpBehaviour {}
    //             ),
    //         );
    //     }
    // }

    // println!("{}", world.entities.next_entity_id());

    let mut tick_interval = tokio::time::interval(Duration::from_millis(50));
    loop {
        tick_interval.tick().await;
        // let start = std::time::Instant::now();

        loop {
            match rx.try_recv() {
                Ok(message) => world.process_event(message),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => bail!("Network thread dropped its reciever."),
            }
        }

        world.tick();
        // println!("elapsed {:?}", start.elapsed())
    }
}

fn room_data() -> DeterministicHashMap<usize, RoomData> {
    let rooms_directory = include_dir!("DungeonData/room_data/");
    let room_data_storage: DeterministicHashMap<usize, RoomData> = rooms_directory.entries()
        .iter()
        .map(|file| {
            let file = file.as_file().unwrap();

            let contents = file.contents_utf8().unwrap();
            let name = file.path().file_name().unwrap().to_str().unwrap();
            let room_data = RoomData::from_raw_json(contents);

            let name_parts: Vec<&str> = name.split(",").collect();
            let room_id = name_parts.first().unwrap().parse::<usize>().unwrap();

            (room_id, room_data)
        }).collect();
    room_data_storage
}

fn door_block_data() -> DeterministicHashMap<DoorType, Vec<Vec<Block>>> {
    let door_data: Vec<Vec<Block>> = include_str!("../DungeonData/door_data/doors.txt")
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

    DeterministicHashMap::from_iter(
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
    )
}