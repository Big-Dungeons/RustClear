use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::entities::npc::NPCBehaviour;
use crate::dungeon::room::room::{Room, RoomStatus};
use crate::dungeon::room::room_implementation::RoomImplementation;
use crate::dungeon::seeded_rng::seeded_rng;
use bevy_ecs::prelude::Component;
use glam::{ivec3, IVec3};
use rand::prelude::{IndexedRandom, SliceRandom};
use rand::rng;
use server::block::rotatable::Rotate;
use server::block::Block;
use server::constants::Sound;
use server::entity::components::entity_appearance::PlayerAppearance;
use server::entity::components::Interactable;
use server::entity::entity::MinecraftEntity;
use server::network::protocol::play::clientbound::{BlockAction, Chat};
use server::{ClientId, Player, World};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const CHEST_POSITIONS: [IVec3; 3] = [
    ivec3(18, 69, 24),
    ivec3(16, 69, 25),
    ivec3(14, 69, 24),
];

const WEIRDO_POSITIONS: [IVec3; 3] = [
    ivec3(17, 69, 24),
    ivec3(15, 69, 25),
    ivec3(13, 69, 24)
];

const WEIRDO_NAMES: [&str; 21] = [
    "§cArdis",
    "§cBaxter",
    "§cBenson",
    "§cCarver",
    "§cElmo",
    "§cEveleth",
    "§cHope",
    "§cHugo",
    "§cLino",
    "§cLuverne",
    "§cMadelia",
    "§cMarshall",
    "§cMelrose",
    "§cMontgomery",
    "§cMorris",
    "§cRamsey",
    "§cRose",
    "§cVictoria",
    "§cVirginia",
    "§cWillmar",
    "§cWinona",
];

const DIALOGUE: [[&str; 3]; 6] = [
    [
        "§e[NPC] §c1§r: The reward is not in my chest!",
        "§e[NPC] §c2§r: One of us is telling the truth!",
        "§e[NPC] §c3§r: They are both telling the truth. The reward isn't in §c1's§r chest.",
    ],
    [
        "§e[NPC] §c1§r: The reward isn't in any of our chests.",
        "§e[NPC] §c2§r: The reward is not in my chest. They are both lying.",
        "§e[NPC] §c3§r: The reward is in my chest!",
    ],
    [
        "§e[NPC] §c1§r: My chest doesn't have the reward. We are all telling the truth.",
        "§e[NPC] §c2§r: My chest doesn't have the reward. At least one of the others is telling the truth!",
        "§e[NPC] §c3§r: One of the others is lying!",
    ],
    [
        "§e[NPC] §c1§r: My chest has the reward and I'm telling the truth!",
        "§e[NPC] §c2§r: They are both lying, the reward is in my chest!",
        "§e[NPC] §c3§r: They are both telling the truth, the reward is in §c2's§r chest!",
    ],
    [
        "§e[NPC] §c1§r: At least one of them is lying, and the reward is not in §c3's§r chest!",
        "§e[NPC] §c2§r: We are all telling the truth!",
        "§e[NPC] §c3§r: §c2§r is telling the truth and the reward is in his chest.",
    ],
    [
        "§e[NPC] §c1§r: Both of them are telling the truth. Also, §c2§r has the reward in their chest!",
        "§e[NPC] §c2§r: §c3§r is telling the truth.",
        "§e[NPC] §c3§r: My chest has the reward!",
    ],
];

