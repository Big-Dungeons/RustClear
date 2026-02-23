use crate::dungeon::door::sound_emitter::DoorSoundEmitter;
use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::entities::block_appearance::{BlockAppearance, FallingBlockAppearance};
use crate::dungeon::entities::components::Lifetime;
use crate::dungeon::entities::moving_block_behaviour::MovingBlockBehaviour;
use crate::dungeon::room::room::{Room, RoomStatus};
use crate::dungeon::room::room_implementation::RoomImplementation;
use crate::dungeon::seeded_rng::seeded_rng;
use glam::{dvec3, ivec3, IVec3};
use rand::prelude::IndexedRandom;
use server::block::Block;
use server::constants::Sound;
use server::entity::components::entity_appearance::NoAppearance;
use server::network::protocol::play::clientbound::Effect;
use server::{ClientId, World};
use std::collections::{HashMap, HashSet};

pub struct IceFillPuzzle {
    current_layer: usize,
    layer: Layer,
    obstacles: [HashSet<IVec3>; 3]
}

pub enum Layer {
    Inactive,
    Active {
        ice: HashSet<IVec3>, // used for clearing the area
        obstacles: HashSet<IVec3>,
        remaining_ice_blocks: HashSet<IVec3>,
        tracked_positions: HashMap<ClientId, IVec3>, // last block
    },
    Respawning {
        in_ticks: usize,
    },
}

impl Default for IceFillPuzzle {
    fn default() -> Self {
        let mut rng = seeded_rng();
        let obstacles = OBSTACLES.each_ref().map(|choices| {
            let choice = choices.choose(&mut rng).copied().unwrap_or(&[]);
            choice.iter().copied().collect::<HashSet<_>>()
        });
        Self {
            current_layer: 0,
            layer: Layer::Inactive,
            obstacles,
        }
    }
}

// this is fitting. bad code for a bad puzzle
impl RoomImplementation for IceFillPuzzle {

    fn discover(&mut self, room: &mut Room, world: &mut World<Dungeon>) {
        self.spawn_active_layer(room, world);
    }

    fn tick(&mut self, room: &mut Room, world: &mut World<Dungeon>) {
        if !matches!(room.status, RoomStatus::Discovered) {
            return;
        }
        match &mut self.layer {
            Layer::Active { ice, obstacles, remaining_ice_blocks, tracked_positions } => {

                let mut result: Option<bool> = None;

                for player in room.players() {

                    let position = player.position.floor().as_ivec3() - IVec3::Y;

                    // player isn't on the puzzle.
                    if !ice.contains(&position) {
                        continue
                    }

                    if let Some(last_position) = tracked_positions.get(&player.client_id).copied() {
                        let delta = position - last_position;
                        if delta == IVec3::ZERO {
                            continue;
                        }
                        if delta.abs().element_sum() > 1 {
                            player.send_message("§cDon't move diagonally! Bad!");
                            result = Some(false);
                            break;
                        }
                    }

                    tracked_positions.insert(player.client_id, position);

                    if !remaining_ice_blocks.contains(&position) {
                        player.send_message("§cOops! You stepped on the wrong block!");
                        result = Some(false);
                        break;
                    }

                    let IVec3 { x, y, z } = position;
                    world.chunk_grid.set_block_at(Block::PackedIce, x, y, z);
                    // wool, sound effect + block particle in one packet
                    world.write_local_packet(position.as_dvec3(), &Effect {
                        effect_id: 2001,
                        position,
                        data: 35,
                        disable_relative_volume: false,
                    });
                    remaining_ice_blocks.remove(&position);

                    if remaining_ice_blocks.is_empty() {
                        result = Some(true);
                        break;
                    }
                }

                match result {
                    Some(true) => {
                        for player in room.players() {
                            let pitch = self.current_layer as f32 * 0.2;

                            DungeonPlayer::queue_sound(player, Sound::NoteHarp, 1.0, 1.2 + pitch, 0);
                            DungeonPlayer::queue_sound(player, Sound::NoteHarp, 1.0, 1.3 + pitch, 5);
                            DungeonPlayer::queue_sound(player, Sound::NoteHarp, 1.0, 1.4 + pitch, 10);
                        }
                        self.current_layer += 1;
                        // no more layers
                        if self.current_layer == 3 {
                            self.complete(room, world)
                        } else {
                            self.spawn_active_layer(room, world);
                        }
                    }
                    Some(false) => {
                        for position in ice.iter() {
                            let IVec3 { x, y, z } = *position;
                            world.chunk_grid.set_block_at(Block::Air, x, y, z);

                            world.write_local_packet(position.as_dvec3(), &Effect {
                                effect_id: 2001,
                                position: *position,
                                data: 79,
                                disable_relative_volume: false,
                            });
                        }
                        for position in obstacles.iter() {
                            let IVec3 { x, y, z } = *position;
                            world.chunk_grid.set_block_at(Block::Air, x, y, z);
                            let mut position = position.as_dvec3();
                            position.x += 0.5;
                            position.z += 0.5;

                            world.spawn_entity(
                                position,
                                0.0,
                                0.0,
                                FallingBlockAppearance {
                                    block: Block::PolishedAndesite,
                                },
                                Lifetime {
                                    ticks: 15,
                                }
                            );
                        }
                        self.layer = Layer::Respawning { in_ticks: 60 };
                    }
                    None => {}
                }
            }
            Layer::Respawning { in_ticks } => {
                *in_ticks -= 1;
                if *in_ticks == 0 {
                    self.spawn_active_layer(room, world);
                }
            }
            _ => {}
        }
    }
}

