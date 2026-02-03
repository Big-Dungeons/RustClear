use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::entities::npc::InteractableNPC;
use crate::dungeon::room::room::{Room, RoomStatus};
use crate::dungeon::room::room_implementation::RoomImplementation;
use crate::dungeon::seeded_rng::seeded_rng;
use glam::{ivec3, IVec3};
use rand::prelude::{IndexedRandom, SliceRandom};
use server::block::rotatable::Rotate;
use server::entity::entity::{EntityBase, EntityExtension};
use server::entity::entity_appearance::PlayerAppearance;
use server::network::packets::packet_buffer::PacketBuffer;
use server::network::protocol::play::clientbound::{BlockAction, Chat};
use server::network::protocol::play::serverbound::EntityInteractionType;
use server::types::chat_component::ChatComponent;
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

pub struct ThreeWeirdosPuzzle {
    player_clicked_weirdos: Rc<RefCell<HashMap<ClientId, [bool; 3]>>>,
    dialogues: [String; 3],
    names: [&'static str; 3],
    correct_chest: usize,
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
            player_clicked_weirdos: Rc::new(RefCell::new(Default::default())),
            dialogues,
            names,
            correct_chest,
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

            // todo, get the skins of the weirdos
            world.spawn_entity(
                position,
                yaw,
                0.0,
                PlayerAppearance::new(
                    self.names[index],
                    Default::default(),
                    "ewogICJ0aW1lc3RhbXAiIDogMTYxODc4MTA4Mzk0NywKICAicHJvZmlsZUlkIiA6ICJhNzdkNmQ2YmFjOWE0NzY3YTFhNzU1NjYxOTllYmY5MiIsCiAgInByb2ZpbGVOYW1lIiA6ICIwOEJFRDUiLAogICJzaWduYXR1cmVSZXF1aXJlZCIgOiB0cnVlLAogICJ0ZXh0dXJlcyIgOiB7CiAgICAiU0tJTiIgOiB7CiAgICAgICJ1cmwiIDogImh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvOWI1Njg5NWI5NjU5ODk2YWQ2NDdmNTg1OTkyMzhhZjUzMmQ0NmRiOWMxYjAzODliOGJiZWI3MDk5OWRhYjMzZCIsCiAgICAgICJtZXRhZGF0YSIgOiB7CiAgICAgICAgIm1vZGVsIiA6ICJzbGltIgogICAgICB9CiAgICB9CiAgfQp9",
                    "aNIhT2Tj20v1lONBOK3fIwBqJwWnjErq20h663Gb+PVmR9Iweh1h2ZEJ2pwDDnM4Af1XFDA5hS1Z9yOc8EdVTKyyi1yj9EIvMwQz/Q4N2sBsjWGZtCe8/Zy+X82iv0APB4cumE2gkgDbPjxCFNbpVKmV3U1WzwY/GKOMHofhWS1ULedQ1TszuMmDuHPLEzWaXigZ+xt5zChXvE8QoLTfBvgb8wtqVpyxAKf/o8xQduKiNE7t+de1CwOhLqbVTGh7DU0vLC5stDuqN+nC9dS7c2CG0ori6gFoGMvP4oIss6zm1nb0laMrZidJTgmuXk2Pv4NGDBXdYcAzhfWcSWGsBVMWrJfccgFheG+YcGYaYj6V2nBp0YTqqhN4wDt3ltyTNEMOr/JKyBTLzq/F7IL6rrdyMw+MbAgCa1FhfXxtzdQE2KsL55pbr2DZ8J4DYf+/OC1pWCJ4vvA/A1qGHyi3Zwtj9lCl1Jq5Qm2P9BgWxpk0ikJefRPMg4qWOEcYnjqwXuEp+IgTJi1xr+j/+g28aS1TsF8ijaJjSbEN4urrf3RYL+PZBcggzX9VaPB0NPdioOXznIotY+S6ZW7FnSh6UnrGAKadQBVLey5zmVWMfXlBUq9JMh0csuNd4dDQCLNK8oGORhMgksOMHhVaBie4otUgJ7ThR/WPjOAKiG2TNU0=",
                ),
                Weirdo {
                    player_clicked_weirdos: self.player_clicked_weirdos.clone(),
                    index,
                    dialogue: self.dialogues[index].clone(),
                    base: InteractableNPC {
                        default_yaw: yaw,
                        default_pitch: 0.0,
                        interact_callback: |_| {}
                    },
                }
            );
        }
    }

    fn interact(&mut self, room: &mut Room, player: &mut Player<DungeonPlayer>, position: IVec3) {
        // todo: improve accuracy
        // if matches!(room.status, RoomStatus::Failed) {
        //     player.write_packet(&Chat {
        //         component: ChatComponent::new("already failed"),
        //         chat_type: 0,
        //     });
        //     return;
        // }
        // if matches!(room.status, RoomStatus::Complete) {
        //     player.write_packet(&Chat {
        //         component: ChatComponent::new("already complete"),
        //         chat_type: 0,
        //     });
        //     return;
        // }
        for (index, relative_position) in CHEST_POSITIONS.into_iter().enumerate() {
            if position == room.get_world_block_position(relative_position) {
                if let Some([a, b, c]) = self.player_clicked_weirdos.borrow().get(&player.client_id) {
                    if *a && *b && *c {
                        if self.correct_chest == index {
                            player.write_packet(&Chat {
                                component: ChatComponent::new("correct"),
                                chat_type: 0,
                            });
                            // simplify
                            player.write_packet(&BlockAction {
                                block_pos: position,
                                event_id: 1,
                                event_data: 1,
                                block_id: 54,
                            });
                            room.status = RoomStatus::Complete
                        } else {
                            player.write_packet(&Chat {
                                component: ChatComponent::new("incorrect"),
                                chat_type: 0,
                            });
                            room.status = RoomStatus::Failed
                        }
                        player.world_mut().map.draw_checkmark(room)
                    } else {
                        player.write_packet(&Chat {
                            component: ChatComponent::new("talk to us"),
                            chat_type: 0,
                        });
                    }
                }
                return;
            }
        }
    }
}

struct Weirdo {
    player_clicked_weirdos: Rc<RefCell<HashMap<ClientId, [bool; 3]>>>,
    index: usize,
    dialogue: String,
    base: InteractableNPC,
}

impl EntityExtension<Dungeon> for Weirdo {
    fn tick(&mut self, entity: &mut EntityBase<Dungeon>, chunk_buffer: &mut PacketBuffer) {
        self.base.tick(entity, chunk_buffer)
    }
    fn interact(
        &mut self,
        _: &mut EntityBase<Dungeon>,
        player: &mut Player<DungeonPlayer>,
        _: EntityInteractionType,
    ) {
        player.write_packet(&Chat {
            component: ChatComponent::new(self.dialogue.as_str()),
            chat_type: 0,
        });
        let mut map = self.player_clicked_weirdos.borrow_mut();
        let value = map.entry(player.client_id).or_insert([false; 3]);
        value[self.index] = true
    }
}
