mod net;
mod server;
mod dungeon;
mod utils;

use crate::dungeon::door::DoorType;
use crate::dungeon::dungeon_state::DungeonState;
use crate::dungeon::room_data::RoomData;
use crate::dungeon::Dungeon;
use crate::net::internal_packets::{MainThreadMessage, NetworkThreadMessage};
use crate::net::packets::client_bound::confirm_transaction::ConfirmTransaction;
use crate::net::packets::client_bound::entity::entity_effect::{Effects, EntityEffect};
use crate::net::packets::packet::SendPacket;
use crate::net::run_network::run_network_thread;
use crate::server::block::blocks::Blocks;
use crate::server::player::scoreboard::ScoreboardLines;
use crate::server::server::Server;
use crate::server::utils::chat_component::chat_component_text::ChatComponentTextBuilder;
use crate::server::utils::color::MCColors;
use crate::server::utils::dvec3::DVec3;
use anyhow::Result;
use include_dir::include_dir;
use indoc::formatdoc;
use rand::seq::IndexedRandom;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel;

const STATUS_RESPONSE_JSON: &str = r#"{
    "version": { "name": "1.8.9", "protocol": 47 },
    "players": { "max": 1, "online": 0 },
    "description": { "text": "RustClear", "color": "gold", "extra": [{ "text": " version ", "color": "gray" }, { "text": "0.1.0", "color": "green"}] }
}"#;