impl IceFillPuzzle {

    // I forgot if this happens on gate open or chest open
    pub fn complete(&mut self, room: &mut Room, world: &mut World<Dungeon>) {
        room.status = RoomStatus::Complete;
        world.map.draw_checkmark(room);

        let mut sound_emitter = room.get_world_block_position(ivec3(14, 74, 27)).as_dvec3();
        sound_emitter.x += 0.5;
        sound_emitter.z += 0.5;

        world.spawn_entity(
            sound_emitter,
            0.0,
            0.0,
            NoAppearance, (
                DoorSoundEmitter {
                    sound: Sound::RandomWoodClick,
                    volume: 2.0,
                    pitch: 0.5,
                },
                Lifetime {
                    ticks: 50
                }
            )
        );

        for x in 13..=17 {
            for y in 74..=79 {
                let block = room.get_world_block_position(ivec3(x, y, 27));
                world.chunk_grid.set_block_at(Block::Barrier, block.x, block.y, block.z);

                world.spawn_entity(
                    block.as_dvec3() + dvec3(0.5, 0.0, 0.5),
                    0.0,
                    0.0,
                    BlockAppearance {
                        block: Block::IronBars,
                    },
                    MovingBlockBehaviour {
                        block,
                        remove_block_in_tick: 20,
                        difference: 0.2,
                    }
                );
            }
        }
    }

    pub fn spawn_active_layer(&mut self, room: &Room, world: &mut World<Dungeon>) {
        let ice = LAYERS[self.current_layer];
        let obstacles = &self.obstacles[self.current_layer];

        for position in ice.iter() {
            let IVec3 { x, y, z } = room.get_world_block_position(*position);
            world.chunk_grid.set_block_at(Block::Ice, x, y, z);
        }
        for position in obstacles.iter() {
            let IVec3 { x, y, z } = room.get_world_block_position(*position);
            world.chunk_grid.set_block_at(Block::PolishedAndesite, x, y, z);
        }

        let mut blocks: HashSet<IVec3> = HashSet::new();
        let mut remaining: HashSet<IVec3> = HashSet::new();

        for block in ice.iter() {
            let position = room.get_world_block_position(*block);
            if !obstacles.contains(&(block + IVec3::Y)) {
                remaining.insert(position);
            }
            blocks.insert(position);
        }

        let obstacles: HashSet<IVec3> = obstacles
            .iter()
            .map(|p| room.get_world_block_position(*p))
            .collect();

        self.layer = Layer::Active {
            ice: blocks,
            obstacles: obstacles.clone(),
            remaining_ice_blocks: remaining,
            tracked_positions: Default::default(),
        };
    }
}

