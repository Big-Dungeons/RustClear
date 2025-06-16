mod net;
mod server;
mod dungeon;

use crate::dungeon::room_data::RoomData;
use crate::dungeon::Dungeon;
use crate::net::client_event::ClientEvent;
use crate::net::network_message::NetworkMessage;
use crate::net::packets::client_bound::confirm_transaction::ConfirmTransaction;
use crate::net::packets::client_bound::entity::entity_effect::{EntityEffect, HASTEID};
use crate::net::packets::client_bound::particles::Particles;
use crate::net::packets::packet::SendPacket;
use crate::net::run_network::run_network_thread;
use crate::server::block::block_pos::BlockPos;
use crate::server::entity::ai::pathfinding::pathfinder::Pathfinder;
use crate::server::entity::entity::Entity;
use crate::server::entity::entity_type::EntityType;
use crate::server::server::Server;
use crate::server::utils::chat_component::chat_component_text::ChatComponentTextBuilder;
use crate::server::utils::direction::Direction;
use crate::server::utils::particles::ParticleTypes;
use crate::server::utils::vec3f::Vec3f;
use crate::dungeon::crushers::Crusher;
use anyhow::Result;
use include_dir::include_dir;
use rand::seq::IndexedRandom;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel;

const STATUS_RESPONSE_JSON: &str = r#"{
    "version": { "name": "1.8.9", "protocol": 47 },
    "players": { "max": 1, "online": 0 },
    "description": { "text": "RustClear", "color": "gold", "extra": [{ "text": " version ", "color": "gray" }, { "text": "0.1.0", "color": "green"}] }
}"#;

