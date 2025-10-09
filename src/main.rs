use crate::assets::{get_assets, load_assets};
use crate::block::blocks::Blocks;
use crate::block::rotatable::Rotatable;
use crate::dungeon::door::door::DoorType;
use crate::dungeon::dungeon::{Dungeon, DungeonState};
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::entity::entity::{EntityBase, EntityImpl};
use crate::entity::entity_metadata::{EntityMetadata, EntityVariant};
use crate::inventory::menu::{DungeonMenu, OpenContainer};
use crate::network::internal_packets::{MainThreadMessage, NetworkThreadMessage};
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::protocol::play::serverbound::EntityInteractionType;
use crate::network::run_network::run_network_thread;
use crate::player::player::Player;
use crate::utils::seeded_rng::{seeded_rng, SeededRng};
use crate::world::world::World;
use anyhow::bail;
use glam::{ivec3, DVec3};
use rand::prelude::IndexedRandom;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::unbounded_channel;

mod assets;
mod world;
mod player;
mod dungeon;
mod network;
mod utils;
mod types;
mod entity;
mod block;
mod inventory;
mod constants;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // todo: either a config file with repo/path or command line args.
    load_assets("assets", "https://github.com/Big-Dungeons/ClearData/archive/refs/heads/main.zip").await?;
    let (network_tx, network_rx) = unbounded_channel::<NetworkThreadMessage>();
    let (main_tx, mut main_rx) = unbounded_channel::<MainThreadMessage>();

    let mut tick_interval = tokio::time::interval(Duration::from_millis(50));
    tokio::spawn(run_network_thread(
        network_rx,
        network_tx.clone(),
        main_tx,
    ));

    let rng_seed: u64 = rand::random();
    SeededRng::set_seed(rng_seed);

    let dungeon_strings = &get_assets().dungeon_seeds;
    let room_data_storage = &get_assets().room_data;

    let dungeon = Dungeon::from_string(
        dungeon_strings.choose(&mut seeded_rng()).unwrap(),
        &room_data_storage
    )?;

    let mut world = World::new(
        network_tx,
        dungeon,
    );

    for room in world.extension.rooms.iter() {
        room.borrow().load_into_world(&mut world.chunk_grid);
    }
    load_doors_into_world(&mut world);

    {
        struct Test {
            yaw: f32,
            pitch: f32,
        }

        impl EntityImpl<Dungeon> for Test {
            fn spawn(&mut self, _: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {
            }
            fn despawn(&mut self, _: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {
            }
            fn tick(&self, entity: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {
                if entity.ticks_existed % 5 == 0 {
                    return;
                }

                let world = entity.world();
                let player: Option<&Player<DungeonPlayer>> = world.players
                    .iter()
                    .filter(|p| entity.position.distance(p.position) <= 5.0)
                    .min_by(|a, b| {
                        entity.position
                            .distance(a.position)
                            .partial_cmp(&entity.position.distance(b.position))
                            .unwrap()
                    });
                
                if let Some(player) = player {
                    let (yaw, pitch) = {
                        let direction = player.position - entity.position;
                        let yaw = direction.z.atan2(direction.x).to_degrees() - 90.0;
                        let horizontal_dist = (direction.x.powi(2) + direction.z.powi(2)).sqrt();
                        let pitch = -direction.y.atan2(horizontal_dist).to_degrees();

                        (yaw, pitch)
                    };
                    entity.yaw = yaw as f32;
                    entity.pitch = pitch as f32;
                } else {
                    entity.yaw = self.yaw;
                    entity.pitch = self.pitch;
                }
            }
            fn interact(
                &self, 
                _: &mut EntityBase<Dungeon>,
                player: &mut Player<DungeonPlayer>,
                action: &EntityInteractionType
            ) {
                if let EntityInteractionType::InteractAt = action {
                    return;
                }
                if let DungeonState::Started { .. } = player.world().state {
                    return;
                }
                player.open_container(OpenContainer::Menu(Box::new(DungeonMenu::Mort)))
            }
        }

        let entrance = world.extension.entrance_room();
        let entrance = entrance.borrow();
        let mut position = entrance.get_world_block_position(ivec3(15, 69, 4)).as_dvec3();
        position.x += 0.5;
        position.z += 0.5;
        
        let yaw = 0.0.rotate(entrance.rotation);
        world.spawn_entity(
            Some(EntityMetadata::new(EntityVariant::NPC { npc_id: "mort" })),
            position,
            yaw,
            0.0,
            Test { yaw, pitch: 0.0 }
        );
    }
    
    loop {
        tick_interval.tick().await;
        // let start = Instant::now();
        
        loop {
            match main_rx.try_recv() {
                Ok(message) => world.process_event(message),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => bail!("Network thread dropped its reciever.")
            }
        }

        world.tick();
        // println!("elapsed {:?}", start.elapsed())
    }
}

pub fn get_chunk_position(position: DVec3) -> (i32, i32) {
    let x = (position.x.floor() as i32) >> 4;
    let z = (position.z.floor() as i32) >> 4;
    (x, z)
}

fn load_doors_into_world(world: &mut World<Dungeon>) {
    
    // Might be a good idea to make a new format for storing doors so that indexes etc don't need to be hard coded.
    // But this works for now...
    let door_data: &Vec<Vec<Blocks>> = &get_assets().door_data;

    let door_type_blocks: HashMap<DoorType, Vec<Vec<Blocks>>> = HashMap::from_iter(
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
        ]
            .into_iter(),
    );
    
    for door in world.extension.doors.iter() {
        door.borrow().load_into_world(&mut world.chunk_grid, &door_type_blocks)
    }
}