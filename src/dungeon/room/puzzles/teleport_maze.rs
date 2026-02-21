use crate::dungeon::dungeon::{Dungeon, DungeonState};
use crate::dungeon::room::room::Room;
use crate::dungeon::room::room_implementation::RoomImplementation;
use crate::dungeon::seeded_rng::{seeded_rng, SeededRng};
use glam::{dvec3, ivec3, IVec3};
use rand::prelude::SliceRandom;
use rand::Rng;
use server::block::rotatable::Rotate;
use server::network::protocol::play::clientbound::PositionLook;
use server::types::aabb::AABB;
use server::World;

pub struct TeleportMaze {
    teleport_pads: Vec<TeleportPad>,
    target: IVec3,
}

struct TeleportPad {
    position: IVec3,
    teleports_to: IVec3,
    rotate_type: TeleportPadRotation,
}

enum TeleportPadRotation {
    Static { yaw: f32 },
    TowardsEnd
}

impl TeleportPad {
    pub fn aabb(&self) -> AABB {
        let position = self.position.as_dvec3();
        AABB::new(position, position + dvec3(1.0, 0.1, 1.0))
    }
}

impl Default for TeleportMaze {
    fn default() -> Self {
        let mut teleport_pads: Vec<TeleportPad> = Vec::new();

        // not the prettiest
        let rng = &mut seeded_rng();
        let mut sections: Vec<Vec<(IVec3, IVec3)>> = Vec::new();

        for index in 0..7 {
            let offset = match index {
                0 => ivec3(20, 70, 6),
                1 => ivec3(20, 70, 14),
                2 => ivec3(20, 70, 22),
                3 => ivec3(12, 70, 22),
                4 => ivec3(4, 70, 22),
                5 => ivec3(4, 70, 14),
                6 => ivec3(4, 70, 6),
                _ => unreachable!()
            };
            sections.push(vec![
                (offset, offset + ivec3(1, -1, 1)),
                (offset + ivec3(6, 0, 0), offset + ivec3(5, -1, 1)),
                (offset + ivec3(0, 0, 6), offset + ivec3(1, -1, 5)),
                (offset + ivec3(6, 0, 6), offset + ivec3(5, -1, 5)),
            ]);
        }

        sections.shuffle(rng);

        const ENTRANCE_PAD: IVec3 = ivec3(15, 70, 12);
        const EXIT_PAD: IVec3 = ivec3(15, 70, 14);
        const ENTRANCE_PAD_TO: IVec3 = ivec3(15, 69, 11);
        const EXIT_PAD_TO: IVec3 = ivec3(15, 69, 15);

        // entrance
        let pads = &mut sections[0];
        let (pad, to_pad) = pop_with_rng(pads, rng);

        teleport_pads.push(TeleportPad {
            position: ENTRANCE_PAD,
            teleports_to: to_pad,
            rotate_type: TeleportPadRotation::TowardsEnd,
        });
        teleport_pads.push(TeleportPad {
            position: pad,
            teleports_to: ENTRANCE_PAD_TO,
            rotate_type: TeleportPadRotation::Static { yaw: 180.0 },
        });

        // exit
        let pads = &mut sections[6];
        let (pad_leading_to_end, _) = pop_with_rng(pads, rng);

        teleport_pads.push(TeleportPad {
            position: pad_leading_to_end,
            teleports_to: EXIT_PAD_TO,
            rotate_type: TeleportPadRotation::Static { yaw: 0.0 },
        });
        teleport_pads.push(TeleportPad {
            position: EXIT_PAD,
            teleports_to: ENTRANCE_PAD_TO,
            rotate_type: TeleportPadRotation::Static { yaw: 180.0 },
        });

        // this makes sure every section is able to be travelled to
        for index in 0..6 {
            let section_a = &mut sections[index];
            let (pad_a, to_pad_a) = pop_with_rng(section_a, rng);
            let section_b = &mut sections[index + 1];
            let (pad_b, to_pad_b) = pop_with_rng(section_b, rng);

            teleport_pads.push(TeleportPad {
                position: pad_a,
                teleports_to: to_pad_b,
                rotate_type: TeleportPadRotation::TowardsEnd,
            });
            teleport_pads.push(TeleportPad {
                position: pad_b,
                teleports_to: to_pad_a,
                rotate_type: TeleportPadRotation::TowardsEnd,
            });
        }

        let mut remaining: Vec<(usize, IVec3, IVec3)> = sections
            .iter()
            .enumerate()
            .flat_map(|(section_idx, pads)| {
                pads.iter().map(move |&(pad, to_pad)| (section_idx, pad, to_pad))
            })
            .collect();

        remaining.shuffle(rng);

        let mut index = 0;
        while index + 1 < remaining.len() {
            if remaining[index].0 == remaining[index + 1].0 {
                let j = remaining[index + 2..]
                    .iter()
                    .position(|(s, _, _)| *s != remaining[index].0)
                    .expect("no valid cross-section partner found");

                remaining.swap(index + 1, index + 2 + j);
            }

            let (_, pad_a, to_pad_a) = remaining[index];
            let (_, pad_b, to_pad_b) = remaining[index + 1];
            teleport_pads.push(TeleportPad {
                position: pad_a,
                teleports_to: to_pad_b,
                rotate_type: TeleportPadRotation::TowardsEnd,
            });
            teleport_pads.push(TeleportPad {
                position: pad_b,
                teleports_to: to_pad_a,
                rotate_type: TeleportPadRotation::TowardsEnd,
            });

            index += 2;
        }

        Self {
            teleport_pads,
            target: pad_leading_to_end,
        }
    }
}