#[tokio::main]
async fn main() -> Result<()> {
    let (network_tx, network_rx) = unbounded_channel::<NetworkMessage>();
    let (event_tx, mut event_rx) = unbounded_channel::<ClientEvent>();


    let mut server = Server::initialize(network_tx);
    server.world.server = &mut server;

    let mut tick_interval = tokio::time::interval(Duration::from_millis(50));
    tokio::spawn(
        run_network_thread(
            network_rx,
            server.network_tx.clone(),
            event_tx.clone(),
        )
    );

    let spawn_pos = Vec3f {
        x: 25.0,
        y: 69.0,
        z: 25.0,
    };

    let rooms_dir = include_dir!("src/room_data/");

    let room_data_storage: HashMap<usize, RoomData> = rooms_dir.entries()
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

    let dungeon_strings = include_str!("dungeon_storage/dungeons.txt")
        .split("\n")
        .collect::<Vec<&str>>();

    let mut rng = rand::rng();
    let dungeon_str = dungeon_strings.choose(&mut rng).unwrap();
    // let dungeon_str = "080909090900080310021104081010121304081415121600041718180100171705190600999999291999901099991999990999919009190001999993999009999909";
    println!("Dungeon String: {}", dungeon_str);

    let dungeon = Dungeon::from_string(dungeon_str, &room_data_storage);
    // let doors = vec![Door { x: 0, z: 0, direction: Axis::X, door_type: DoorType::NORMAL}];
    // let dungeon = Dungeon::with_rooms_and_doors(
    //     vec![
    //         Room::new(
    //             vec![(2, 2), (3, 2), (3, 1)],
    //             &doors,
    //             get_random_data_with_type(RoomType::Normal, RoomShape::L, &room_data_storage)
    //         ),
    //         Room::new(
    //             vec![(0, 0), (1, 0), (2, 0), (3, 0)],
    //             &doors,
    //             get_random_data_with_type(RoomType::Normal, RoomShape::OneByFour, &room_data_storage)
    //         ),
    //         Room::new(
    //             vec![(1, 1), (2, 1)],
    //             &doors,
    //             get_random_data_with_type(RoomType::Normal, RoomShape::OneByTwo, &room_data_storage)
    //         ),
    //         Room::new(
    //             vec![(4, 2)],
    //             &doors,
    //             get_random_data_with_type(RoomType::Normal, RoomShape::OneByOne, &room_data_storage)
    //         ),
    //     ], doors);

    for room in &dungeon.rooms {
        // println!("Room: {:?} type={:?} rotation={:?} shape={:?} corner={:?}", room.segments, room.room_data.room_type, room.rotation, room.room_data.shape, room.get_corner_pos());
        room.load_into_world(&mut server.world);
    }

    for door in &dungeon.doors {
        dungeon.load_door(door, &mut server.world);
    }

    let mut crusher = Crusher::new(
        BlockPos {
            x: 30,
            y: 69,
            z: 20,
        },
        Direction::North,
        5,
        5,
        10,
        10,
        20,
    );

    // let room_to_load = include_str!("room_data/462,-312");

    // let room_data = RoomData::from_raw_json(room_to_load);

    // let room = Room::new(vec![(0, 0)], room_data);

    // room.load_into_world(&mut server.world);

    // println!("Room Data: {:?}", room_data);

    let zombie = Entity::create_at(EntityType::Zombie, spawn_pos, server.world.new_entity_id());
    let path = Pathfinder::find_path(&zombie, &BlockPos { x: 10, y: 69, z: 10 }, &server.world)?;

    server.world.entities.insert(zombie.entity_id, zombie);
    let text = ChatComponentTextBuilder::new("Hello World!").build();
    server.world.player_info.update_text(1, text);

    loop {
        tick_interval.tick().await;

        while let Ok(message) = event_rx.try_recv() {
            server.process_event(message).unwrap_or_else(|err| eprintln!("Error processing event: {err}"));
        }

        for entity_id in server.world.entities.keys().cloned().collect::<Vec<_>>() {
            if let Some(mut entity) = server.world.entities.remove(&entity_id) {
                entity.ticks_existed += 1;
                // this may at some point be abused to prevent getting an entities own self if it iterates over world entities so be careful if you change this
                let returned = entity.update(&mut server.world, &server.network_tx);
                server.world.entities.insert(entity_id, returned);
            }
        }

        // this needs to be changed to work with loaded chunks, tracking last sent data per player (maybe), etc.
        // also needs to actually be in a vanilla adjacent way.
        for player in server.players.values_mut() {
            // println!("player ticked: {player:?}");
            ConfirmTransaction::new().send_packet(player.client_id, &server.network_tx)?; // should stop disconnects? keep alive logic would too probably.
            // for entity in player.tracked_entities.iter() {
            //     if let Some(entity) = server.world.entities.get_mut(entity) {
            //         EntityLookMove::from_entity(entity).send_packet(player.client_id, &server.network_tx)?;
            //         EntityHeadLook::new(entity.entity_id, entity.head_yaw).send_packet(player.client_id, &server.network_tx)?;
            //     }
            // }

            let room = dungeon.get_player_room(player);

            if player.scoreboard.header_dirty {
                player.scoreboard.header_packet().send_packet(player.client_id, &server.network_tx)?;
            }

            // maybe another value if any lines are updated? this will just not pull any packets if nothing is updated but it will still iterate...
            for packet in player.scoreboard.get_packets() {
                packet.send_packet(player.client_id, &server.network_tx)?;
            }

            if !player.scoreboard.displaying {
                player.scoreboard.display_packet().send_packet(player.client_id, &server.network_tx)?;
            }

            if let Some(player_entity) = server.world.entities.get(&player.entity_id) {
                if player_entity.ticks_existed % 20 == 0 {
                    let seconds = player_entity.ticks_existed / 20;
                    player.scoreboard.update_line("etime", format!("Time Elapsed: §a§a{seconds}s")); // this isnt accurate to hypixel atm but its ok!
                }

                if player_entity.ticks_existed % 150 == 0 {
                    //player.scoreboard.add_line_at(0, "resize", "amazing");

                    // player.scoreboard.update_header("NEW HEADER WOWOWOW");
                }

                if player_entity.ticks_existed % 250 == 0 {
                    player.scoreboard.remove_line("etime");

                    // player.scoreboard.update_header("old header :(");
                }

                if player_entity.ticks_existed % 5 == 0 {
                    let mut current_index = 1;
                    for pos in path.iter() {
                        let particle = Particles::new(
                            ParticleTypes::Crit,
                            Vec3f::from(pos),
                            Vec3f::new(0.1, 0.1, 0.1),
                            0.0,
                            current_index,
                            true,
                            None,
                        );
                        current_index += 1;

                        particle?.send_packet(player.client_id, &server.network_tx)?;
                    }
                }

                if player_entity.ticks_existed % 60 == 0 {
                    EntityEffect {
                        entity_id: player.entity_id,
                        effect_id: HASTEID,
                        amplifier: 2,
                        duration: 200,
                        hide_particles: true,
                    }.send_packet(player.client_id, &server.network_tx)?;

                    // EntityEffect {
                    //     entity_id: player.entity_id,
                    //     effect_id: NIGHTVISIONID,
                    //     amplifier: 0,
                    //     duration: 400,
                    //     hide_particles: true,
                    // }.send_packet(player.client_id, &server.network_tx)?;
                }
            }
        }

        // if  {  }

        crusher.tick(&mut server);
    }
}