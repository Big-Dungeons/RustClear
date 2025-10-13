use crate::assets::{get_assets, load_assets};
use crate::dungeon::dungeon::{Dungeon, DungeonState};
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::menus::MortMenu;
use anyhow::bail;
use glam::ivec3;
use rand::prelude::IndexedRandom;
use rand::rng;
use server::block::rotatable::Rotatable;
use server::entity::entity::{EntityBase, EntityImpl};
use server::entity::entity_metadata::{EntityMetadata, EntityVariant};
use server::inventory::menu::OpenContainer;
use server::network::network::start_network;
use server::network::packets::packet_buffer::PacketBuffer;
use server::network::protocol::play::serverbound::EntityInteractionType;
use server::player::player::Player;
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

    {
        struct Test {
            yaw: f32,
            pitch: f32,
        }

        impl EntityImpl<Dungeon> for Test {
            fn spawn(&mut self, _: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {}
            fn despawn(&mut self, _: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {}
            fn tick(&self, entity: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {
                if entity.ticks_existed % 5 == 0 {
                    return;
                }

                let world = entity.world();
                let player: Option<&Player<DungeonPlayer>> = world
                    .players
                    .iter()
                    .filter(|p| entity.position.distance(p.position) <= 5.0)
                    .min_by(|a, b| {
                        entity
                            .position
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
                action: &EntityInteractionType,
            ) {
                if let EntityInteractionType::InteractAt = action {
                    return;
                }
                if let DungeonState::Started { .. } = player.world().state {
                    return;
                }
                player.open_container(OpenContainer::Menu(Box::new(MortMenu {})))
            }
        }

        let entrance = world.extension.entrance_room();
        let entrance = entrance.borrow();
        let mut position = entrance
            .get_world_block_position(ivec3(15, 69, 4))
            .as_dvec3();
        position.x += 0.5;
        position.z += 0.5;

        let yaw = 0.0.rotate(entrance.rotation);
        world.spawn_entity(
            Some(EntityMetadata::new(EntityVariant::NPC { npc_id: "mort" })),
            position,
            yaw,
            0.0,
            Test { yaw, pitch: 0.0 },
        );
    }

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