const SKINS: [(&str, &str); 3] = [
    (
        "eyJ0aW1lc3RhbXAiOjE1ODIxNDYwNjAxMDYsInByb2ZpbGVJZCI6ImEyZjgzNDU5NWM4OTRhMjdhZGQzMDQ5NzE2Y2E5MTBjIiwicHJvZmlsZU5hbWUiOiJiUHVuY2giLCJzaWduYXR1cmVSZXF1aXJlZCI6dHJ1ZSwidGV4dHVyZXMiOnsiU0tJTiI6eyJ1cmwiOiJodHRwOi8vdGV4dHVyZXMubWluZWNyYWZ0Lm5ldC90ZXh0dXJlLzdiNGM2ZjVkZjMxMzRhZGY0YTdlYWUxMmZlMjJlYjZhYTEwMmI2NzM1MjIxZTdmNTQ3NWM3YmJlYzQyMzdiYjgifX19",
        "rDW4GM5nUP2hvfh9it3pfXgGeaDoa+JEHoOefy5Rwruz2clabGqda1lXt527QWTAWieS4lFcNWnqwJUtzLow83i/kbFZ72MkUTo3c0LC3nFDTtABGijY8KfcIVRp0XHzWdQwG7PXWYt5RvX+RgEdOmd+yhDoq16Cf4d3MhWhuFrSpKJohzvQ3ad/FFXdpSiWmklnsQ2n7ZP1ZRzuWWg4kRdtYEEjE2oodVkQoN8xqtddK+eT/3kz9n/aqPfokAHjWMJDbkqPBLweLVK2+WYkI9c6unHcG/uWKwhw8lwG7oEXLNhtDnipoWqA+TNcP//m8DAF9kA2MeBjO72U2v+UkNIGXZPamy5wSqhoNhyTAmG0MsammQprwfzL/K3PVW5QZxIldAIDMFNn/T6tYH2PtT345A+0gC0xtZUXHjscjlok/dcvYyleHyxK15fPyYtxcmGE59AUjj0Xllv90aEECRHrzC3t+2/gj+nWcDrLPvxX/qbjlTXKyxT/V0vJlMrzfoj8apPHgdj3S3mu3XDog18kfj7iPmoN0X1xllGzgR4SmOqnlCSWKFieYx7wrbN9J1y23itVteto9DiMWKbgc314m6nxGSaiSVSZriMX4lciNv1js7ADkyQ5LX3FuWUe7KRJuHYv/aRSzj70IEHq+/G6I5EHd3WbRJKmM6AMg4Q=",
    ),
    (
        "eyJ0aW1lc3RhbXAiOjE1ODIxNDU3Njc3MDEsInByb2ZpbGVJZCI6ImEyZjgzNDU5NWM4OTRhMjdhZGQzMDQ5NzE2Y2E5MTBjIiwicHJvZmlsZU5hbWUiOiJiUHVuY2giLCJzaWduYXR1cmVSZXF1aXJlZCI6dHJ1ZSwidGV4dHVyZXMiOnsiU0tJTiI6eyJ1cmwiOiJodHRwOi8vdGV4dHVyZXMubWluZWNyYWZ0Lm5ldC90ZXh0dXJlLzE0Nzk0Yjc3N2Y5YmIzNDhjNTBiMTlhNTAwZDk2ODkwNGM1NzAwNTRlZGMxNzhhNTIwYzRlMWEyZGEwZDY1ODcifX19",
        "k+o7OResg2zOWnG/s2kdEWgEytoj4sVcaFHU0oiLkhwZfxp9NpEeGdTC3A8PdiGiLGAMGJnmBjgXPY0iyJJbfpe+f8FLb/F0FpFPT8P6t7Et9t/jep8ZtAKRQF8kKkgTvDljycSSQQod/DXHOuX/9LCb0UytSc8FFeaTKStTqbGpAOA9Wgb3cF9Mg5DmKC3RJwOeMw3G1nnWrJuG8W+6Uf7oR8f4+M6DBxay3GDNRNBVRQfFxovd/3T0f9VBCenn5ednB+T4t5h8pEssUVmgGiHLwJONxfQolzQkgb7sVC8I9oi74JDwAg3k4rKFb64YTbVKsvbmqb+sqfrcbZUQhPLR9BgNKlWr/A3SokplfjraK7+m90B/vM4jZiFnexxpW1TwNGZkrDrkozNckYRtTK2j9PmOwUgscrRMl4pMNMe7bPDpb1w8PeAzXMSTYzcvVQDK4rFuGCviwWq5JhQnbI+sFTHCJtxixH1AQSWTnLIqUqcKNtQsVYgoN3AGpaj3v5cqoGHh2WPjo1vOMNQN1VjCpNGMiNEJhf9xxfigh4bdC4thH4NMBKavXeMedJ5M9azmBo30b9u0YT3nYbqrx82D4HxagmKeb2+j2O3StdGWk1VUUfxUpwQ4mR9nMKfN1k2JYewog3uxGcJ9FGAcOflvQqBwSIVrMnZgtHFpuQ0=",
    ),
    (
        "eyJ0aW1lc3RhbXAiOjE1ODIwNTY1MjY1ODksInByb2ZpbGVJZCI6ImEyZjgzNDU5NWM4OTRhMjdhZGQzMDQ5NzE2Y2E5MTBjIiwicHJvZmlsZU5hbWUiOiJiUHVuY2giLCJzaWduYXR1cmVSZXF1aXJlZCI6dHJ1ZSwidGV4dHVyZXMiOnsiU0tJTiI6eyJ1cmwiOiJodHRwOi8vdGV4dHVyZXMubWluZWNyYWZ0Lm5ldC90ZXh0dXJlLzYxOTBhN2IyYmIxM2FlNTgyM2Y2YTE4NDZmODQyYzM0ODllMjYyMGEzNjY1NTc0YTBmYmE5NzVjMzk0MTA5MjIifX19",
        "D/AAFPkwp3dqEx8OktDtqX0PSwJfu6PS+u67e8mq+0FMz+yqvDhD4FmzlvJlz6dwVa+UWGBCX6CMPXbPja9eeR90GFEYU+AYInam8IvyrmDzw7q0Fx3jzP9aRmHSn4229Y8GXhkOJ37k3pWf5zrcIJmT9npIq4lwEc3B0OxEZtQadanWX0/qIr/bpbrB+en2zIWzzwQWIAXPJUwQgiVj7mRwfMCyajoOqGs0AApzTi5IPresYF2BZZ9pLWyLv96YFhm96ncMHVJlSl3h8mt0R1pGi2BwOROYIfFq6HDONpSfD3R7aaty0fyPeV9kcrswndCS5/ubZxvv1bLp2wqhR0A5NzWr2GM3GK7o2EQgM5o9gsKS65SGPaWF0h3dUzsrpCOSMKzxzj29eAP4TLsLxWNAyaR/Q4NQt8cluiisLSk2yKUDovzUiSqrdLToD+5DPFqNYxDRraCc2gQQlsHpp3aXHNpqoBYcczTXwUHgjqh71HodzXGo4pxOcJNo5kOjV+uyfgvR9zCIuoN3j7UK6E2F3LQrLDTTRb8W4KGkMDnEESv8jGrXwQs4WzqUdP4HHXOFHbp2cx0Pi7xc93MQW1ZmjNtDeLr+xvsqDbL2syI8D2mf2++vjLhPSzxqtx4hOcyWQFCMC9Sdzb/bDg8JAuXgQSAMgmp1E1Oz/msZJ1w="
    ),
];

