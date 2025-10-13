use crate::assets::{get_assets, load_assets};
use crate::dungeon::dungeon::{Dungeon, DungeonState};
use crate::dungeon::entities::npc::InteractableNPC;
use crate::dungeon::menus::MortMenu;
use anyhow::bail;
use glam::ivec3;
use rand::prelude::IndexedRandom;
use rand::rng;
use server::block::rotatable::Rotatable;
use server::entity::entity_metadata::{EntityMetadata, EntityVariant};
use server::inventory::menu::OpenContainer;
use server::network::network::start_network;
use server::utils::seeded_rng::SeededRng;
use server::world::world::World;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;

mod assets;
mod dungeon;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // todo: either a config file with repo/path or command line args.
    load_assets(
        "assets",
        "https://github.com/Big-Dungeons/ClearData/archive/refs/heads/main.zip",
    )
    .await?;
    // ^^^ for rooms/doors this is really pointless, because there is no reason to customize them, especially once finalised
    // I can understand it for favicon tho


    let (tx, mut rx) = start_network("127.0.0.1:4972", Arc::new(
        r#"{
            "version": {
                "name": "1.8.9",
                "protocol": 47
            },
            "players": {
                "max": 1,
                "online": 0
            },
            "description": {
                "text": "RustClear",
                "color": "gold",
                "extra": [
                    {
                        "text": " version ",
                        "color": "gray"
                    },
                    {
                        "text": "{version}",
                        "color": "green"
                    }
                ]
            },
            "favicon": "data:image/png;base64,<data>"
        }"#.into()
    ));

    let rng_seed: u64 = rand::random();
    SeededRng::set_seed(rng_seed);

    let dungeon_strings = &get_assets().dungeon_seeds;
    let dungeon_seed = dungeon_strings.choose(&mut rng()).unwrap();

    let room_data_storage = &get_assets().room_data;

    let dungeon = Dungeon::from_string(dungeon_seed, &room_data_storage)?;

    let mut world = World::new(tx, dungeon);

    for room in world.extension.rooms.iter() {
        room.borrow().load_into_world(&mut world.chunk_grid);
    }
    load_doors_into_world(&mut world);


    let entrance = world.extension.entrance_room();
    let entrance = entrance.borrow();
    let mut position = entrance.get_world_block_position(ivec3(15, 69, 4)).as_dvec3();

    position.x += 0.5;
    position.z += 0.5;

    let yaw = 0.0.rotate(entrance.rotation);
    world.spawn_entity(
        Some(EntityMetadata::new(EntityVariant::NPC { npc_id: "mort" })),
        position, yaw, 0.0,
        InteractableNPC { default_yaw: yaw, default_pitch: 0.0, interact_callback: |player| {
            // todo: messages / dialogue
            if let DungeonState::Started { .. } = player.world().state {
                return;
            }
            player.open_container(OpenContainer::Menu(Box::new(MortMenu {})))
        }},
    );


    let mut tick_interval = tokio::time::interval(Duration::from_millis(50));

    loop {
        tick_interval.tick().await;
        // let start = Instant::now();

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

fn load_doors_into_world(world: &mut World<Dungeon>) {
    let door_type_blocks = &get_assets().door_data;

    for door in world.extension.doors.iter() {
        door.borrow().load_into_world(&mut world.chunk_grid, &door_type_blocks)
    }
}