impl RoomImplementation for TeleportMaze {
    fn discover(&mut self, room: &mut Room, _world: &mut World<Dungeon>) {
        // convert teleport pad data to absolute position
        for pad in self.teleport_pads.iter_mut() {
            pad.position = room.get_world_block_position(pad.position);
            pad.teleports_to = room.get_world_block_position(pad.teleports_to);
        }
        self.target = room.get_world_block_position(self.target);

        // for pad in self.teleport_pads.iter() {
        //     _world.spawn_entity(
        //         pad.position.as_dvec3(),
        //         0.0,
        //         0.0,
        //         MobAppearance {
        //             variant: EntityVariant::Zombie,
        //             metadata: EntityMetadata::Zombie(Default::default()),
        //         },
        //         ()
        //     );
        // }
    }

    fn tick(&mut self, room: &mut Room, world: &mut World<Dungeon>) {
        // todo, improve accuracy
        if let DungeonState::Started { ticks } = world.state && ticks % 10 != 0 {
            return;
        }
        'outer: for player in room.players() {
            let player_aabb = player.collision_aabb();
            for pad in self.teleport_pads.iter() {
                if !pad.aabb().intersects(&player_aabb) {
                    continue
                }

                let mut origin = pad.teleports_to.as_dvec3();
                origin.x += 0.5;
                origin.y += 0.5;
                origin.z += 0.5;

                let yaw = match pad.rotate_type {
                    TeleportPadRotation::Static { yaw } => {
                        yaw.rotate(room.rotation)
                    },
                    TeleportPadRotation::TowardsEnd => {
                        let mut target = self.target.as_dvec3();
                        target.x += 0.5;
                        target.z += 0.5;
                        let diff = target - origin;
                        f64::atan2(-diff.x, diff.z).to_degrees() as f32
                    }
                };
                player.write_packet(&PositionLook {
                    x: origin.x,
                    y: origin.y,
                    z: origin.z,
                    yaw,
                    pitch: 0.0,
                    flags: Default::default(),
                });
                continue 'outer
            }
        }
    }

    // todo interact
}

fn pop_with_rng(vec: &mut Vec<(IVec3, IVec3)>, rng: &mut SeededRng) -> (IVec3, IVec3) {
    let pad_index = rng.random_range(0..vec.len());
    let it = vec[pad_index];
    vec.remove(pad_index);
    it
}