pub struct ThreeWeirdosPuzzle {
    shared: Rc<RefCell<Shared>>,
}

#[derive(Component)]
struct Weirdo {
    index: usize,
    puzzle: Rc<RefCell<Shared>>,
}

unsafe impl Send for Weirdo {}
unsafe impl Sync for Weirdo {}

#[derive(Default)]
struct Shared {
    clicked_weirdos: HashMap<ClientId, [bool; 3]>,
    has_clicked_any_chest: bool,

    names: [&'static str; 3],
    dialogues: [String; 3],
    correct_index: usize,
}

impl Default for ThreeWeirdosPuzzle {
    fn default() -> Self {
        let mut rng = seeded_rng();

        let chosen_names: [&str; 3] = WEIRDO_NAMES
            .choose_multiple(&mut rng, 3)
            .cloned()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let chosen_dialogue = *DIALOGUE
            .choose(&mut rng)
            .expect("Failed to choose dialogue");

        let mut indices: [usize; 3] = [0, 1, 2];
        indices.shuffle(&mut rng);

        let mut dialogues: [String; 3] = [String::new(), String::new(), String::new()];
        let mut names: [&str; 3] = [""; 3];
        let mut correct_chest = 0;

        for (index, shuffled_index) in indices.iter().enumerate() {
            let npc_name = chosen_names[*shuffled_index];
            let template_text = chosen_dialogue[*shuffled_index];

            let text = template_text
                .replace("1", chosen_names[0])
                .replace("2", chosen_names[1])
                .replace("3", chosen_names[2]);

            dialogues[index] = text;
            names[index] = npc_name;

            if *shuffled_index == 0 {
                correct_chest = index;
            }
        }

        Self {
            shared: Rc::new(RefCell::new(Shared {
                clicked_weirdos: Default::default(),
                has_clicked_any_chest: false,
                names,
                dialogues,
                correct_index: correct_chest,
            })),
        }
    }
}

impl RoomImplementation for ThreeWeirdosPuzzle {
    fn discover(&mut self, room: &mut Room, world: &mut World<Dungeon>) {
        for (index, position) in WEIRDO_POSITIONS.into_iter().enumerate() {
            let mut position = room.get_world_block_position(position).as_dvec3();
            position.x += 0.5;
            position.z += 0.5;
            let yaw = 180.0.rotate(room.rotation);

            let (texture, signature) = SKINS[index];

            world.spawn_entity(
                position,
                yaw,
                0.0,
                PlayerAppearance::new(
                    self.shared.borrow().names[index],
                    Default::default(),
                    texture,
                    signature
                ),
                (
                    Weirdo {
                        index,
                        puzzle: self.shared.clone(),
                    },
                    NPCBehaviour {
                        default_yaw: yaw,
                        default_pitch: 0.0
                    },
                    Interactable::<Dungeon>::new(|entity, player| {
                        let mc_entity = entity.get::<MinecraftEntity<Dungeon>>().unwrap();
                        let weirdo = entity.get::<Weirdo>().unwrap();
                        let mut puzzle = weirdo.puzzle.borrow_mut();
                        let value = puzzle.clicked_weirdos.entry(player.client_id).or_insert([false; 3]);
                        value[weirdo.index] = true;

                        if !puzzle.has_clicked_any_chest {
                            player.send_message(puzzle.dialogues[weirdo.index].as_str());
                        } else {
                            const FINISHED_DIALOGUE: [&str; 4] = [
                                "§e[NPC] §cname§f: You're free to leave.§7",
                                "§e[NPC] §cname§f: You can leave now! Bye!§7",
                                "§e[NPC] §cname§f: Thanks for playing! Now get out!§7",
                                "§e[NPC] §cname§f: Scram!§7",
                            ];
                            let chosen = FINISHED_DIALOGUE.choose(&mut rng()).unwrap();
                            player.send_message(&chosen.replace("name", puzzle.names[weirdo.index]));
                        }

                        player.play_sound_at(
                            Sound::DonkeyHit,
                            1.0,
                            0.5,
                            mc_entity.position
                        );
                    })
                )
            );
        }
    }


