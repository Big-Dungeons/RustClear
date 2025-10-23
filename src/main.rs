#![allow(clippy::collapsible_if, clippy::too_many_arguments, clippy::new_without_default)]

use crate::assets::{get_assets, load_assets};
use crate::dungeon::dungeon::{Dungeon, DungeonState};
use crate::dungeon::entities::npc::InteractableNPC;
use crate::dungeon::menus::MortMenu;
use anyhow::bail;
use glam::ivec3;
use rand::prelude::IndexedRandom;
use server::block::rotatable::Rotatable;
use server::entity::entity_appearance::PlayerAppearance;
use server::inventory::menu::OpenContainer;
use server::network::internal_packets::NetworkThreadMessage;
use server::network::network::start_network;
use server::types::chat_component::{ChatComponent, MCColors};
use server::types::status::Status;
use server::utils::seeded_rng::{seeded_rng, SeededRng};
use server::world::world::World;
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::UnboundedSender as Sender;

mod assets;
mod dungeon;

pub fn initialize_world(tx: Sender<NetworkThreadMessage>) -> anyhow::Result<World<Dungeon>> {
    let rng_seed: u64 = rand::random();
    SeededRng::set_seed(rng_seed);

    let dungeon_layouts = &get_assets().dungeon_seeds;
    let layout = dungeon_layouts.choose(&mut seeded_rng()).unwrap();

    let room_data_storage = &get_assets().room_data;
    let door_type_blocks = &get_assets().door_data;

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
            "ewogICJ0aW1lc3RhbXAiIDogMTYxODc4MTA4Mzk0NywKICAicHJvZmlsZUlkIiA6ICJhNzdkNmQ2YmFjOWE0NzY3YTFhNzU1NjYxOTllYmY5MiIsCiAgInByb2ZpbGVOYW1lIiA6ICIwOEJFRDUiLAogICJzaWduYXR1cmVSZXF1aXJlZCIgOiB0cnVlLAogICJ0ZXh0dXJlcyIgOiB7CiAgICAiU0tJTiIgOiB7CiAgICAgICJ1cmwiIDogImh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvOWI1Njg5NWI5NjU5ODk2YWQ2NDdmNTg1OTkyMzhhZjUzMmQ0NmRiOWMxYjAzODliOGJiZWI3MDk5OWRhYjMzZCIsCiAgICAgICJtZXRhZGF0YSIgOiB7CiAgICAgICAgIm1vZGVsIiA6ICJzbGltIgogICAgICB9CiAgICB9CiAgfQp9",
            "aNIhT2Tj20v1lONBOK3fIwBqJwWnjErq20h663Gb+PVmR9Iweh1h2ZEJ2pwDDnM4Af1XFDA5hS1Z9yOc8EdVTKyyi1yj9EIvMwQz/Q4N2sBsjWGZtCe8/Zy+X82iv0APB4cumE2gkgDbPjxCFNbpVKmV3U1WzwY/GKOMHofhWS1ULedQ1TszuMmDuHPLEzWaXigZ+xt5zChXvE8QoLTfBvgb8wtqVpyxAKf/o8xQduKiNE7t+de1CwOhLqbVTGh7DU0vLC5stDuqN+nC9dS7c2CG0ori6gFoGMvP4oIss6zm1nb0laMrZidJTgmuXk2Pv4NGDBXdYcAzhfWcSWGsBVMWrJfccgFheG+YcGYaYj6V2nBp0YTqqhN4wDt3ltyTNEMOr/JKyBTLzq/F7IL6rrdyMw+MbAgCa1FhfXxtzdQE2KsL55pbr2DZ8J4DYf+/OC1pWCJ4vvA/A1qGHyi3Zwtj9lCl1Jq5Qm2P9BgWxpk0ikJefRPMg4qWOEcYnjqwXuEp+IgTJi1xr+j/+g28aS1TsF8ijaJjSbEN4urrf3RYL+PZBcggzX9VaPB0NPdioOXznIotY+S6ZW7FnSh6UnrGAKadQBVLey5zmVWMfXlBUq9JMh0csuNd4dDQCLNK8oGORhMgksOMHhVaBie4otUgJ7ThR/WPjOAKiG2TNU0=",
        ),
        InteractableNPC {
            default_yaw: yaw,
            default_pitch: 0.0,
            interact_callback: |player| {
                if let DungeonState::Started { .. } = player.world().state {
                    return;
                }
                player.open_container(OpenContainer::Menu(Box::new(MortMenu {})))
            },
        }
    );
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // todo: either a config file with repo/path or command line args.
    load_assets(
        "assets",
        "https://github.com/Big-Dungeons/ClearData/archive/refs/heads/main.zip",
    ).await?;
    // ^^^ for rooms/doors this is really pointless, because there is no reason to customize them, especially once finalised
    // I can understand it for favicon tho

    let text = ChatComponent::new("RustClear").color(MCColors::Gold)
        .append(ChatComponent::new(" version ").color(MCColors::Gray))
        .append(ChatComponent::new(env!("CARGO_PKG_VERSION")).color(MCColors::Green));

    let status = Status::new(0, 1, text, get_assets().icon_data);
    let (tx, mut rx) = start_network("127.0.0.1:4972", status);

    let mut world = initialize_world(tx)?;
    spawn_mort(&mut world);

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