// ice blocks
const LAYERS: [&[IVec3]; 3] = [
    &[
        ivec3(14, 69, 7), ivec3(15, 69, 7), ivec3(15, 69, 8),
        ivec3(14, 69, 8), ivec3(15, 69, 9), ivec3(16, 69, 7),
        ivec3(14, 69, 9), ivec3(16, 69, 8), ivec3(16, 69, 9),
        ivec3(15, 69, 10),
    ],
    &[
        ivec3(13, 70, 12), ivec3(14, 70, 12), ivec3(15, 70, 12), ivec3(16, 70, 12), ivec3(17, 70, 12),
        ivec3(13, 70, 13), ivec3(14, 70, 13), ivec3(15, 70, 13), ivec3(16, 70, 13), ivec3(17, 70, 13),
        ivec3(13, 70, 14), ivec3(14, 70, 14), ivec3(15, 70, 14), ivec3(16, 70, 14), ivec3(17, 70, 14),
        ivec3(13, 70, 15), ivec3(14, 70, 15), ivec3(15, 70, 15), ivec3(16, 70, 15), ivec3(17, 70, 15),
        ivec3(13, 70, 16), ivec3(14, 70, 16), ivec3(15, 70, 16), ivec3(16, 70, 16), ivec3(17, 70, 16),
        ivec3(15, 70, 17),
    ],
    &[
        ivec3(12, 71, 19), ivec3(13, 71, 19), ivec3(14, 71, 19), ivec3(15, 71, 19), ivec3(16, 71, 19), ivec3(17, 71, 19), ivec3(18, 71, 19),
        ivec3(12, 71, 20), ivec3(13, 71, 20), ivec3(14, 71, 20), ivec3(15, 71, 20), ivec3(16, 71, 20), ivec3(17, 71, 20), ivec3(18, 71, 20),
        ivec3(12, 71, 21), ivec3(13, 71, 21), ivec3(14, 71, 21), ivec3(15, 71, 21), ivec3(16, 71, 21), ivec3(17, 71, 21), ivec3(18, 71, 21),
        ivec3(12, 71, 22), ivec3(13, 71, 22), ivec3(14, 71, 22), ivec3(15, 71, 22), ivec3(16, 71, 22), ivec3(17, 71, 22), ivec3(18, 71, 22),
        ivec3(12, 71, 23), ivec3(13, 71, 23), ivec3(14, 71, 23), ivec3(15, 71, 23), ivec3(16, 71, 23), ivec3(17, 71, 23), ivec3(18, 71, 23),
        ivec3(12, 71, 24), ivec3(13, 71, 24), ivec3(14, 71, 24), ivec3(15, 71, 24), ivec3(16, 71, 24), ivec3(17, 71, 24), ivec3(18, 71, 24),
        ivec3(12, 71, 25), ivec3(13, 71, 25), ivec3(14, 71, 25), ivec3(15, 71, 25), ivec3(16, 71, 25), ivec3(17, 71, 25), ivec3(18, 71, 25),
        ivec3(15, 71, 26),
    ],
];

const OBSTACLES: [&[&[IVec3]]; 3] = [
    &[
        &[
            ivec3(14, 70, 7),
            ivec3(16, 70, 9),
        ],
    ],
    &[
        &[
            ivec3(15, 71, 13),
            ivec3(17, 71, 12),
            ivec3(17, 71, 16),
            ivec3(17, 71, 15),
            ivec3(16, 71, 16),
            ivec3(14, 71, 15),
        ]
    ],
    &[
        &[
            ivec3(14, 72, 19),
            ivec3(14, 72, 20),
            ivec3(16, 72, 21),
            ivec3(16, 72, 22),
            ivec3(17, 72, 22),
            ivec3(16, 72, 25),
            ivec3(13, 72, 24),
            ivec3(14, 72, 24),
        ]
    ],
];