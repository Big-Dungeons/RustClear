use crate::block::blocks::Blocks;
use crate::block::rotatable::Rotatable;
use crate::dungeon::door::door::DoorType;
use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::room::room_data::RoomData;
use crate::entity::entity::{EntityBase, EntityImpl};
use crate::entity::entity_metadata::{EntityMetadata, EntityVariant};
use crate::inventory::menu::{DungeonMenu, OpenContainer};
use crate::network::internal_packets::{MainThreadMessage, NetworkThreadMessage};
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::protocol::play::serverbound::EntityInteractionType;
use crate::network::run_network::run_network_thread;
use crate::player::player::Player;
use crate::types::block_position::BlockPos;
use crate::utils::hasher::deterministic_hasher::DeterministicHashMap;
use crate::utils::seeded_rng::{seeded_rng, SeededRng};
use crate::world::world::{World, WorldExtension};
use anyhow::bail;
use glam::DVec3;
use include_dir::include_dir;
use rand::prelude::IndexedRandom;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::unbounded_channel;

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

    let dungeon_strings = include_str!("dungeon_storage/dungeons.txt")
        .split("\n")
        .collect::<Vec<&str>>();


    let rooms_dir = include_dir!("src/room_data/");

    // roomdata first digit (the key) is just a list of numbers 0..etc. this could just be a vec with roomid lookups.
    let room_data_storage: DeterministicHashMap<usize, RoomData> = rooms_dir
        .entries()
        .iter()
        .map(|file| {
            let file = file.as_file().unwrap();

            let contents = file.contents_utf8().unwrap();
            let name = file.path().file_name().unwrap().to_str().unwrap();
            let room_data = RoomData::from_raw_json(contents);

            let name_parts: Vec<&str> = name.split(",").collect();
            let room_id = name_parts.first().unwrap().parse::<usize>().unwrap();

            (room_id, room_data)
        })
        .collect();

    let dungeon = Dungeon::from_string(
        dungeon_strings.choose(&mut seeded_rng()).unwrap(),
        &room_data_storage
    )?;

    let mut world = World::new(
        network_tx,
        dungeon,
    );

    for room in world.extension.rooms.iter() {
        room.load_into_world(&mut world.chunk_grid);
    }
    load_doors_into_world(&mut world);

    {
        struct Test;
        impl EntityImpl<Dungeon> for Test {
            fn spawn(&mut self, _: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {
            }
            fn despawn(&mut self, _: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {
            }
            fn tick(&self, entity: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {
                // entity.position.y += 0.1;
                // y no work?
                entity.pitch += 5.0
            }
            fn interact(
                &self, _: &mut EntityBase<Dungeon>,
                player: &mut Player<DungeonPlayer>,
                action: &EntityInteractionType
            ) {
                if let EntityInteractionType::InteractAt = action {
                    return;
                }
                player.open_container(OpenContainer::Menu(Box::new(DungeonMenu::Mort)))
            }
        }

        let entrance = world.extension.entrance_room();
        let position = entrance.get_world_block_pos(&BlockPos::new(15, 69, 4)).as_dvec3_centered();
        let yaw = 0.0.rotate(entrance.rotation);
        world.spawn_entity(
            EntityMetadata::new(EntityVariant::Zombie { is_child: false, is_villager: false }),
            position,
            yaw,
            0.0,
            Test {}
        );
    }
    
    loop {
        tick_interval.tick().await;
        
        loop {
            match main_rx.try_recv() {
                Ok(message) => world.process_event(message),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => bail!("Network thread dropped its reciever.")
            }
        }

        world.tick();
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
    let door_data: Vec<Vec<Blocks>> = include_str!("door_data/doors.txt")
        .split("\n")
        .map(|line| {
            let mut blocks: Vec<Blocks> = Vec::new();

            for i in (0..line.len() - 1).step_by(4) {
                let substr = line.get(i..i + 4).unwrap();
                let state = u16::from_str_radix(substr, 16).unwrap();

                blocks.push(Blocks::from(state));
            }

            blocks
        })
        .collect();

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
        door.load_into_world(&mut world.chunk_grid, &door_type_blocks)
    }
}