    fn interact(&mut self, room: &mut Room, player: &mut Player<DungeonPlayer>, position: IVec3) {
        if matches!(room.status, RoomStatus::Complete | RoomStatus::Failed) {
            return;
        }

        let mut data = self.shared.borrow_mut();

        for (index, relative_position) in CHEST_POSITIONS.into_iter().enumerate() {
            let real_position = room.get_world_block_position(relative_position);
            if position != real_position {
                continue
            }
            if let Some([a, b, c]) = data.clicked_weirdos.get(&player.client_id) {
                if !*a || !*b || !*c {
                    player.write_packet(&Chat::new("talk to us"));
                    return;
                }
                if data.correct_index == index {
                    player.world_mut().write_global_packet(&Chat::new(
                        &format!("§aPUZZLE SOLVED! §7{} §ewasn't fooled by §c{}§e! §4G§co§6o§ed §2j§bo§3b§5!", player.profile.username, data.names[index]),
                    ));

                    // maybe some form of block entity system for chests.
                    // to allow it being opened?
                    player.write_packet(&BlockAction {
                        block_pos: position,
                        event_id: 1,
                        event_data: 1,
                        block_id: 54,
                    });

                    room.status = RoomStatus::Complete
                } else {
                    let world = player.world_mut();

                    player.send_message(&format!("§e[NPC] §c{}§f: You fool!", data.names[index]));
                    world.write_global_packet(&Chat::new(
                        &format!("§cPUZZLE FAIL! §7{} §ewas fooled by §c{}§e! §4Y§ci§6k§ee§as§2!", player.profile.username, data.names[index]),
                    ));
                    player.play_sound_at(
                        Sound::RandomExplode,
                        1.0,
                        1.0,
                        real_position.as_dvec3()
                    );

                    // maybe
                    for rel_pos in CHEST_POSITIONS {
                        let IVec3 { x, y, z } = room.get_world_block_position(rel_pos);
                        world.chunk_grid.set_block_at(Block::Air, x, y, z);
                    }

                    room.status = RoomStatus::Failed
                }
                data.has_clicked_any_chest = true;
                player.world_mut().map.draw_checkmark(room)
            }
            return;
        }

    }
}