#[tokio::main]
async fn main() -> Result<()> {
    let (network_tx, network_rx) = unbounded_channel::<NetworkThreadMessage>();
    let (main_tx, mut main_rx) = unbounded_channel::<MainThreadMessage>();

    let args: Vec<String> = env::args().collect();

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

    // Might be a good idea to make a new format for storing doors so that indexes etc don't need to be hard coded.
    // But this works for now...
    let door_data: Vec<Vec<Blocks>> = include_str!("door_data/doors.txt").split("\n").map(|line| {
        let mut blocks: Vec<Blocks> = Vec::new();

        for i in (0..line.len()-1).step_by(4) {
            let substr = line.get(i..i+4).unwrap();
            let state = u16::from_str_radix(substr, 16).unwrap();

            blocks.push(Blocks::from(state));
        }

        blocks
    }).collect();

    let door_type_blocks: HashMap<DoorType, Vec<Vec<Blocks>>> = HashMap::from_iter(vec![
        (DoorType::BLOOD, vec![
            door_data[0].clone(),
        ]),
        (DoorType::ENTRANCE, vec![
            door_data[1].clone(),
        ]),
        (DoorType::NORMAL, vec![
            door_data[1].clone(),
            door_data[2].clone(),
            door_data[3].clone(),
            door_data[4].clone(),
            door_data[5].clone(),
            door_data[6].clone(),
            door_data[7].clone(),
        ]),
    ].into_iter());

    let dungeon_strings = include_str!("dungeon_storage/dungeons.txt")
        .split("\n")
        .collect::<Vec<&str>>();

    let dungeon_str = match args.len() {
        0..=1 => {
            let mut rng = rand::rng();
            dungeon_strings.choose(&mut rng).unwrap()
        },
        _ => args.get(1).unwrap().as_str()
    };

    println!("Dungeon String: {}", dungeon_str);

    let dungeon = Dungeon::from_string(dungeon_str, &room_data_storage)?;
    let mut server = Server::initialize_with_dungeon(network_tx, dungeon);
    server.world.server = &mut server;
    server.dungeon.server = &mut server;

    let mut tick_interval = tokio::time::interval(Duration::from_millis(50));
    tokio::spawn(
        run_network_thread(
            network_rx,
            server.network_tx.clone(),
            main_tx,
        )
    );

    let dungeon = &server.dungeon;
    
    for room in &dungeon.rooms {
        // println!("Room: {:?} type={:?} rotation={:?} shape={:?} corner={:?}", room.segments, room.room_data.room_type, room.rotation, room.room_data.shape, room.get_corner_pos());
        room.load_into_world(&mut server.world);
    }

    for door in &dungeon.doors {
        door.load_into_world(&mut server.world, &door_type_blocks);
    }

    let zombie_spawn_pos = DVec3 {
        x: 25.0,
        y: 69.0,
        z: 25.0,
    };
    
    // let zombie = Entity::create_at(EntityType::Zombie, zombie_spawn_pos, server.world.new_entity_id());
    // let path = Pathfinder::find_path(&zombie, &BlockPos { x: 10, y: 69, z: 10 }, &server.world)?;

    // server.world.entities.insert(zombie.entity_id, zombie);

    let cata_line =
        ChatComponentTextBuilder::new("")
            .append(ChatComponentTextBuilder::new("Dungeon: ").color(MCColors::Aqua).bold().build())
            .append(ChatComponentTextBuilder::new("Catacombs").color(MCColors::Gray).build())
            .build();

    server.world.player_info.set_line(0, cata_line);
    
    
    loop {
        tick_interval.tick().await;

        while let Ok(message) = main_rx.try_recv() {
            server.process_event(message).unwrap_or_else(|err| eprintln!("Error processing event: {err}"));
        }

        server.dungeon.tick()?;
        server.world.tick()?;
        
        // for entity_id in server.world.entities.keys().cloned().collect::<Vec<_>>() {
        //     if let Some(mut entity) = server.world.entities.remove(&entity_id) {
        //         entity.ticks_existed += 1;
        //         // this may at some point be abused to prevent getting an entities own self if it iterates over world entities so be careful if you change this
        //         let returned = entity.update(&mut server.world, &server.network_tx);
        //         server.world.entities.insert(entity_id, returned);
        //     }
        // }

        let tab_list_packet = server.world.player_info.get_packet();

        // this needs to be changed to work with loaded chunks, tracking last sent data per player (maybe), etc.
        // also needs to actually be in a vanilla adjacent way.
        for player in server.world.players.values_mut() {
            // println!("player ticked: {player:?}");
            player.ticks_existed += 1;
            ConfirmTransaction::new().send_packet(player.client_id, &server.network_tx)?; // should stop disconnects? keep alive logic would too probably.
            
            // for entity in player.tracked_entities.iter() {
            //     if let Some(entity) = server.world.entities.get_mut(entity) {
            //         EntityLookMove::from_entity(entity).send_packet(player.client_id, &server.network_tx)?;
            //         EntityHeadLook::new(entity.entity_id, entity.head_yaw).send_packet(player.client_id, &server.network_tx)?;
            //     }
            // }

            let mut sidebar_lines = ScoreboardLines(Vec::new());

            // TODO: correctly handle date based on clock, handle room id according to current room
            sidebar_lines.push(formatdoc! {r#"
                §e§lSKYBLOCK
                §7{date} §8m24§87W {room_id}

                Winter 22nd
                §73:10pm
                 §7⏣ §cThe Catacombs §7(F7)

            "#,
            date = "06/14/25",
            room_id = "730,-420",
            });

            match server.dungeon.state {
                DungeonState::NotReady => {
                    for (_, p) in &player.server_mut().world.players {
                        sidebar_lines.push(format!("§c[M] §7{}", p.profile.username))
                    }
                    sidebar_lines.new_line();
                }
                DungeonState::Starting { tick_countdown } => {
                    for (_, p) in &player.server_mut().world.players {
                        sidebar_lines.push(format!("§a[M] §7{}", p.profile.username))
                    }
                    sidebar_lines.new_line();
                    sidebar_lines.push(format!("Starting in: §a0§a:0{}", (tick_countdown / 20) + 1));
                    sidebar_lines.new_line();
                }
                DungeonState::Started { current_ticks } => {
                    // this is scuffed but it works
                    let seconds = current_ticks / 20;
                    let time = if seconds >= 60 {
                        let minutes = seconds / 60;
                        let seconds = seconds % 60;
                        format!("{}{}m{}{}s", if minutes < 10 { "0" } else { "" }, minutes, if seconds < 10 { "0" } else { "" }, seconds)
                    } else {
                        let seconds = seconds % 60;
                        format!("{}{}s", if seconds < 10 { "0" } else { "" }, seconds)
                    };
                    // TODO: display correct keys, and cleared percentage
                    // clear percentage is based on amount of tiles that are cleared.
                    sidebar_lines.push(formatdoc! {r#"
                        Keys: §c■ §c✖ §8§8■ §a0x
                        Time elapsed: §a§a{time}
                        Cleared: §c{clear_percent}% §8§8({score})

                        §3§lSolo

                    "#,
                    clear_percent = "0",
                    score = "0",
                    });
                }
                DungeonState::Finished => {}
            }

            if let Some(tab_list) = &tab_list_packet {
                tab_list.clone().send_packet(player.client_id, &server.network_tx)?;
            }

            sidebar_lines.push_str("§emc.hypixel.net");

            for packet in player.sidebar.update(sidebar_lines) {
                packet.send_packet(player.client_id, &server.network_tx)?;
            }
            
            // if player.ticks_existed % 5 == 0 {
            //     let mut current_index = 1;
            //     for pos in path.iter() {
            //         let particle = Particles::new(
            //             ParticleTypes::Crit,
            //             DVec3::from(pos),
            //             DVec3::new(0.1, 0.1, 0.1),
            //             0.0,
            //             current_index,
            //             true,
            //             None,
            //         );
            //         current_index += 1;
            // 
            //         particle?.send_packet(player.client_id, &server.network_tx)?;
            //     }
            // }
            if player.ticks_existed % 60 == 0 {
                player.send_packet(EntityEffect {
                    entity_id: player.entity_id,
                    effect: Effects::Haste,
                    amplifier: 2,
                    duration: 200,
                    hide_particles: true,
                })?;
                player.send_packet(EntityEffect {
                    entity_id: player.entity_id,
                    effect: Effects::NightVision,
                    amplifier: 0,
                    duration: 400,
                    hide_particles: true,
                })?;
            }
        }
    }
}